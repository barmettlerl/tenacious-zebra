use std::fmt::Display;

use crate::{
    common::{data::Bytes, store::hash},
    database::store::MapId,
};


use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq, Serialize, Deserialize,)]
pub(crate) enum Label {
    Internal(MapId, Bytes),
    Leaf(MapId, Bytes),
    Empty,
}

impl Label {
    pub fn is_empty(&self) -> bool {
        *self == Label::Empty
    }

    pub fn map(&self) -> &MapId {
        match self {
            Label::Internal(map, _) => map,
            Label::Leaf(map, _) => map,
            Label::Empty => {
                panic!("called `Label::map()` on an `Empty` value")
            }
        }
    }

    pub fn hash(&self) -> Bytes {
        match self {
            Label::Internal(_, hash) => *hash,
            Label::Leaf(_, hash) => *hash,
            Label::Empty => hash::empty(),
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Internal(map, hash) => {
                writeln!(f, "Internal: {} {}", map, hash::hex(hash))
            }
            Label::Leaf(map, hash) => {
                writeln!(f, "Leaf: {} {}", map, hash::hex(hash))
            }
            Label::Empty => writeln!(f, "Empty"),
        }
    }
}