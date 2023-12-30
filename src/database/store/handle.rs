use crate::{
    common::{store::Field, tree::Path},
    database::{
        interact::{apply, diff, drop, export, Batch},
        store::{Cell, Label},
    },
    map::store::Node as MapNode,
};

use oh_snap::Snap;
use serde::de::DeserializeOwned;

use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash as StdHash,
    ptr, sync::RwLock,
};

use talk::crypto::primitives::hash::Hash;

pub(crate) struct Handle<Key: Field + DeserializeOwned, Value: Field + DeserializeOwned> {
    pub cell: Cell<Key, Value>,
    pub root: RwLock<Label>,
}

impl<Key, Value> Handle<Key, Value>
where
    Key: Field + DeserializeOwned,
    Value: Field + DeserializeOwned,
{
    pub fn empty(cell: Cell<Key, Value>) -> Self {
        Handle {
            cell,
            root: RwLock::new(Label::Empty),
        }
    }

    pub fn new(cell: Cell<Key, Value>, root: Label) -> Self {
        Handle { cell, root: RwLock::new(root) }
    }

    pub fn commit(&self) -> Hash {
        self.root.read().unwrap().hash().into()
    }

    pub fn apply(&self, batch: Batch<Key, Value>) -> Batch<Key, Value> {

        let store = self.cell.take();


        let (store, root, batch) = apply::apply(store, self.root.read().unwrap().clone(), batch);

        self.cell.restore(store);
        *self.root.write().unwrap() = root;

        batch
    }

    pub fn export(&self, paths: Snap<Path>) -> MapNode<Key, Value>
    where
        Key: Clone,
        Value: Clone,
    {
        let store = self.cell.take();
        let (store, root) = export::export(store, self.root.read().unwrap().clone(), paths);
        self.cell.restore(store);

        root
    }

    pub fn diff(
        lho: &Handle<Key, Value>,
        rho: &Handle<Key, Value>,
    ) -> HashMap<Key, (Option<Value>, Option<Value>)>
    where
        Key: Clone + Eq + StdHash,
        Value: Clone + Eq,
    {
        if !ptr::eq(lho.cell.as_ref(), rho.cell.as_ref()) {
            panic!("called `Handle::diff` on two `Handle`s for different `Store`s (most likely, `Table::diff` / `Collection::diff` was called on two objects belonging to different `Database`s / `Family`-es)");
        }

        let store = lho.cell.take();

        let (store, lho_candidates, rho_candidates) = diff::diff(store, lho.root.read().unwrap().clone(), rho.root.read().unwrap().clone());

        lho.cell.restore(store);

        let mut diff: HashMap<Key, (Option<Value>, Option<Value>)> = HashMap::new();

        for (key, value) in lho_candidates {
            let key = (**key.inner()).clone();
            let value = (**value.inner()).clone();

            diff.insert(key, (Some(value), None));
        }

        for (key, value) in rho_candidates {
            let key = (**key.inner()).clone();
            let value = (**value.inner()).clone();

            match diff.entry(key) {
                Entry::Occupied(mut entry) => {
                    if entry.get().0.as_ref().unwrap() == &value {
                        entry.remove_entry();
                    } else {
                        entry.get_mut().1 = Some(value);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert((None, Some(value)));
                }
            }
        }

        diff
    }
}

impl<Key, Value> Clone for Handle<Key, Value>
where
    Key: Field + DeserializeOwned,
    Value: Field + DeserializeOwned,
{
    fn clone(&self) -> Self {
        let mut store = self.cell.take();
        store.incref(self.root.read().unwrap().clone());
        self.cell.restore(store);

        Handle {
            cell: self.cell.clone(),
            root: RwLock::new(self.root.read().unwrap().clone()),
        }
    }
}

impl<Key, Value> Drop for Handle<Key, Value>
where
    Key: Field + DeserializeOwned,
    Value: Field + DeserializeOwned,
{
    fn drop(&mut self) {
        let mut store = self.cell.take();
        drop::drop(&mut store, self.root.read().unwrap().clone());
        self.cell.restore(store);
    }
}
