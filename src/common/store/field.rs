use serde::{Serialize, Deserialize};

pub trait Field: 'static + Serialize + for<'de> Deserialize<'de> + Send + Sync {}

impl<T> Field for T where for<'de> T: 'static + Serialize + Deserialize<'de>  + Send + Sync {}
