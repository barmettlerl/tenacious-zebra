use std::fmt::{Display, self, Formatter};

use crate::{common::store::Field, database::interact::Operation};

#[derive(Debug)]
pub(crate) enum Task<'a, Key: Field, Value: Field> {
    Pass,
    Do(&'a mut Operation<Key, Value>),
    Split,
}

impl<'a, Key, Value> PartialEq for Task<'a, Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn eq(&self, rho: &Self) -> bool {
        match (self, rho) {
            (Task::Pass, Task::Pass) => true,
            (Task::Do(self_op), Task::Do(rho_op)) => self_op == rho_op,
            (Task::Split, Task::Split) => true,
            _ => false,
        }
    }
}

impl<'a, Key, Value> Eq for Task<'a, Key, Value>
where
    Key: Field,
    Value: Field,
{
}

impl<'a, Key, Value> Display for Task<'a, Key, Value>
where
    Key: Field + Display,
    Value: Field + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Task::Pass => write!(f, "Pass"),
            Task::Do(op) => write!(f, "Do({})", op),
            Task::Split => write!(f, "Split"),
        }
    }
}