//! Persistence for the `folders` table.

use rusqlite::Connection;

use crate::core::account::FolderRole;
use crate::core::imap_check::ImapFolder;
use crate::services::database::DatabaseError;

/// Replace all folders for an account with a new set from the server.
pub fn replace_folders(
    conn: &Connection,
    account_id: &str,
    folders: &[ImapFolder],
) -> Result<(), DatabaseError> {
    conn.execute(
        "DELETE FROM folders WHERE account_id = ?1",
        rusqlite::params![account_id],
    )?;

    let mut stmt = conn.prepare(
        "INSERT INTO folders (account_id, name, attributes, role)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    for folder in folders {
        let role_str = folder.role.as_ref().map(|r| format!("{r}"));
        stmt.execute(rusqlite::params![
            account_id,
            folder.name,
            folder.attributes,
            role_str
        ])?;
    }

    Ok(())
}

/// Load all folders for an account.
pub fn load_folders(conn: &Connection, account_id: &str) -> Result<Vec<ImapFolder>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT name, attributes, role FROM folders WHERE account_id = ?1 ORDER BY name",
    )?;

    let rows = stmt.query_map(rusqlite::params![account_id], |row| {
        let name: String = row.get(0)?;
        let attributes: String = row.get(1)?;
        let role_str: Option<String> = row.get(2)?;
        let role = role_str.and_then(|s| parse_folder_role(&s));
        Ok(ImapFolder {
            name,
            attributes,
            role,
        })
    })?;

    let mut folders = Vec::new();
    for row in rows {
        folders.push(row?);
    }
    Ok(folders)
}

/// Insert a single folder row. Returns the new row id.
pub fn insert_folder(
    conn: &Connection,
    account_id: &str,
    name: &str,
) -> Result<i64, DatabaseError> {
    conn.execute(
        "INSERT INTO folders (account_id, name, attributes) VALUES (?1, ?2, '')",
        rusqlite::params![account_id, name],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Rename a folder row by id. Returns true if a row was updated.
pub fn rename_folder(
    conn: &Connection,
    folder_id: i64,
    new_name: &str,
) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        rusqlite::params![new_name, folder_id],
    )?;
    Ok(updated > 0)
}

/// Delete a folder row by id. Returns true if a row was deleted.
pub fn delete_folder(conn: &Connection, folder_id: i64) -> Result<bool, DatabaseError> {
    let deleted = conn.execute(
        "DELETE FROM folders WHERE id = ?1",
        rusqlite::params![folder_id],
    )?;
    Ok(deleted > 0)
}

