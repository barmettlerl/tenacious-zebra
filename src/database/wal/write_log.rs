
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
    Set(Wrap<Key>, Wrap<Value>, String),
    Remove(Wrap<Key>, String),
}


pub (crate) fn write_log<Key: Field, Value: Field> (log: &WriteAheadLog, batch: &Batch<Key, Value>, table_name: String) {
    let mut writer = log.begin_entry().unwrap();

    for operation in batch.operations() {
        match operation {
            Operation{ action: Action::Set(key, value), .. } => {
                let chunk = LogEntry::Set(key.clone(), value.clone(), table_name.clone());
                let chunk_bytes = bincode::serialize(&chunk).unwrap();
                writer.write_chunk(&chunk_bytes).unwrap();
            },
            Operation{ action: Action::Remove(key), .. } => {
                writer.write_chunk(&bincode::serialize(&LogEntry::<Key,Value>::Remove(key.clone(), table_name.clone())).unwrap()).unwrap();
            },
            _ => {}
    }
    }
    writer.commit().unwrap();
} 