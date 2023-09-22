
use crate::{
    common::store::Field,
    database::{Collection, CollectionReceiver, Database},
};

#[derive(Clone)]
pub struct Family<Item: Field>(pub(crate) Database<Item, ()>);

impl<'de, Item> Family<Item>
where
    Item: Field,
{
    pub fn new() -> Self {
        Family(Database::new())
    }

    pub fn empty_collection(&self, name: &str) -> Collection<Item> {
        Collection(self.0.empty_table(name))
    }

    pub fn receive(&self) -> CollectionReceiver<Item> {
        CollectionReceiver(self.0.receive())
    }
}
