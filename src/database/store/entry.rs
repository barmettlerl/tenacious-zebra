use serde::{Serialize, Deserialize};

use crate::{common::store::Field, database::store::Node};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Entry<Key: Field, Value: Field> {

    #[serde(bound(deserialize = "Node<Key, Value>: Deserialize<'de>"))]
    pub node: Node<Key, Value>,
    pub references: usize,
}

