use serde::{Serialize};
use std::fmt::Debug;

pub trait Field: 'static + Serialize + Send + Debug + Sync {} 

impl<T> Field for T where T: 'static + Serialize + Send + Debug + Sync {}
