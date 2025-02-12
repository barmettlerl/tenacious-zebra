use crate::{common::store::Field, database::store::Wrap};

use std::sync::Arc;

#[derive(Debug, Eq)]
pub(crate) enum Action<Key: Field, Value: Field> {
    Get(Option<Arc<Value>>),
    Set(Wrap<Key>, Wrap<Value>),
    Remove(Wrap<Key>),
}

impl<Key, Value> PartialEq for Action<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn eq(&self, rho: &Self) -> bool {
        match (self, rho) {
            (Action::Get(..), Action::Get(..)) => true,
            (Action::Set(self_key, self_value), Action::Set(rho_key, rho_value)) => {
                self_key == rho_key && self_value == rho_value
            }
            (Action::Remove(self_key), Action::Remove(other_key)) => self_key == other_key,
            _ => false,
        }
    }
}

