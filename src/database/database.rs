use std::{sync::{RwLock, Arc}, path::Path, io::{Write, Read}, collections::HashMap};
use crate::{
    common::{store::Field},
    database::{
        store::{Cell, Store},
        Table, TableReceiver,
    },
};

use talk::sync::lenders::AtomicLender;

/// A datastrucure for memory-efficient storage and transfer of maps with a
/// large degree of similarity (% of key-pairs in common).
///
/// A database maintains a collection of [`Table`]s which in turn represent
/// a collection of key-value pairs. A [`Table`] can be read and modified by
/// creating and executing a [`Transaction`].
///
/// We optimize for the following use cases:
/// 1) Storing multiple maps with a lot of similarities (e.g. snapshots in a system)
/// 2) Transfering maps to databases with similar maps
/// 3) Applying large batches of operations (read, write, remove) to a single map
/// ([`Table`]). In particular, within a batch, we apply operations concurrently
/// and with minimal synchronization between threads.
///
/// The default hashing algorithm is currently Blake3, though this is
/// subject to change at any point in the future.
///
/// It is required that the keys implement `'static` and the [`Serialize`],
/// [`Send`] and [`Sync`] traits.
///
/// [`Field`]: crate::common::store::Field
/// [`Table`]: crate::database::Table
/// [`Transaction`]: crate::database::TableTransaction
/// [`Serialize`]: serde::Serialize
/// [`Send`]: Send
/// [`Sync`]: Sync
///
/// # Examples
///
/// ```rust
///
/// use tenaciouszebra::database::{Database, Table, TableTransaction, TableResponse, Query};
///
/// fn main() {
///     // Type inference lets us omit an explicit type signature (which
///     // would be `Database<String, integer>` in this example).
///     let database = Database::new("test");
///
///     // We create a new transaction. See [`Transaction`] for more details.
///     let mut modify = TableTransaction::new();
///     modify.set(String::from("Alice"), 42).unwrap();
///
///     let mut table = database.empty_table("test");
///     let _ = table.execute(modify);
///
///     let mut read = TableTransaction::new();
///     let query_key = read.get("Alice".to_string()).unwrap();
///     let response = table.execute(read);
///
///     assert_eq!(response.get(&query_key), Some(&42));
///
///     // Let's remove "Alice" and set "Bob".
///     let mut modify = TableTransaction::new();
///     modify.remove("Alice".to_string()).unwrap();
///     modify.set("Bob".to_string(), 23).unwrap();
///
///     // Ignore the response (modify only)
///     let _ = table.execute(modify);
///
///     let mut read = TableTransaction::new();
///     let query_key_alice = read.get("Alice".to_string()).unwrap();
///     let query_key_bob = read.get("Bob".to_string()).unwrap();
///     let response = table.execute(read);
///
///     assert_eq!(response.get(&query_key_alice), None);
///     assert_eq!(response.get(&query_key_bob), Some(&23));
/// }
/// ```

pub struct Database<Key, Value>
where
    Key: Field,
    Value: Field,
{
    pub(crate) store: Cell<Key, Value>,
    pub(crate) tables: RwLock<Vec<Arc<Table<Key, Value>>>>,
    backup_path: String,
}

