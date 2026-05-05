use std::cell::RefCell;
use std::path::PathBuf;

use rusqlite::Connection;
use uuid::Uuid;

use crate::services::database;
use crate::services::order_store::OrderStoreError;

/// SQLite-backed order store. Drop-in replacement for the JSON-backed
/// `OrderStore` with the same public method signatures.
#[derive(Debug)]
pub struct SqliteOrderStore {
    conn: RefCell<Connection>,
}

impl SqliteOrderStore {
    /// Open the database at `db_path`, run migrations, and return a ready store.
    pub fn new(db_path: PathBuf) -> Result<Self, OrderStoreError> {
        let conn = database::open_and_migrate(&db_path).map_err(|e| match e {
            database::DatabaseError::Sqlite(e) => {
                OrderStoreError::Io(std::io::Error::other(e.to_string()))
            }
            database::DatabaseError::Io(e) => OrderStoreError::Io(e),
        })?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }

    /// Load the persisted account order. Returns `None` if no order has been set.
    pub fn load(&self) -> Result<Option<Vec<Uuid>>, OrderStoreError> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare("SELECT account_id FROM account_order ORDER BY position ASC")
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;

        let rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut ids = Vec::with_capacity(rows.len());
        for s in &rows {
            let id = Uuid::parse_str(s).map_err(|e| {
                OrderStoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            })?;
            ids.push(id);
        }
        Ok(Some(ids))
    }

    /// Save the account order to the database.
    pub fn save(&self, order: &[Uuid]) -> Result<(), OrderStoreError> {
        let conn = self.conn.borrow();
        conn.execute("DELETE FROM account_order", [])
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;

        let mut stmt = conn
            .prepare("INSERT INTO account_order (position, account_id) VALUES (?1, ?2)")
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;

        for (i, id) in order.iter().enumerate() {
            stmt.execute(rusqlite::params![i as i64, id.to_string()])
                .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;
        }
        Ok(())
    }

    /// Clear the persisted order (reset to default).
    pub fn clear(&self) -> Result<(), OrderStoreError> {
        let conn = self.conn.borrow();
        conn.execute("DELETE FROM account_order", [])
            .map_err(|e| OrderStoreError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    /// Import order from a list of UUIDs. Idempotent: replaces existing order.
    pub fn import_from_json(&self, order: &[Uuid]) -> Result<(), OrderStoreError> {
        self.save(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, SqliteOrderStore) {
        let dir = TempDir::new().unwrap();
        let store = SqliteOrderStore::new(dir.path().join("fairmail.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn load_returns_none_on_empty_db() {
        let (_dir, store) = make_store();
        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_dir, store) = make_store();
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        store.save(&ids).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, ids);
    }

    #[test]
    fn save_is_idempotent() {
        let (_dir, store) = make_store();
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        store.save(&ids).unwrap();
        store.save(&ids).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, ids);
    }

    #[test]
    fn clear_removes_order() {
        let (_dir, store) = make_store();
        let ids: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        store.save(&ids).unwrap();
        store.clear().unwrap();
        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn clear_when_empty_is_ok() {
        let (_dir, store) = make_store();
        store.clear().unwrap();
    }

    #[test]
    fn import_from_json_works() {
        let (_dir, store) = make_store();
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        store.import_from_json(&ids).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, ids);
    }
}
