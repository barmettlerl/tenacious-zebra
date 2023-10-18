use serde::{Serialize, de::DeserializeOwned, Deserialize};

use std::fmt::{Debug, Formatter, Display, Error};

pub trait Field: 'static + Serialize + Send + DeserializeOwned + Debug + Sync {} 

impl<T> Field for T where T: 'static + Serialize + Send + DeserializeOwned + Debug + Sync {}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct EmptyField;

impl Display for EmptyField {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "EmptyField")
    }
}