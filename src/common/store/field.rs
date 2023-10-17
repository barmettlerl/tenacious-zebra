use serde::{Serialize, de::DeserializeOwned};

pub trait Field: 'static + Serialize + Send + DeserializeOwned + Sync {}

impl<T> Field for T where T: 'static + Serialize + Send + DeserializeOwned + Sync {}
