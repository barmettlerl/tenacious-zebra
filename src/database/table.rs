use crate::{
    common::{data::Bytes, store::Field, tree::{Path, Prefix}},
    database::{
        errors::QueryError,
        store::{Cell, Handle, Label},
        TableResponse, TableSender, TableTransaction,
    },
    map::Map,
};
use doomstack::{here, ResultExt, Top};

use oh_snap::Snap;
use std::{borrow::Borrow, collections::{hash_map::Entry::{Occupied, Vacant}, HashMap}, hash::Hash as StdHash};

use talk::crypto::primitives::{hash, hash::Hash};

// Documentation links
#[allow(unused_imports)]
use crate::database::{Database, TableReceiver};

use super::store::{Store, Node, Wrap};

/// A map implemented using Merkle Patricia Trees.
///
/// Allows for:
/// 1) Concurrent processing of operations on different keys with minimal
/// thread synchronization.
/// 2) Cheap cloning (O(1)).
/// 3) Efficient sending to [`Database`]s containing similar maps (high % of
/// key-value pairs in common)
///
/// [`Database`]: crate::database::Database
/// [`Table`]: crate::database::Table
/// [`Transaction`]: crate::database::TableTransaction
/// [`TableSender`]: crate::database::TableSender
/// [`TableReceiver`]: crate::database::TableReceiver

pub struct Table<Key: Field, Value: Field>(Handle<Key, Value>, String);

impl<Key, Value> Table<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub(crate) fn empty(cell: Cell<Key, Value>, name: String) -> Self {
        Table(Handle::empty(cell), name)
    }

    pub(crate) fn new(cell: Cell<Key, Value>, root: Label, name:String) -> Self {
        Table(Handle::new(cell, root), name)
    }

    pub(crate) fn from_handle(handle: Handle<Key, Value>, name: String) -> Self {
        Table(handle, name)
    }

    /// Returns a cryptographic commitment to the contents of the `Table`.
    pub fn commit(&self) -> Hash {
        self.0.commit()
    }

    pub(crate) fn get_root(&self) -> Label {
        *self.0.root.read().unwrap()
    }

    pub(crate) fn get_name(&self) -> String {
        self.1.clone()
    }

    /// Executes a [`TableTransaction`] returning a [`TableResponse`]
    /// (see their respective documentations for more details).
    ///
    /// # Examples
    ///
    /// ```
    /// use tenaciouszebra::database::{Database, TableTransaction};
    ///
    /// fn main() {
    ///
    ///     let mut database = Database::new();
    ///
    ///     // Create a new transaction.
    ///     let mut transaction = TableTransaction::new();
    ///
    ///     // Set (key = 0, value = 0)
    ///     transaction.set(0, 0).unwrap();
    ///
    ///     // Remove record with (key = 1)
    ///     transaction.remove(&1).unwrap();
    ///
    ///     // Read records with (key = 2)
    ///     let read_key = transaction.get(&2).unwrap();
    ///
    ///     let mut table = database.empty_table("test");
    ///     
    ///     // Executes the transaction, returning a response.
    ///     let response = table.execute(transaction);
    ///
    ///     let value_read = response.get(&read_key);
    ///     assert_eq!(value_read, None);
    /// }
    /// ```
    pub fn execute(
        &self,
        transaction: TableTransaction<Key, Value>,
    ) -> TableResponse<Key, Value> {
        let (tid, batch) = transaction.finalize();

        let batch = self.0.apply(batch);
        TableResponse::new(tid, batch)
    }

    pub fn export<I, K>(&self, keys: I) -> Result<Map<Key, Value>, Top<QueryError>>
    // TODO: Decide if a `QueryError` is appropriate here
    where
        Key: Clone,
        Value: Clone,
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
    {
        let paths: Result<Vec<Path>, Top<QueryError>> = keys
            .into_iter()
            .map(|key| {
                hash::hash(key.borrow())
                    .pot(QueryError::HashError, here!())
                    .map(|digest| Path::from(Bytes::from(digest)))
            })
            .collect();

        let mut paths = paths?;
        paths.sort();
        let paths = Snap::new(paths);

        let root = self.0.export(paths);
        Ok(Map::raw(root))
    }

    pub fn diff(
        lho: &Table<Key, Value>,
        rho: &Table<Key, Value>,
    ) -> HashMap<Key, (Option<Value>, Option<Value>)>
    where
        Key: Clone + Eq + StdHash,
        Value: Clone + Eq,
    {
        Handle::diff(&lho.0, & rho.0)
    }

    /// Transforms the table into a [`TableSender`], preparing it for sending to
    /// to a [`TableReceiver`] of another [`Database`]. For details on how to use
    /// Senders and Receivers check their respective documentation.
    /// ```
    /// use tenaciouszebra::database::Database;
    ///
    /// let mut database: Database<u32, u32> = Database::new();
    /// let original = database.empty_table("test");
    ///
    /// // Sending consumes the copy so we typically clone first, which is cheap.
    /// let copy = original.clone();
    /// let sender = copy.send();
    ///
    /// // Use sender...
    /// ```
    pub fn send(&self) -> TableSender<Key, Value> {
        TableSender::from_handle(self.0.clone())
    }

    fn check_internal(store: &mut Store<Key,Value>, label: Label) {
        let (left, right) = Self::fetch_internal(store, label);

        match (left, right) {
            (Label::Empty, Label::Empty)
            | (Label::Empty, Label::Leaf(..))
            | (Label::Leaf(..), Label::Empty) => {
                panic!("`check_internal`: children violate compactness")
            }
            _ => {}
        }

        for child in [left, right] {
            if child != Label::Empty {
                if let Vacant(..) = store.entry(child) {
                    panic!("`check_internal`: child not found");
                }
            }
        }
    }

    fn check_leaf(store: &mut Store<Key,Value>, label: Label, location: Prefix) {
        let (key, _) = Self::fetch_leaf(store, label);
        if !location.contains(&Path::from(key.digest())) {
            panic!("`check_leaf`: leaf outside of its key path")
        }
    }

    fn fetch_node(store: &mut Store<Key,Value>, label: Label) -> Node<Key, Value> {
        match store.entry(label) {
            Occupied(entry) => entry.get().node.clone(),
            Vacant(..) => panic!("`fetch_node`: node not found"),
        }
    }

    fn fetch_internal(store: &mut Store<Key,Value>, label: Label) -> (Label, Label) {
        match Self::fetch_node(store, label) {
            Node::Internal(left, right) => (left, right),
            _ => panic!("`fetch_internal`: node not `Internal`"),
        }
    }

    fn fetch_leaf(store: &mut Store<Key,Value>, label: Label) -> (Wrap<Key>, Wrap<Value>) {
        match Self::fetch_node(store, label) {
            Node::Leaf(key, value) => (key, value),
            _ => panic!("`fetch_leaf`: node not `Leaf`"),
        }
    }

    fn check_tree_recursion(store: &mut Store<Key, Value>, label: Label, location: Prefix)
    {
        match label {
            Label::Internal(..) => {
                Self::check_internal(store, label);

                let (left, right) = Self::fetch_internal(store, label);
                Self::check_tree_recursion(store, left, location.left());
                Self::check_tree_recursion(store, right, location.right());
            }
            Label::Leaf(..) => {
                Self::check_leaf(store, label, location);
            }
            Label::Empty => {}
        }
    }

    pub(crate) fn check(&self) {
        let mut store = self.0.cell.take();

        Self::check_tree_recursion(&mut store, self.get_root(), Prefix::root());

        self.0.cell.restore(store);
    }
}

