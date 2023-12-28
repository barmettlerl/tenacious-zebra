use std::{io::{self, Read}, marker::PhantomData};

use okaywal::{LogManager, Entry, SegmentReader, EntryId, WriteAheadLog, ReadChunkResult};
use serde::de::DeserializeOwned;

use crate::{common::store::Field, database::wal::write_log::LogEntry};

#[derive(Debug)]
pub (crate) struct LoggingCheckpointer<Key: Field, Value: Field> {
    _marker: PhantomData<(Key, Value)>,
}

impl<Key, Value> LoggingCheckpointer<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub fn new() -> Self {
        LoggingCheckpointer {
            _marker: PhantomData,
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
            // Convert the Vec<u8>'s to Strings.
            let all_chunks = all_chunks
                .into_iter()
                .map(|chunk| bincode::deserialize::<LogEntry<Key, Value>>(&chunk).unwrap());
            
            println!(" len chunks: {:?}", all_chunks.count());
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
