use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::{common::store::Field, database::store::Node};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Entry<Key: Field, Value: Field> {

    #[serde(bound(deserialize = "Node<Key, Value>: Deserialize<'de>"))]
    pub node: Node<Key, Value>,
    pub references: i32,
}

impl<Key, Value> Display for Entry<Key, Value>
where
    Key: Field + Display,
    Value: Field + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.node)?;
        writeln!(f, "references: {}", self.references)?;
        Ok(())
    }
}

