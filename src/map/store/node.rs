use crate::{
    common::{
        data::Bytes,
        store::{hash, Field},
    },
    map::store::Wrap,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{DeserializeOwned, Visitor, MapAccess, self}};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum Node<Key: Field, Value: Field> {
    Empty,
    #[serde(bound = "")]
    Internal(Internal<Key, Value>),
    #[serde(bound = "")]
    Leaf(Leaf<Key, Value>),
    Stub(Stub),
}

#[derive(Clone)]
pub(crate) struct Internal<Key: Field, Value: Field> {
    hash: Bytes,
    children: Children<Key, Value>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Children<Key, Value>
where
    Key: Field,
    Value: Field,
{
    #[serde(bound = "")]
    left: Box<Node<Key, Value>>,
    #[serde(bound = "")]
    right: Box<Node<Key, Value>>,
}


#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Leaf<Key: Field, Value: Field> {
    hash: Bytes,
    #[serde(bound = "")]
    fields: Fields<Key, Value>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Fields<Key: Field, Value: Field> {
    #[serde(bound = "")]
    key: Wrap<Key>,
    #[serde(bound = "")]
    value: Wrap<Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Stub {
    hash: Bytes,
}

impl<Key, Value> Node<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn internal(left: Node<Key, Value>, right: Node<Key, Value>) -> Self {
        Node::Internal(Internal::new(left, right))
    }

    pub fn leaf(key: Wrap<Key>, value: Wrap<Value>) -> Self {
        Node::Leaf(Leaf::new(key, value))
    }

    pub fn stub(hash: Bytes) -> Self {
        Node::Stub(Stub::new(hash))
    }

    pub fn hash(&self) -> Bytes {
        match self {
            Node::Empty => hash::empty(),
            Node::Internal(internal) => internal.hash(),
            Node::Leaf(leaf) => leaf.hash(),
            Node::Stub(stub) => stub.hash(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }

    pub fn is_internal(&self) -> bool {
        matches!(self, Node::Internal(_))
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    pub fn is_stub(&self) -> bool {
        matches!(self, Node::Stub(_))
    }
}

impl<Key, Value> Internal<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn new(left: Node<Key, Value>, right: Node<Key, Value>) -> Self {
        Internal::from_children(Children {
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn from_children(children: Children<Key, Value>) -> Self {
        let hash = hash::internal(children.left.hash(), children.right.hash());
        Internal { hash, children }
    }

    pub(crate) fn raw(hash: Bytes, left: Node<Key, Value>, right: Node<Key, Value>) -> Self {
        Internal {
            hash,
            children: Children {
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }

    pub fn hash(&self) -> Bytes {
        self.hash
    }

    pub fn children(self) -> (Node<Key, Value>, Node<Key, Value>) {
        (*self.children.left, *self.children.right)
    }

    pub fn left(&self) -> &Node<Key, Value> {
        &*self.children.left
    }

    pub fn left_mut(&mut self) -> &mut Node<Key, Value> {
        &mut *self.children.left
    }

    pub fn right(&self) -> &Node<Key, Value> {
        &*self.children.right
    }

    pub fn right_mut(&mut self) -> &mut Node<Key, Value> {
        &mut *self.children.right
    }
}

impl<Key, Value> Leaf<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn new(key: Wrap<Key>, value: Wrap<Value>) -> Self {
        Leaf::from_fields(Fields { key, value })
    }

    fn from_fields(fields: Fields<Key, Value>) -> Self {
        let hash = hash::leaf(fields.key.digest(), fields.value.digest());
        Leaf { hash, fields }
    }

    pub(crate) fn raw(hash: Bytes, key: Wrap<Key>, value: Wrap<Value>) -> Self {
        Leaf {
            hash,
            fields: Fields { key, value },
        }
    }

    pub fn hash(&self) -> Bytes {
        self.hash
    }

    pub fn fields(self) -> (Wrap<Key>, Wrap<Value>) {
        (self.fields.key, self.fields.value)
    }

    pub fn key(&self) -> &Wrap<Key> {
        &self.fields.key
    }

    pub fn value(&self) -> &Wrap<Value> {
        &self.fields.value
    }
}

impl Stub {
    pub fn new(hash: Bytes) -> Self {
        Stub { hash }
    }

    pub fn hash(&self) -> Bytes {
        self.hash
    }
}

impl<Key, Value> Serialize for Internal<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.children.serialize(serializer)
    }
}

impl<'de, Key, Value> Deserialize<'de> for Internal<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let children = Children::deserialize(deserializer)?;
        Ok(Internal::from_children(children))
    }
}



