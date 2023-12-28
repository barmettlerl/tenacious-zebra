use std::io;

use okaywal::{LogManager, Entry, SegmentReader, EntryId, WriteAheadLog};

#[derive(Debug)]
pub (crate) struct LoggingCheckpointer;

impl LogManager for LoggingCheckpointer {
    fn recover(&mut self, entry: &mut Entry<'_>) -> io::Result<()> {
        // This example uses read_all_chunks to load the entire entry into
        // memory for simplicity. The crate also supports reading each chunk
        // individually to minimize memory usage.
        if let Some(all_chunks) = entry.read_all_chunks()? {
            // Convert the Vec<u8>'s to Strings.
            let all_chunks = all_chunks
                .into_iter()
                .map(String::from_utf8)
                .collect::<Result<Vec<String>, _>>()
                .expect("invalid utf-8");
            println!(
                "LoggingCheckpointer::recover(entry_id: {:?}, data: {:?})",
                entry.id(),
                all_chunks,
            );
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