/// Look up a folder's name by id.
pub fn folder_name_by_id(
    conn: &Connection,
    folder_id: i64,
) -> Result<Option<String>, DatabaseError> {
    let result = conn.query_row(
        "SELECT name FROM folders WHERE id = ?1",
        rusqlite::params![folder_id],
        |row| row.get(0),
    );
    match result {
        Ok(name) => Ok(Some(name)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Check whether notifications are enabled for a specific folder.
/// Returns `true` if the folder has `notifications_enabled = 1` (the default),
/// or if the folder is not found (conservative: notify by default).
pub fn is_folder_notifications_enabled(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
) -> Result<bool, DatabaseError> {
    let result = conn.query_row(
        "SELECT notifications_enabled FROM folders WHERE account_id = ?1 AND name = ?2",
        rusqlite::params![account_id, folder_name],
        |row| row.get::<_, i32>(0),
    );
    match result {
        Ok(val) => Ok(val != 0),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Update the notifications_enabled flag for a specific folder.
pub fn set_folder_notifications_enabled(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
    enabled: bool,
) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE folders SET notifications_enabled = ?1 WHERE account_id = ?2 AND name = ?3",
        rusqlite::params![enabled as i32, account_id, folder_name],
    )?;
    Ok(updated > 0)
}

/// Check whether a folder is marked as read-only.
/// Returns `false` if the folder is not found.
pub fn is_folder_read_only(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
) -> Result<bool, DatabaseError> {
    let result = conn.query_row(
        "SELECT read_only FROM folders WHERE account_id = ?1 AND name = ?2",
        rusqlite::params![account_id, folder_name],
        |row| row.get::<_, i32>(0),
    );
    match result {
        Ok(val) => Ok(val != 0),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Update the read_only flag for a specific folder.
pub fn set_folder_read_only(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
    read_only: bool,
) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE folders SET read_only = ?1 WHERE account_id = ?2 AND name = ?3",
        rusqlite::params![read_only as i32, account_id, folder_name],
    )?;
    Ok(updated > 0)
}

/// Look up the role of a folder by account and name.
pub fn folder_role_by_name(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
) -> Result<Option<FolderRole>, DatabaseError> {
    let result = conn.query_row(
        "SELECT role FROM folders WHERE account_id = ?1 AND name = ?2",
        rusqlite::params![account_id, folder_name],
        |row| row.get::<_, Option<String>>(0),
    );
    match result {
        Ok(Some(s)) => Ok(parse_folder_role(&s)),
        Ok(None) => Ok(None),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Look up the folder name that has a given role for an account.
/// Returns `None` if no folder with that role exists.
pub fn folder_name_by_role(
    conn: &Connection,
    account_id: &str,
    role: FolderRole,
) -> Result<Option<String>, DatabaseError> {
    let role_str = format!("{role}");
    let result = conn.query_row(
        "SELECT name FROM folders WHERE account_id = ?1 AND role = ?2",
        rusqlite::params![account_id, role_str],
        |row| row.get(0),
    );
    match result {
        Ok(name) => Ok(Some(name)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

fn parse_folder_role(s: &str) -> Option<FolderRole> {
    match s {
        "Drafts" => Some(FolderRole::Drafts),
        "Sent" => Some(FolderRole::Sent),
        "Archive" => Some(FolderRole::Archive),
        "Trash" => Some(FolderRole::Trash),
        "Junk" => Some(FolderRole::Junk),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::open_and_migrate;
    use tempfile::TempDir;

    fn setup_db() -> (tempfile::TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        (dir, conn)
    }

    #[test]
    fn replace_and_load_folders() {
        let (_dir, conn) = setup_db();

        let folders = vec![
            ImapFolder {
                name: "INBOX".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Sent".to_string(),
                attributes: "\\Sent".to_string(),
                role: Some(FolderRole::Sent),
            },
            ImapFolder {
                name: "Trash".to_string(),
                attributes: "\\Trash".to_string(),
                role: Some(FolderRole::Trash),
            },
        ];

        replace_folders(&conn, "acct-1", &folders).unwrap();
        let loaded = load_folders(&conn, "acct-1").unwrap();
        assert_eq!(loaded.len(), 3);
        assert!(loaded.iter().any(|f| f.name == "INBOX" && f.role.is_none()));
        assert!(loaded
            .iter()
            .any(|f| f.name == "Sent" && f.role == Some(FolderRole::Sent)));
    }

    #[test]
    fn replace_replaces_existing() {
        let (_dir, conn) = setup_db();

        let folders1 = vec![ImapFolder {
            name: "INBOX".to_string(),
            attributes: "".to_string(),
            role: None,
        }];
        replace_folders(&conn, "acct-1", &folders1).unwrap();

        let folders2 = vec![
            ImapFolder {
                name: "INBOX".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Drafts".to_string(),
                attributes: "\\Drafts".to_string(),
                role: Some(FolderRole::Drafts),
            },
        ];
        replace_folders(&conn, "acct-1", &folders2).unwrap();

        let loaded = load_folders(&conn, "acct-1").unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn load_empty_returns_empty() {
        let (_dir, conn) = setup_db();
        let loaded = load_folders(&conn, "acct-1").unwrap();
        assert!(loaded.is_empty());
    }
}