impl<Key, Value> Clone for Table<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn clone(&self) -> Self {
        Table(self.0.clone(), self.1.clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use rand::seq::IteratorRandom;

    use std::{fmt::Debug, hash::Hash, collections::HashMap};

    impl<Key, Value> Table<Key, Value>
    where
        Key: Field,
        Value: Field,
    {
        pub(crate) fn root(&self) -> Label {
            self.0.root.read().unwrap().clone()
        }

        pub(crate) fn assert_records<I>(&self, reference: I)
        where
            Key: Debug + Clone + Eq + Hash,
            Value: Debug + Clone + Eq + Hash,
            I: IntoIterator<Item = (Key, Value)>,
        {
            let mut store = self.0.cell.take();
            store.assert_records(*self.0.root.read().unwrap(), reference);
            self.0.cell.restore(store);
        }

    }

    #[test]
    fn export_empty() {
        let database: Database<u32, u32> = Database::new();
        let table = database.empty_table("test");

        let map = table.export::<[u32; 0], u32>([]).unwrap(); // Explicit type arguments are to aid type inference on an empty array

        map.check_tree();
        map.assert_records([]);

        table.check();
        table.assert_records([]);
    }

    #[test]
    fn export_none() {
        let database: Database<u32, u32> = Database::new();
        let table = database.empty_table("test");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        table.execute(transaction);

        let map = table.export::<[u32; 0], u32>([]).unwrap(); // Explicit type arguments are to aid type inference on an empty array

        map.check_tree();
        map.assert_records([]);

        table.check();
        table.assert_records((0..1024).map(|i| (i, i)));
    }

    #[test]
    fn export_single() {
        let database: Database<u32, u32> = Database::new();
        let table = database.empty_table("test");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        table.execute(transaction);

        let map = table.export([33]).unwrap();

        map.check_tree();
        map.assert_records([(33, 33)]);

        table.check();
        table.assert_records((0..1024).map(|i| (i, i)));
    }

    #[test]
    fn export_half() {
        let database: Database<u32, u32> = Database::new();
        let table = database.empty_table("test");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        table.execute(transaction);

        let map = table.export(0..512).unwrap();
        println!("{:?}", map);

        map.check_tree();
        map.assert_records((0..512).map(|i| (i, i)));

        table.check();
        table.assert_records((0..1024).map(|i| (i, i)));
    }

    #[test]
    fn export_all() {
        let database: Database<u32, u32> = Database::new();
        let table = database.empty_table("test");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        table.execute(transaction);

        let map = table.export(0..1024).unwrap();
        map.check_tree();
        map.assert_records((0..1024).map(|i| (i, i)));

        table.check();
        table.assert_records((0..1024).map(|i| (i, i)));
    }

    #[test]
    fn diff_empty_empty() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        assert_eq!(Table::diff(&mut lho, &mut rho), HashMap::new());
    }

