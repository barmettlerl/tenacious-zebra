
use std::fmt::Debug;

use serde::de::DeserializeOwned;

use crate::{
    common::store::Field,
    database::{Collection, CollectionReceiver, Database},
};

#[derive(Clone)]
pub struct Family<Item: Field>(pub(crate) Database<Item, ()>);

impl<'de, Item> Family<Item>
where
    Item: Field + DeserializeOwned + Debug,
{
    pub fn new(log_path: &str) -> Self {
        Family(Database::new(log_path))
    }

    pub fn empty_collection(&self, name: &str) -> Collection<Item> {
        Collection(self.0.empty_table(name))
    }

    pub fn receive(&self) -> CollectionReceiver<Item> {
        CollectionReceiver(self.0.receive())
    }
}
