//! Persistence layer for identities (the `identities` table).

use rusqlite::{params, Connection};

use crate::services::database::DatabaseError;

/// An identity row from the database.
#[derive(Debug, Clone)]
pub struct IdentityRow {
    pub id: i64,
    pub account_id: String,
    pub email_address: String,
    pub display_name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_encryption: String,
    pub smtp_username: String,
    pub smtp_realm: String,
    pub use_ip_in_ehlo: bool,
    pub custom_ehlo: Option<String>,
    pub login_before_send: bool,
    pub max_message_size_cache: Option<u64>,
}

/// Insert a new identity row. Returns the newly created row ID.
pub fn insert_identity(conn: &Connection, row: &IdentityRow) -> Result<i64, DatabaseError> {
    conn.execute(
        "INSERT INTO identities (
            account_id, email_address, display_name,
            smtp_host, smtp_port, smtp_encryption, smtp_username, smtp_realm,
            use_ip_in_ehlo, custom_ehlo, login_before_send, max_message_size_cache
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            row.account_id,
            row.email_address,
            row.display_name,
            row.smtp_host,
            row.smtp_port as i64,
            row.smtp_encryption,
            row.smtp_username,
            row.smtp_realm,
            row.use_ip_in_ehlo as i64,
            row.custom_ehlo,
            row.login_before_send as i64,
            row.max_message_size_cache.map(|v| v as i64),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Update the cached max message size for an identity.
pub fn update_max_message_size(
    conn: &Connection,
    identity_id: i64,
    max_message_size: Option<u64>,
) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE identities SET max_message_size_cache = ?1 WHERE id = ?2",
        params![max_message_size.map(|v| v as i64), identity_id],
    )?;
    Ok(())
}

/// Load a single identity by its row id.
pub fn load_identity_by_id(
    conn: &Connection,
    identity_id: i64,
) -> Result<Option<IdentityRow>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, email_address, display_name,
                smtp_host, smtp_port, smtp_encryption, smtp_username, smtp_realm,
                use_ip_in_ehlo, custom_ehlo, login_before_send, max_message_size_cache
         FROM identities WHERE id = ?1",
    )?;

    let result = stmt.query_row(params![identity_id], |row| {
        Ok(IdentityRow {
            id: row.get(0)?,
            account_id: row.get(1)?,
            email_address: row.get(2)?,
            display_name: row.get(3)?,
            smtp_host: row.get(4)?,
            smtp_port: row.get::<_, i64>(5)? as u16,
            smtp_encryption: row.get(6)?,
            smtp_username: row.get(7)?,
            smtp_realm: row.get(8)?,
            use_ip_in_ehlo: row.get::<_, i64>(9)? != 0,
            custom_ehlo: row.get(10)?,
            login_before_send: row.get::<_, i64>(11)? != 0,
            max_message_size_cache: row.get::<_, Option<i64>>(12)?.map(|v| v as u64),
        })
    });

    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Load all identities for an account.
pub fn load_identities_for_account(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<IdentityRow>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, email_address, display_name,
                smtp_host, smtp_port, smtp_encryption, smtp_username, smtp_realm,
                use_ip_in_ehlo, custom_ehlo, login_before_send, max_message_size_cache
         FROM identities WHERE account_id = ?1",
    )?;

    let rows = stmt
        .query_map(params![account_id], |row| {
            Ok(IdentityRow {
                id: row.get(0)?,
                account_id: row.get(1)?,
                email_address: row.get(2)?,
                display_name: row.get(3)?,
                smtp_host: row.get(4)?,
                smtp_port: row.get::<_, i64>(5)? as u16,
                smtp_encryption: row.get(6)?,
                smtp_username: row.get(7)?,
                smtp_realm: row.get(8)?,
                use_ip_in_ehlo: row.get::<_, i64>(9)? != 0,
                custom_ehlo: row.get(10)?,
                login_before_send: row.get::<_, i64>(11)? != 0,
                max_message_size_cache: row.get::<_, Option<i64>>(12)?.map(|v| v as u64),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::open_and_migrate;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        // Insert a test account for FK
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acc1', 'Test', 'imap', 'imap.test.com', 993, 'SslTls', 'password', 'user', '')",
            [],
        ).unwrap();
        (dir, conn)
    }

    #[test]
    fn insert_and_load_identity() {
        let (_dir, conn) = setup_db();

        let row = IdentityRow {
            id: 0,
            account_id: "acc1".to_string(),
            email_address: "user@test.com".to_string(),
            display_name: "Test User".to_string(),
            smtp_host: "smtp.test.com".to_string(),
            smtp_port: 587,
            smtp_encryption: "StartTls".to_string(),
            smtp_username: "user@test.com".to_string(),
            smtp_realm: "".to_string(),
            use_ip_in_ehlo: false,
            custom_ehlo: None,
            login_before_send: false,
            max_message_size_cache: Some(26_214_400),
        };

        let id = insert_identity(&conn, &row).unwrap();
        assert!(id > 0);

        let loaded = load_identities_for_account(&conn, "acc1").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].email_address, "user@test.com");
        assert_eq!(loaded[0].max_message_size_cache, Some(26_214_400));
    }

    #[test]
    fn update_max_message_size_persists() {
        let (_dir, conn) = setup_db();

        let row = IdentityRow {
            id: 0,
            account_id: "acc1".to_string(),
            email_address: "user@test.com".to_string(),
            display_name: "".to_string(),
            smtp_host: "smtp.test.com".to_string(),
            smtp_port: 465,
            smtp_encryption: "SslTls".to_string(),
            smtp_username: "user@test.com".to_string(),
            smtp_realm: "".to_string(),
            use_ip_in_ehlo: false,
            custom_ehlo: None,
            login_before_send: false,
            max_message_size_cache: None,
        };

        let id = insert_identity(&conn, &row).unwrap();
        update_max_message_size(&conn, id, Some(52_428_800)).unwrap();

        let loaded = load_identities_for_account(&conn, "acc1").unwrap();
        assert_eq!(loaded[0].max_message_size_cache, Some(52_428_800));
    }

    #[test]
    fn identities_cascade_on_account_delete() {
        let (_dir, conn) = setup_db();

        let row = IdentityRow {
            id: 0,
            account_id: "acc1".to_string(),
            email_address: "user@test.com".to_string(),
            display_name: "".to_string(),
            smtp_host: "smtp.test.com".to_string(),
            smtp_port: 587,
            smtp_encryption: "StartTls".to_string(),
            smtp_username: "".to_string(),
            smtp_realm: "".to_string(),
            use_ip_in_ehlo: false,
            custom_ehlo: None,
            login_before_send: false,
            max_message_size_cache: None,
        };

        insert_identity(&conn, &row).unwrap();
        conn.execute("DELETE FROM accounts WHERE id = 'acc1'", [])
            .unwrap();

        let loaded = load_identities_for_account(&conn, "acc1").unwrap();
        assert!(loaded.is_empty());
    }
}
