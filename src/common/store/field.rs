use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub trait Field: 'static + Serialize + Send + DeserializeOwned + Debug + Sync {}

impl<T> Field for T where T: 'static + Serialize + Send + DeserializeOwned + Debug + Sync {}
