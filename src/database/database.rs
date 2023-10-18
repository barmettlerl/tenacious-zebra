use std::{sync::{RwLock, Arc}, path::Path, io::{Write, Read}, fmt::Display};
use serde::{Serialize, de::DeserializeOwned};
use bincode;
use crate::{
    common::{store::Field},
    database::{
        store::{Cell, Store},
        Table, TableReceiver,
    },
};

use talk::sync::lenders::AtomicLender;

use super::store::Label;

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
///     let database = Database::new();
///
///     // We create a new transaction. See [`Transaction`] for more details.
///     let mut modify = TableTransaction::new();
///     modify.set(String::from("Alice"), 42).unwrap();
///
///     let mut table = database.empty_table("test");
///     let _ = table.execute(modify);
///
///     let mut read = TableTransaction::new();
///     let query_key = read.get(&"Alice".to_string()).unwrap();
///     let response = table.execute(read);
///
///     assert_eq!(response.get(&query_key), Some(&42));
///
///     // Let's remove "Alice" and set "Bob".
///     let mut modify = TableTransaction::new();
///     modify.remove(&"Alice".to_string()).unwrap();
///     modify.set("Bob".to_string(), 23).unwrap();
///
///     // Ignore the response (modify only)
///     let _ = table.execute(modify);
///
///     let mut read = TableTransaction::new();
///     let query_key_alice = read.get(&"Alice".to_string()).unwrap();
///     let query_key_bob = read.get(&"Bob".to_string()).unwrap();
///     let response = table.execute(read);
///
///     assert_eq!(response.get(&query_key_alice), None);
///     assert_eq!(response.get(&query_key_bob), Some(&23));
/// }
/// ```

pub struct Database<Key, Value>
where
    Key: Field + Display,
    Value: Field + Display,
{
    pub(crate) store: Cell<Key, Value>,
    pub(crate) tables: RwLock<Vec<Arc<Table<Key, Value>>>>,
}

