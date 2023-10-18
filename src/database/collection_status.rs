use std::fmt::Display;

use crate::{
    common::store::Field,
    database::{Collection, CollectionReceiver, Question},
};

pub enum CollectionStatus<Item: Field + Display> {
    Complete(Collection<Item>),
    Incomplete(CollectionReceiver<Item>, Question),
}
