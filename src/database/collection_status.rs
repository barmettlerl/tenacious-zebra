use serde::de::DeserializeOwned;

use crate::{
    common::store::Field,
    database::{Collection, CollectionReceiver, Question},
};

pub enum CollectionStatus<Item: Field + DeserializeOwned> {
    Complete(Collection<Item>),
    Incomplete(CollectionReceiver<Item>, Question),
}
