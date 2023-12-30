use std::sync::Arc;

use crate::{
    common::store::Field,
    database::{errors::SyncError, Collection, CollectionAnswer, Question, TableSender},
};

use doomstack::Top;
use serde::de::DeserializeOwned;

pub struct CollectionSender<Item: Field + DeserializeOwned>(pub(crate) TableSender<Item, ()>);

impl<Item> CollectionSender<Item>
where
    Item: Field + DeserializeOwned,
{
    pub fn hello(&self) -> CollectionAnswer<Item> {
        self.0.hello()
    }

    pub fn answer(
        &self,
        question: &Question,
    ) -> Result<CollectionAnswer<Item>, Top<SyncError>> {
        self.0.answer(question)
    }

    pub fn end(self, name: String) -> Collection<Item> {
        Collection(Arc::new(self.0.end(name)))
    }
}
