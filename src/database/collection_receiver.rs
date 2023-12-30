use std::sync::Arc;

use crate::{
    common::store::Field,
    database::{
        errors::SyncError, Collection, CollectionAnswer, CollectionStatus, TableReceiver,
        TableStatus,
    },
};

use doomstack::Top;
use serde::de::DeserializeOwned;

pub struct CollectionReceiver<Item: Field + DeserializeOwned>(pub(crate) TableReceiver<Item, ()>);

impl<Item> CollectionReceiver<Item>
where
    Item: Field + DeserializeOwned,
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
