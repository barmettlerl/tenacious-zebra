use serde::{Serialize, Deserialize};

use crate::{common::store::Field, database::store::Node};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Entry<Key: Field, Value: Field> {
    pub node: Node<Key, Value>,
    pub references: usize,
}