impl<Key, Value> Database<Key, Value>
where
    Key: Field + Display,
    Value: Field + Display,
{
    /// Creates an empty `Database`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tenaciouszebra::database::{Database, TableTransaction};
    /// let mut database: Database<String, i32> = Database::new();
    /// ```
    pub fn new() -> Self {
        Database {
            store: Cell::new(AtomicLender::new(Store::new())),
            tables: RwLock::new(Vec::new()),
        }
    }

    pub(crate) fn from_store(store: Store<Key, Value>) -> Self {
        Database {
            store: Cell::new(AtomicLender::new(store)),
            tables: RwLock::new(Vec::new()),
        }
    }

    pub(crate) fn add_table(&self, table: Arc<Table<Key, Value>>) {
        self.tables.write().unwrap().push(table);
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
    /// let mut database: Database<String, i32> = Database::new();
    ///
    /// let table = database.empty_table("test");
    /// ```
    pub fn empty_table(&self, name: &str) -> Arc<Table<Key, Value>> {
        let table = Arc::new(Table::empty(self.store.clone(), name.to_string()));
        self.tables.write().unwrap().push(table.clone());
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
    /// let mut database: Database<String, i32> = Database::new();
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
    Key: Field + Display,
    Value: Field + Display,
{
    fn default() -> Self {
        Self::new()
    }
}

// impl<Key, Value> Database<Key, Value>
//     where
//         Key: Field + Serialize + DeserializeOwned,
//         Value: Field + Serialize + DeserializeOwned,
//     {

//     /// Creates a backup of the database in the specified folder.
//     /// 
//     /// # Examples
//     /// 
//     /// ```
//     ///
//     /// use tenaciouszebra::database::{Database, TableTransaction};
//     /// let database = Database::new();
//     /// 
//     /// let mut modify = TableTransaction::new();
//     /// modify.set("Alice".to_string(), 42).unwrap();
//     /// 
//     /// 
//     /// let table = database.empty_table("test");
//     /// let _ = table.execute(modify);
//     /// 
//     /// database.backup("./backup");
//     /// 
//     /// let new_database = Database::<String, i32>::restore("./backup");
//     /// 
//     /// let new_table = new_database.get_table("test").unwrap();
//     /// 
//     /// let mut read = TableTransaction::new();
//     /// let query_key = read.get(&"Alice".to_string()).unwrap();
//     /// let response = new_table.execute(read);
//     /// ```
//     pub fn backup(&self, folder_path: &str){
        
//         if !Path::new(folder_path).exists() {
//             std::fs::create_dir(folder_path).unwrap();
//         }

//         let mut file = std::fs::File::create(format!("{}/store", folder_path)).unwrap();
//         let store = self.store.take();
//         let store_str = bincode::serialize(&store).unwrap();
//         self.store.restore(store);

//         file.write_all(&store_str).unwrap();
        
//         let mut file = std::fs::File::create(format!("{}/tables", folder_path)).unwrap();
//         let tables = self.tables.write().unwrap();
//         let labels: Vec<(String, Label)> = tables.to_vec().iter().map(|e| {(e.get_name(), e.get_root())}).collect();
//         let tables_str = bincode::serialize(&labels).unwrap();
//         file.write_all(&tables_str).unwrap();
//     }

//     /// Restores a database from a backup.
//     /// 
//     /// # Examples
//     /// 
//     /// ```
//     ///
//     /// use tenaciouszebra::database::{Database, TableTransaction};
//     /// let database = Database::new();
//     /// 
//     /// let mut modify = TableTransaction::new();
//     /// modify.set("Alice".to_string(), 42).unwrap();
//     /// 
//     /// 
//     /// let table = database.empty_table("test");
//     /// let _ = table.execute(modify);
//     /// 
//     /// database.backup("./backup");
//     /// 
//     /// let new_database = Database::<String, i32>::restore("./backup");
//     /// 
//     /// let new_table = new_database.get_table("test").unwrap();
//     /// 
//     /// let mut read = TableTransaction::new();
//     /// let query_key = read.get(&"Alice".to_string()).unwrap();
//     /// let response = new_table.execute(read);
//     /// ```
//     pub fn restore(folder_path: &str) -> Self{
//         let mut file = std::fs::File::open(format!("{}/store", folder_path)).unwrap();
        
//         let mut store_str = Vec::<u8>::new();
//         file.read_to_end(&mut store_str).unwrap();
//         let store: Store<Key, Value> = bincode::deserialize(&store_str).unwrap();
//         let database = Database::from_store(store);

//         let mut file = std::fs::File::open(format!("{}/tables", folder_path)).unwrap();
//         let mut tables_str = Vec::<u8>::new();
//         file.read_to_end(&mut tables_str).unwrap();
//         let labels: Vec<(String, Label)> = bincode::deserialize(&tables_str).unwrap();
//         labels.iter().for_each(|e| {
//             database.add_table(Arc::new(Table::new(database.store.clone(), e.1, e.0.clone())))
//         });

//         database.tables.read().unwrap().iter().for_each(|e| e.check());

//         database
//     }
    
// }

impl<Key, Value> Clone for Database<Key, Value>
where
    Key: Field + Display,
    Value: Field + Display,
{
    fn clone(&self) -> Self {
        Database {
            store: self.store.clone(),
            tables: RwLock::new(self.tables.read().unwrap().clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::*;

    use crate::database::{store::Label, TableTransaction};

    impl<Key, Value> Database<Key, Value>
    where
        Key: Field + Serialize + Display,
        Value: Field + Serialize + Display,
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
    }

    #[test]
    fn test_if_table_is_correct_after_execution_of_operations() {
        let database: Database<u32, u32> = Database::new();

        let table = database.table_with_records((0..256).map(|i| (i, i)));

        let mut transaction = TableTransaction::new();
        for i in 128..256 {
            transaction.set(i, i + 1).unwrap();
        }
        let _ = table.execute(transaction);
        table.assert_records((0..256).map(|i| (i, if i < 128 { i } else { i + 1 })));

        database.check_correctness([table.as_ref()], []);
    }

    #[test]
    fn test_if_changes_are_seen_when_clone_of_table_is_modified() {
        let database: Database<u32, u32> = Database::new();

        let table = database.table_with_records((0..256).map(|i| (i, i)));
        let table_clone = table.clone();

        let mut transaction = TableTransaction::new();
        for i in 128..256 {
            transaction.set(i, i + 1).unwrap();
        }
        let _response = table.execute(transaction);
        table_clone.assert_records((0..256).map(|i| (i, if i < 128 { i } else { i + 1 })));
        table.assert_records((0..256).map(|i| (i, if i < 128 { i } else { i + 1 })));
    }

    #[test]
    fn test_if_len_is_correct_when_database_contains_zero_elements() {
        let database: Database<u32, u32> = Database::new();
        let tables = database.tables.read().unwrap();

        assert_eq!(tables.len(), 0)
    }

    #[test]
    fn test_if_len_is_correct_when_database_contains_one_element() {
        let database: Database<u32, u32> = Database::new();

        database.empty_table("test");

        let tables = database.tables.read().unwrap();

        assert_eq!(tables.len(), 1)
    }

    #[test]
    fn test_if_database_sees_changes_made_on_table() {

        let database: Database<u32, u32> = Database::new();

        let table = database.table_with_records((0..256).map(|i| (i, i)));

        {
            let tables = database.tables.read().unwrap();
            println!("{:?}", tables[0].root());
            println!("{:?}", table.root());

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

    }
}
