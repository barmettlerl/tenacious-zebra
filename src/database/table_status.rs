use serde::de::DeserializeOwned;

use crate::{
    common::store::Field,
    database::{Question, Table, TableReceiver},
};

pub enum TableStatus<Key: Field + DeserializeOwned, Value: Field + DeserializeOwned> {
    Complete(Table<Key, Value>),
    Incomplete(TableReceiver<Key, Value>, Question),
}
