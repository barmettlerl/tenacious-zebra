use std::{marker::PhantomData, fmt};

use crate::{
    common::{
        data::Bytes,
        store::{hash, Field},
    },
    map::store::Wrap,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{DeserializeOwned, Visitor, MapAccess, self}};

#[derive(Clone, Serialize)]
pub(crate) enum Node<Key: Field, Value: Field> {
    Empty,
    Internal(Internal<Key, Value>),
    Leaf(Leaf<Key, Value>),
    Stub(Stub),
}

#[derive(Clone)]
pub(crate) struct Internal<Key: Field, Value: Field> {
    hash: Bytes,
    children: Children<Key, Value>,
}

#[derive(Clone, Serialize)]
struct Children<Key, Value>
where
    Key: Field,
    Value: Field,
{
    left: Box<Node<Key, Value>>,
    right: Box<Node<Key, Value>>,
}

impl<'de, Key, Value> Deserialize<'de> for Children<Key, Value>
where
    Key: Field + DeserializeOwned,
    Value: Field + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum InternalField {
            Left,
            Right,
        }

        struct __Visitor<'de, Key, Value>(PhantomData<(&'de Key, &'de Value)>);

        impl<'de, Key, Value> Visitor<'de> for __Visitor<'de, Key, Value>
        where
            Key: Field + DeserializeOwned,
            Value: Field + DeserializeOwned,
        {
            type Value = Children<Key, Value>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Children")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut left = None;
                let mut right = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        InternalField::Left => {
                            if left.is_some() {
                                return Err(de::Error::duplicate_field("left"));
                            }
                            left = Some(map.next_value()?);
                        }
                        InternalField::Right => {
                            if right.is_some() {
                                return Err(de::Error::duplicate_field("right"));
                            }
                            right = Some(map.next_value()?);
                        }
                    }
                }
                let left = left.ok_or_else(|| de::Error::missing_field("left"))?;
                let right = right.ok_or_else(|| de::Error::missing_field("right"))?;
                Ok(Children { left, right })
            }
        }

        deserializer.deserialize_map(__Visitor::<Key, Value>(PhantomData))
    }
}


#[derive(Clone)]
pub(crate) struct Leaf<Key: Field, Value: Field> {
    hash: Bytes,
    fields: Fields<Key, Value>,
}

#[derive(Clone, Serialize)]
struct Fields<Key: Field, Value: Field> {
    key: Wrap<Key>,
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
        match self {
            Node::Empty => true,
            _ => false,
        }
    }

    pub fn is_internal(&self) -> bool {
        match self {
            Node::Internal(_) => true,
            _ => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }

    pub fn is_stub(&self) -> bool {
        match self {
            Node::Stub(_) => true,
            _ => false,
        }
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
            hash: hash,
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

impl<'de, Key, Value> Deserialize<'de> for Node<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Node::deserialize(deserializer)
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

impl<Key, Value> Serialize for Leaf<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.fields.serialize(serializer)
    }
}

