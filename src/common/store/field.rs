use serde::{Serialize};

pub trait Field: 'static + Serialize + Send + std::fmt::Debug + Sync {}

impl<T> Field for T where T: 'static + Serialize + Send + std::fmt::Debug + Sync {}
