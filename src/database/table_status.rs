use std::fmt::Display;

use crate::{
    common::store::Field,
    database::{Question, Table, TableReceiver},
};

pub enum TableStatus<Key: Field + Display, Value: Field + Display> {
    Complete(Table<Key, Value>),
    Incomplete(TableReceiver<Key, Value>, Question),
}
