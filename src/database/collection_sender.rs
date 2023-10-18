use std::{sync::Arc, fmt::Display};

use crate::{
    common::store::{Field, EmptyField},
    database::{errors::SyncError, Collection, CollectionAnswer, Question, TableSender},
};

use doomstack::Top;

pub struct CollectionSender<Item: Field>(pub(crate) TableSender<Item, EmptyField>);

impl<Item> CollectionSender<Item>
where
    Item: Field + Display,
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
