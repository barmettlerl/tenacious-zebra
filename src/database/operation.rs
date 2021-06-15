use drop::crypto::hash::HashError;

use super::action::Action;
use super::field::Field;
use super::path::Path;
use super::wrap::Wrap;

#[derive(Debug)]
pub(crate) struct Operation<Key: Field, Value: Field> {
    pub path: Path,
    pub key: Wrap<Key>,
    pub action: Action<Value>,
}

impl<Key, Value> Operation<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn set(key: Key, value: Value) -> Result<Self, HashError> {
        let key = Wrap::new(key)?;
        let value = Wrap::new(value)?;

        Ok(Operation {
            path: Path::from(*key.digest()),
            key,
            action: Action::Set(value),
        })
    }

    pub fn remove(key: Key) -> Result<Self, HashError> {
        let key = Wrap::new(key)?;
        Ok(Operation {
            path: Path::from(*key.digest()),
            key,
            action: Action::Remove,
        })
    }
}

impl<Key, Value> PartialEq for Operation<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn eq(&self, rho: &Self) -> bool {
        (self.key == rho.key) && (self.action == rho.action) // `path` is uniquely determined by `key`
    }
}

impl<Key, Value> Eq for Operation<Key, Value>
where
    Key: Field,
    Value: Field,
{
}
