use crate::{database::TableAnswer, common::store::EmptyField};

pub type CollectionAnswer<Item> = TableAnswer<Item, EmptyField>;
