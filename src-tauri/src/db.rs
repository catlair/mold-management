use crate::excel;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

const SCHEMA_VERSION: &str = "1";

/// 打开（必要时创建）SQLite 数据库：WAL 模式 + busy_timeout，桌面单机足够可靠。
pub fn connect(db_path: &str) -> Result<Connection, String> {
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建数据目录失败「{}」: {}", parent.display(), e))?;
    }
    let conn =
        Connection::open(db_path).map_err(|e| format!("打开数据库失败「{}」: {}", db_path, e))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|e| format!("设置数据库等待超时失败: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    Ok(conn)
}

fn sheet_table(sheet_name: &str) -> String {
    format!("\"{}\"", sheet_name.replace('"', "\"\""))
}

fn column_name(key: &str) -> String {
    format!("\"{}\"", key.replace('"', "\"\""))
}

/// 按 excel::SHEETS 定义创建全部业务表（幂等），并记录 schema 版本。
pub fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )
    .map_err(|e| format!("创建元数据表失败: {}", e))?;
    for &(sheet_name, columns) in excel::SHEETS {
        let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (", sheet_table(sheet_name));
        let mut first = true;
        for &(_, key) in columns {
            if !first {
                sql.push_str(", ");
            }
            first = false;
            sql.push_str(&format!("{} TEXT", column_name(key)));
        }
        sql.push_str(");");
        conn.execute_batch(&sql)
            .map_err(|e| format!("创建表「{}」失败: {}", sheet_name, e))?;
        // id 列建立唯一索引，加速按 ID 查询与写入
        let has_id = columns.iter().any(|&(_, key)| key == "id");
        if has_id {
            conn.execute_batch(&format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_{} ON {} ({});",
                sanitize_index_name(sheet_name),
                sheet_table(sheet_name),
                column_name("id")
            ))
            .map_err(|e| format!("创建表「{}」索引失败: {}", sheet_name, e))?;
        }
    }
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("记录 schema 版本失败: {}", e))?;
    Ok(())
}

fn sanitize_index_name(sheet_name: &str) -> String {
    let cleaned: String = sheet_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    if cleaned.is_empty() {
        "sheet".to_string()
    } else {
        cleaned
    }
}

fn column_keys(sheet_name: &str) -> Vec<&'static str> {
    excel::SHEETS
        .iter()
        .find(|(name, _)| *name == sheet_name)
        .map(|(_, columns)| columns.iter().map(|&(_, key)| key).collect())
        .unwrap_or_default()
}

/// 读取某表全部行（键为列 key）。
pub fn get_all(
    conn: &Connection,
    sheet_name: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let keys = column_keys(sheet_name);
    let sql = format!("SELECT * FROM {}", sheet_table(sheet_name));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("查询「{}」失败: {}", sheet_name, e))?;
    let rows = stmt
        .query_map([], |row| {
            let mut map = HashMap::new();
            for (index, key) in keys.iter().enumerate() {
                let value: Option<String> = row.get(index).unwrap_or(None);
                map.insert(key.to_string(), value.unwrap_or_default());
            }
            Ok(map)
        })
        .map_err(|e| format!("读取「{}」失败: {}", sheet_name, e))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| format!("解析「{}」行失败: {}", sheet_name, e))?);
    }
    Ok(items)
}

pub fn get_by_id(
    conn: &Connection,
    sheet_name: &str,
    id: &str,
) -> Result<Option<HashMap<String, String>>, String> {
    let items = get_all(conn, sheet_name)?;
    Ok(items
        .into_iter()
        .find(|item| item.get("id").map(|v| v == id).unwrap_or(false)))
}

/// 插入一行（INSERT OR REPLACE：同 id 覆盖）。
pub fn insert_row(
    conn: &Connection,
    sheet_name: &str,
    row: &HashMap<String, String>,
) -> Result<(), String> {
    let keys = column_keys(sheet_name);
    let columns: Vec<String> = keys.iter().map(|key| column_name(key)).collect();
    let placeholders: Vec<String> = (1..=keys.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
        sheet_table(sheet_name),
        columns.join(", "),
        placeholders.join(", ")
    );
    let values: Vec<Option<String>> = keys
        .iter()
        .map(|key| row.get(*key).cloned().filter(|v| !v.is_empty()))
        .collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("写入「{}」失败: {}", sheet_name, e))?;
    let params_ref: Vec<&dyn rusqlite::ToSql> = values
        .iter()
        .map(|value| match value {
            Some(text) => text as &dyn rusqlite::ToSql,
            None => &rusqlite::types::Null as &dyn rusqlite::ToSql,
        })
        .collect();
    stmt.execute(params_ref.as_slice())
        .map_err(|e| format!("写入「{}」失败: {}", sheet_name, e))?;
    Ok(())
}

pub fn delete_row(conn: &Connection, sheet_name: &str, id: &str) -> Result<bool, String> {
    let sql = format!(
        "DELETE FROM {} WHERE {} = ?1",
        sheet_table(sheet_name),
        column_name("id")
    );
    let affected = conn
        .execute(&sql, params![id])
        .map_err(|e| format!("删除「{}」失败: {}", sheet_name, e))?;
    Ok(affected > 0)
}

/// 整表替换（独立事务）。
pub fn replace_all(
    conn: &mut Connection,
    sheet_name: &str,
    rows: &[HashMap<String, String>],
) -> Result<(), String> {
    let tx = conn
        .transaction()
        .map_err(|e| format!("开启事务失败: {}", e))?;
    replace_all_in_tx(&tx, sheet_name, rows)?;
    tx.commit()
        .map_err(|e| format!("提交「{}」替换失败: {}", sheet_name, e))?;
    Ok(())
}

/// 整表替换（调用方已处于事务中，用于一次导入多个表）。
pub fn replace_all_in_tx(
    conn: &Connection,
    sheet_name: &str,
    rows: &[HashMap<String, String>],
) -> Result<(), String> {
    let sql = format!("DELETE FROM {}", sheet_table(sheet_name));
    conn.execute(&sql, [])
        .map_err(|e| format!("清空「{}」失败: {}", sheet_name, e))?;
    for row in rows {
        insert_row(conn, sheet_name, row)?;
    }
    Ok(())
}

/// 校验数据库：可打开且全部业务表存在。
pub fn validate_db(db_path: &str) -> Result<(), String> {
    let conn = connect(db_path)?;
    for &(sheet_name, _) in excel::SHEETS {
        let sql = format!("SELECT name FROM sqlite_master WHERE type='table' AND name=?1");
        let exists: Option<String> = conn
            .query_row(&sql, params![sheet_name], |row| row.get(0))
            .optional()
            .map_err(|e| format!("校验数据库失败「{}」: {}", db_path, e))?;
        if exists.is_none() {
            return Err(format!("数据库缺少业务表「{}」", sheet_name));
        }
    }
    Ok(())
}

/// 从旧 Excel 一次性迁移全部表数据到数据库（仅当数据库尚无数据时执行）。
pub fn migrate_from_xlsx(conn: &Connection, xlsx_path: &str) -> Result<(), String> {
    if !Path::new(xlsx_path).is_file() {
        return Ok(());
    }
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT IN ('meta', 'sqlite_sequence')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("检查数据库状态失败: {}", e))?;
    if existing > 1 {
        return Ok(()); // 已有业务表，不重复迁移
    }
    for &(sheet_name, _) in excel::SHEETS {
        let items = excel::read_xlsx_all(xlsx_path, sheet_name)?;
        for row in items {
            insert_row(conn, sheet_name, &row)?;
        }
    }
    Ok(())
}