impl<Key, Value> Database<Key, Value>
where
    Key: Field,
    Value: Field,
{
    /// Creates an empty `Database`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tenaciouszebra::database::{Database, TableTransaction};
    /// let mut database: Database<String, i32> = Database::new("test");
    /// ```
    pub fn new(backup_path: &str) -> Self {
        let store = Cell::new(AtomicLender::new(Store::new(backup_path)));
        std::fs::create_dir_all(backup_path).unwrap();
        //check if file exists and if not create it
        if !Path::new(backup_path).join("tables").exists() {
            let _ = std::fs::File::create(Path::new(backup_path).join("tables")).unwrap();
        }
        let mut file = std::fs::File::open(Path::new(backup_path).join("tables")).unwrap();

        let mut serialized = Vec::new();
        file.read_to_end(&mut serialized).unwrap();

        // check if serialized is empty
        if !serialized.is_empty() {
            let tables = bincode::deserialize::<Vec<String>>(&serialized).unwrap()
            .iter().map(|f|(f.to_string(), Arc::new(Table::empty(store.clone(), f.to_string())))).collect::<HashMap<String, Arc<Table<Key, Value>>>>();
            
            let (new_store, table_transactions) = store.take().restore_backup(&tables);
            store.restore(new_store);

            for (_, (table, transaction)) in table_transactions {
                table.execute(transaction);
            }

            Database {
                store: store.clone(),
                tables: RwLock::new(tables.values().cloned().collect()),
                backup_path: backup_path.to_string(),
            }
        } else {
            Database {
                store: store.clone(),
                tables: RwLock::new(Vec::new()),
                backup_path: backup_path.to_string(),
            }
        }


    }

    /// Adds a [`Table`] to the `Database` and store it on the disk.
    pub(crate) fn add_table(&self, table: Arc<Table<Key, Value>>) {
        let table_exists = self.tables.read().unwrap().iter().any(|f|f.get_name() == table.get_name());
        if table_exists {
            return;
        }

        self.tables.write().unwrap().push(table);

        let tables = self.tables.read()
        .unwrap()
        .iter()
        .map(|f|f.get_name())
        .collect::<Vec<String>>();

        let serialized = bincode::serialize(&tables).unwrap();
        let mut file: std::fs::File = std::fs::File::create(Path::new(&self.backup_path).join("tables")).unwrap();
        file.write_all(&serialized).unwrap();
    }

    pub fn get_table(&self, name: &str) -> Option<Arc<Table<Key, Value>>> {
        self.tables.read().unwrap().iter().find(|e| e.get_name() == name).cloned()
    }

    /// Creates and assigns an empty [`Table`] to the `Database`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tenaciouszebra::database::{Database, TableTransaction};
    /// let mut database: Database<String, i32> = Database::new("test");
    ///
    /// let table = database.empty_table("test");
    /// ```
    pub fn empty_table(&self, name: &str) -> Arc<Table<Key, Value>> {
        let table = Arc::new(Table::empty(self.store.clone(), name.to_string()));
        self.add_table(table.clone());
        table
    }

    /// Creates a [`TableReceiver`] assigned to this `Database`. The
    /// receiver is used to efficiently receive a [`Table`]
    /// from other databases and add them this one.
    ///
    /// See [`TableReceiver`] for more details on its operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tenaciouszebra::database::{Database, TableTransaction};
    /// let mut database: Database<String, i32> = Database::new("test");
    ///
    /// let mut receiver = database.receive();
    ///
    /// // Do things with receiver...
    ///
    /// ```
    pub fn receive(&self) -> TableReceiver<Key, Value> {
        TableReceiver::new(self.store.clone())
    }
}

impl<Key, Value> Default for Database<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn default() -> Self {
        Self::new("backup")
    }
}

