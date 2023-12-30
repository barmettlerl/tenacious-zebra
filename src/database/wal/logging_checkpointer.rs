use std::{io::{self, Read}, marker::PhantomData, sync::{RwLock, Arc}, collections::HashMap};

use okaywal::{LogManager, Entry, SegmentReader, EntryId, WriteAheadLog, ReadChunkResult};
use serde::de::DeserializeOwned;

use crate::{common::store::Field, database::{wal::{write_log::LogEntry, recovery_table_transaction::RecoveryTableTransaction}, store::Cell, Table, TableTransaction}, map::Set};

#[derive(Debug)]
pub (crate) struct LoggingCheckpointer<Key: Field, Value: Field> {
    store: Cell<Key, Value>,
    tables: Arc<RwLock<Vec<Table<Key, Value>>>>,
}

impl<Key, Value> LoggingCheckpointer<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn new(store: Cell<Key, Value>, tables: Arc<RwLock<Vec<Table<Key, Value>>>>) -> Self {
        LoggingCheckpointer {
            store,
            tables,
        }
    }

    fn add_tables(&mut self, table_names: Vec<String>) {
        for table_name in table_names {
            let table = Table::empty(self.store.clone(), table_name, None);
            self.tables.write().unwrap().push(table);
        }
    }

}


impl<Key, Value> LogManager for LoggingCheckpointer<Key, Value>

where 
    Key: Field + std::fmt::Debug + DeserializeOwned,
    Value: Field + std::fmt::Debug + DeserializeOwned,
{

    fn recover(&mut self, entry: &mut Entry<'_>) -> io::Result<()> {
        // This example uses read_all_chunks to load the entire entry into
        // memory for simplicity. The crate also supports reading each chunk
        // individually to minimize memory usage.


        if let Some(all_chunks) = entry.read_all_chunks()? {

            let all_chunks = all_chunks
                .into_iter()
                .map(|chunk| bincode::deserialize::<LogEntry<Key, Value>>(&chunk).unwrap());

            let mut transactions = HashMap::<String, RecoveryTableTransaction<Key, Value>>::new();

            all_chunks.for_each(|chunk| {
                match chunk {
                    LogEntry::Set(key, value, table_name) => {
                        let transaction = transactions.entry(table_name).or_insert(RecoveryTableTransaction::new());
                        transaction.set(key, value).unwrap();
                    },
                    LogEntry::Remove(key, table_name) => {
                        let transaction = transactions.entry(table_name).or_insert(RecoveryTableTransaction::new());
                        transaction.remove(key).unwrap();
                    },
                }
            });
            
            self.add_tables(transactions.keys().cloned().collect());

            for (table_name, transaction) in transactions {
                let mut tables = self.tables.write().unwrap();
                let table = tables.iter_mut().find(|table| table.get_name() == table_name).unwrap();
                table.recover(transaction);
            }
            
        } else {
            // This entry wasn't completely written. This could happen if a
            // power outage or crash occurs while writing an entry.
        }

        Ok(())
    }


    fn checkpoint_to(
        &mut self,
        last_checkpointed_id: EntryId,
        _checkpointed_entries: &mut SegmentReader,
        _wal: &WriteAheadLog,
    ) -> io::Result<()> {
        // checkpoint_to is called once enough data has been written to the
        // WriteAheadLog. After this function returns, the log will recycle the
        // file containing the entries being checkpointed.
        //
        // This function is where the entries must be persisted to the storage
        // layer the WriteAheadLog is sitting in front of. To ensure ACID
        // compliance of the combination of the WAL and the storage layer, the
        // storage layer must be fully resilliant to losing any changes made by
        // the checkpointed entries before this function returns.
        println!("LoggingCheckpointer::checkpoint_to({last_checkpointed_id:?}");
        Ok(())
    }
}
