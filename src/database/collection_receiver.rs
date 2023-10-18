use std::{sync::Arc, fmt::Display};

use crate::{
    common::store::{Field, EmptyField},
    database::{
        errors::SyncError, Collection, CollectionAnswer, CollectionStatus, TableReceiver,
        TableStatus,
    },
};

use doomstack::Top;

pub struct CollectionReceiver<Item: Field + Display>(pub(crate) TableReceiver<Item, EmptyField>);

impl<Item> CollectionReceiver<Item>
where
    Item: Field + Display,
{
    pub fn learn(
        self,
        answer: CollectionAnswer<Item>,
    ) -> Result<CollectionStatus<Item>, Top<SyncError>> {
        let status = self.0.learn(answer)?;

        let status = match status {
            TableStatus::Complete(table) => CollectionStatus::Complete(Collection(Arc::new(table))),
            TableStatus::Incomplete(receiver, question) => {
                CollectionStatus::Incomplete(CollectionReceiver(receiver), question)
            }
        };

        Ok(status)
    }
}