impl<Key, Value> Clone for Database<Key, Value>
where
    Key: Field,
    Value: Field,
{
    fn clone(&self) -> Self {
        Database {
            store: self.store.clone(),
            tables: RwLock::new(self.tables.read().unwrap().clone()),
            backup_path: self.backup_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    use crate::{database::{store::Label, TableTransaction}, common::data};

    impl<Key, Value> Database<Key, Value>
    where
        Key: Field + Serialize,
        Value: Field + Serialize,
    {
        pub(crate) fn table_with_records<I>(&self, records: I) -> Arc<Table<Key, Value>>
        where
            I: IntoIterator<Item = (Key, Value)>,
        {
            let table = self.empty_table("test");
            let mut transaction = TableTransaction::new();

            for (key, value) in records {
                transaction.set(key, value).unwrap();
            }

            table.execute(transaction);
            table
        }

        pub(crate) fn check_correctness<'a, I, J>(&self, tables: I, receivers: J)
        where
            I: IntoIterator<Item = &'a Table<Key, Value>>,
            J: IntoIterator<Item = &'a TableReceiver<Key, Value>>,
            Key: Field,
            Value: Field,
        {
            let tables: Vec<&'a Table<Key, Value>> = tables.into_iter().collect();

            let receivers: Vec<&'a TableReceiver<Key, Value>> = receivers.into_iter().collect();

            for table in &tables {
                table.check();
            }

            let table_held = tables.iter().map(|table| table.root());

            let receiver_held = receivers.iter().flat_map(|receiver| receiver.held());

            let held: Vec<Label> = table_held.chain(receiver_held).collect();

            let mut store = self.store.take();
            store.check_leaks(held.clone());
            store.check_references(held.clone());
            self.store.restore(store);
        }

        pub(crate) fn test_database(test_function: fn(Database<Key, Value>) -> ()) 
        where 
            Key: Field,
            Value: Field,
        {
            let path = format!("test/{}", rand::random::<u64>());
            let database: Database<Key, Value> = Database::new(&path);
            test_function(database);
            std::fs::remove_dir_all(path).unwrap();
        }
    }

    #[test]
    fn test_if_table_is_correct_after_execution_of_operations() {
        Database::test_database(|database| {
            let table = database.table_with_records((0..4).map(|i| (i, i)));

            let mut transaction = TableTransaction::new();
            for i in 2..4 {
                transaction.set(i, i + 1).unwrap();
            }
            let _ = table.execute(transaction);
            table.assert_records((0..4).map(|i| (i, if i < 2 { i } else { i + 1 })));    
            database.check_correctness([table.as_ref()], []);
        });
    }

    #[test]
    fn test_if_changes_are_seen_when_clone_of_table_is_modified() {
        Database::test_database(|database| {
            let table = database.table_with_records((0..256).map(|i| (i, i)));
            let table_clone = table.clone();
    
            let mut transaction = TableTransaction::new();
            for i in 128..256 {
                transaction.set(i, i + 1).unwrap();
            }
            let _response = table.execute(transaction);
            table_clone.assert_records((0..256).map(|i| (i, if i < 128 { i } else { i + 1 })));
            table.assert_records((0..256).map(|i| (i, if i < 128 { i } else { i + 1 })));
        });
    }

    #[test]
    fn test_if_len_is_correct_when_database_contains_zero_elements() {
        Database::<u32, u32>::test_database(|database| {
            let tables = database.tables.read().unwrap();
            assert_eq!(tables.len(), 0)
        });
    }

    #[test]
    fn test_if_len_is_correct_when_database_contains_one_element() {
        Database::test_database(|database: Database<u32, u32>| {
            database.empty_table("test");
            let tables = database.tables.read().unwrap();

            assert_eq!(tables.len(), 1)
        });
    }

    #[test]
    fn test_if_database_sees_changes_made_on_table() {
        Database::test_database(|database| {
            let table = database.table_with_records((0..256).map(|i| (i, i)));

            {
                let tables = database.tables.read().unwrap();
                assert_eq!(tables[0].root(), table.root())
            }
    
            let mut transaction = TableTransaction::new();
            for i in 128..256 {
                transaction.set(i, i + 1).unwrap();
            }
    
            table.execute(transaction);
    
            {
                let tables = database.tables.read().unwrap();
                assert_eq!(tables[0].root(), table.root())
            }
        })
    }

    #[test]
    fn test_if_same_key_values_stored_in_multiple_table_are_restored_correctly() {
        let path: String = format!("test/{}", rand::random::<u64>());

        {
            let database1: Database<u32, u32> = Database::new(&path);

            let table1 = database1.empty_table("test1");
            let table2 = database1.empty_table("test2");
    
            let mut transaction = TableTransaction::new();
            for i in 0..256 {
                transaction.set(i, i + 1).unwrap();
            }
    
            table1.execute(transaction);
    
            let mut transaction = TableTransaction::new();
    
            for i in 0..128 {
                transaction.set(i, i + 1).unwrap();
            }
    
            table2.execute(transaction);
        }
        
        {
            let database2: Database<u32, u32> = Database::new(&path);
    
            let table1 = database2.get_table("test1").unwrap();
    
            let table2 = database2.get_table("test2").unwrap();
                    
            table1.assert_records((0..256).map(|i| (i, i + 1)));
            table2.assert_records((0..128).map(|i| (i, i + 1)));
        }

        // delete 
        std::fs::remove_dir_all(path).unwrap();

    }

    #[test]
    fn test_if_database_keys_are_not_visible_after_they_are_deleted() {
        let path: String = format!("test/{}", rand::random::<u64>());

        {
            let database1: Database<u32, u32> = Database::new(&path);

            let table1 = database1.empty_table("test1");
    
            let mut transaction = TableTransaction::new();
            for i in 0..4 {
                transaction.set(i, i + 1).unwrap();
            }
    
            table1.execute(transaction);

            let mut transaction = TableTransaction::new();
            transaction.remove(0).unwrap();
            transaction.remove(1).unwrap();

            table1.execute(transaction);
        }
        
        {
            let database2: Database<u32, u32> = Database::new(&path);
    
            let table1 = database2.get_table("test1").unwrap();
    
            assert_eq!(table1.collect_records().len(), 2);  
        }

        // delete 
        std::fs::remove_dir_all(path).unwrap();

    }
}
