use crate::{
    common::{store::Field, tree::Path},
    database::{
        errors::QueryError,
        interact::{Batch, Operation},
        Query, store::Wrap,
    },
};

use doomstack::{here, Doom, ResultExt, Top};

use std::{
    collections::HashSet,
    sync::atomic::{AtomicUsize, Ordering},
    vec::Vec,
};

pub(crate) type Tid = usize;

static TID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct RecoveryTableTransaction<Key: Field, Value: Field> {
    tid: Tid,
    operations: Vec<Operation<Key, Value>>,
    paths: HashSet<Path>,
}

impl<Key, Value> RecoveryTableTransaction<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub (crate) fn new() -> Self {
        RecoveryTableTransaction {
            tid: TID.fetch_add(1, Ordering::Relaxed),
            operations: Vec::new(),
            paths: HashSet::new(),
        }
    }


    pub (crate) fn set(&mut self, key: Wrap<Key>, value: Wrap<Value>) -> Result<(), Top<QueryError>> {
        let operation = Operation::recovery_set(key, value).pot(QueryError::HashError, here!())?;

        if self.paths.insert(operation.path) {
            self.operations.push(operation);
            Ok(())
        } else {
            QueryError::KeyCollision.fail().spot(here!())
        }
    }

    pub (crate) fn remove(&mut self, key: Wrap<Key>) -> Result<(), Top<QueryError>> {
        let operation = Operation::recovery_remove(key).pot(QueryError::HashError, here!())?;

        if self.paths.insert(operation.path) {
            self.operations.push(operation);
            Ok(())
        } else {
            QueryError::KeyCollision.fail().spot(here!())
        }
    }

    pub(crate) fn finalize(self) -> (Tid, Batch<Key, Value>) {
        (self.tid, Batch::new(self.operations))
    }
}
