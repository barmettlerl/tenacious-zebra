use std::slice;

use crate::{
    common::store::Field,
    database::{
        interact::{Action, Batch},
        Query, Tid,
    },
};

use super::interact::Operation;

#[derive(Debug)]
pub struct TableResponse<Key: Field, Value: Field> {
    tid: Tid,
    batch: Batch<Key, Value>,
}

impl<Key, Value> TableResponse<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub(crate) fn new(tid: Tid, batch: Batch<Key, Value>) -> Self {
        TableResponse { tid, batch }
    }

    pub fn get(&self, query: &Query) -> Option<&Value> {
        assert_eq!(
            query.tid, self.tid,
            "called `Response::get` with a foreign `Query`"
        );

        let index = self
            .batch
            .operations()
            .binary_search_by_key(&query.path, |operation| operation.path)
            .unwrap();
        match &self.batch.operations()[index].action {
            Action::Get(Some(holder)) => Some(holder),
            Action::Get(None) => None,
            _ => unreachable!(),
        }
    }
}


impl<'a, Key, Value> IntoIterator for &'a TableResponse<Key, Value> 
where 
    Key: Field,
    Value: Field,
{
    type Item = &'a Operation<Key, Value>;
    type IntoIter = slice::Iter<'a, Operation<Key, Value>>;

    fn into_iter(self) -> slice::Iter<'a, Operation<Key, Value>> {
        self.batch.operations().iter()
    }
}

impl<'a, Key, Value> IntoIterator for &'a mut TableResponse<Key, Value> 
where 
    Key: Field,
    Value: Field,
{
    type Item = &'a mut Operation<Key, Value>;
    type IntoIter = slice::IterMut<'a, Operation<Key, Value>>;

    fn into_iter(self) -> slice::IterMut<'a, Operation<Key, Value>> {
        self.batch.operations_mut().iter_mut()
    }
}