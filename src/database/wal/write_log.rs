
use okaywal::WriteAheadLog;
use serde::{Serialize, Deserialize};
use crate::{
    common::{
        data::Bytes,
        store::{hash, Field},
    },
    database::store::{Label, Wrap},
};

use crate::{database::interact::{Batch, Operation, Action}, common::{tree::Path}};


#[derive(Debug, Serialize, Deserialize)]
pub (crate) enum LogEntry<Key: Field, Value: Field> {
    #[serde(bound(deserialize = "Wrap<Key>: Deserialize<'de>, Wrap<Value>: Deserialize<'de>"))]
    Set(Path, Wrap<Key>, Wrap<Value>),
    Remove(Path),
}


pub (crate) fn write_log<Key: Field, Value: Field> (log: &WriteAheadLog, batch: &Batch<Key, Value>) {
    let mut writer = log.begin_entry().unwrap();

    for operation in batch.operations() {
        match operation {
            Operation{ path, action: Action::Set(key, value) } => {
                let chunk = LogEntry::Set(*path, key.clone(), value.clone());
                let chunk_bytes = bincode::serialize(&chunk).unwrap();
                writer.write_chunk(&chunk_bytes).unwrap();
            },
            Operation{ path, action: Action::Remove } => {
                writer.write_chunk(&bincode::serialize(&LogEntry::<Key,Value>::Remove(*path)).unwrap()).unwrap();
            },
            _ => {}
    }
    }
    writer.commit().unwrap();
} 