    #[test]
    fn diff_identity_empty() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            assert_eq!(diff[&key], (Some(key), None));
        }

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            assert_eq!(diff[&key], (None, Some(key)));
        }
    }

    #[test]
    fn diff_identity_match() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        rho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);
        assert_eq!(diff, HashMap::new());

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);
        assert_eq!(diff, HashMap::new());
    }

    #[test]
    fn diff_identity_successor() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i + 1)) {
            transaction.set(key, value).unwrap();
        }

        rho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            assert_eq!(diff[&key], (Some(key), Some(key + 1)));
        }

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            assert_eq!(diff[&key], (Some(key + 1), Some(key)));
        }
    }

    #[test]
    fn diff_first_identity_match_rest_successor() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let mut transaction = TableTransaction::new();
        transaction.set(0, 0).unwrap();

        for (key, value) in (1..1024).map(|i| (i, i + 1)) {
            transaction.set(key, value).unwrap();
        }

        rho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            if key == 0 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (Some(key), Some(key + 1)));
            }
        }

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            if key == 0 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (Some(key + 1), Some(key)));
            }
        }
    }

    #[test]
    fn diff_half_identity_match_half_successor() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let mut transaction = TableTransaction::new();

        for (key, value) in (0..512).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        for (key, value) in (512..1024).map(|i| (i, i + 1)) {
            transaction.set(key, value).unwrap();
        }

        rho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            if key < 512 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (Some(key), Some(key + 1)));
            }
        }

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1024 {
            if key < 512 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (Some(key + 1), Some(key)));
            }
        }
    }

    #[test]
    fn diff_identity_overlap() {
        let database: Database<u32, u32> = Database::new();

        let mut lho = database.empty_table("test");
        let mut rho = database.empty_table("test2");

        let mut transaction = TableTransaction::new();
        for (key, value) in (0..1024).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        lho.execute(transaction);

        let mut transaction = TableTransaction::new();

        for (key, value) in (512..1536).map(|i| (i, i)) {
            transaction.set(key, value).unwrap();
        }

        rho.execute(transaction);

        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1536 {
            if key < 512 {
                assert_eq!(diff[&key], (Some(key), None));
            } else if key < 1024 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (None, Some(key)));
            }
        }

        let (mut lho, mut rho) = (rho, lho);
        let diff = Table::diff(&mut lho, &mut rho);

        for key in 0..1536 {
            if key < 512 {
                assert_eq!(diff[&key], (None, Some(key)));
            } else if key < 1024 {
                assert_eq!(diff.get(&key), None);
            } else {
                assert_eq!(diff[&key], (Some(key), None));
            }
        }
    }

    #[test]
    #[ignore]
    fn diff_stress() {
        enum Set {
            Identity,
            Successor,
            Empty,
        }

        const SETS: &[Set] = &[Set::Identity, Set::Successor, Set::Empty];

        let database: Database<u32, u32> = Database::new();
        let mut rng = rand::thread_rng();

        for _ in 0..512 {
            let mut lho = database.empty_table("test");
            let mut rho = database.empty_table("test2");
            let mut diff_reference = HashMap::new();

            let mut lho_transaction = TableTransaction::new();
            let mut rho_transaction = TableTransaction::new();

            for key in 0..512 {
                let lho_set = SETS.iter().choose(&mut rng).unwrap();
                let rho_set = SETS.iter().choose(&mut rng).unwrap();

                match lho_set {
                    Set::Identity => lho_transaction.set(key, key).unwrap(),
                    Set::Successor => lho_transaction.set(key, key + 1).unwrap(),
                    Set::Empty => (),
                }

                match rho_set {
                    Set::Identity => rho_transaction.set(key, key).unwrap(),
                    Set::Successor => rho_transaction.set(key, key + 1).unwrap(),
                    Set::Empty => (),
                }

                match (lho_set, rho_set) {
                    (Set::Identity, Set::Successor) => {
                        diff_reference.insert(key, (Some(key), Some(key + 1)));
                    }
                    (Set::Identity, Set::Empty) => {
                        diff_reference.insert(key, (Some(key), None));
                    }
                    (Set::Successor, Set::Identity) => {
                        diff_reference.insert(key, (Some(key + 1), Some(key)));
                    }
                    (Set::Successor, Set::Empty) => {
                        diff_reference.insert(key, (Some(key + 1), None));
                    }
                    (Set::Empty, Set::Identity) => {
                        diff_reference.insert(key, (None, Some(key)));
                    }
                    (Set::Empty, Set::Successor) => {
                        diff_reference.insert(key, (None, Some(key + 1)));
                    }
                    _ => {}
                }
            }

            lho.execute(lho_transaction);
            rho.execute(rho_transaction);

            assert_eq!(Table::diff(&mut lho, &mut rho), diff_reference);
        }
    }
}
