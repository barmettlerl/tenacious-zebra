
use std::fmt::Display;

use crate::{
    common::store::{Field, EmptyField},
    database::{Collection, CollectionReceiver, Database},
};

#[derive(Clone)]
pub struct Family<Item: Field + Display>(pub(crate) Database<Item, EmptyField>);

impl<'de, Item> Family<Item>
where
    Item: Field + Display,
{
    pub fn new(path: &str) -> Self {
        Family(Database::new(path))
    }

    pub fn empty_collection(&self, name: &str) -> Collection<Item> {
        Collection(self.0.empty_table(name))
    }

    pub fn receive(&self) -> CollectionReceiver<Item> {
        CollectionReceiver(self.0.receive())
    }
}
