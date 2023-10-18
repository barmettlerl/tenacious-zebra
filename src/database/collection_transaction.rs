use crate::{
    common::store::{Field, EmptyField},
    database::{errors::QueryError, Query, TableTransaction},
};

use doomstack::Top;

pub struct CollectionTransaction<Item: Field>(pub(crate) TableTransaction<Item, EmptyField>);

impl<Item> CollectionTransaction<Item>
where
    Item: Field,
{
    pub fn new() -> Self {
        CollectionTransaction(TableTransaction::new())
    }

    pub fn contains(&mut self, item: &Item) -> Result<Query, Top<QueryError>> {
        self.0.get(item)
    }

    pub fn insert(&mut self, item: Item) -> Result<(), Top<QueryError>> {
        self.0.set(item, EmptyField)
    }

    pub fn remove(&mut self, item: &Item) -> Result<(), Top<QueryError>> {
        self.0.remove(item)
    }
}
