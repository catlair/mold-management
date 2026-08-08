use crate::excel;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    }
    ensure_unique_indexes(conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("记录 schema 版本失败: {}", e))?;
    Ok(())
}

/// 为所有含 id 列的业务表建立/补齐 id 唯一索引（幂等）。
/// 注意：索引名必须用「idx_表名」而非 ASCII 化——SQLite 索引名是数据库级全局唯一的，
/// 把中文表名替换成下划线会互相冲突，导致 `IF NOT EXISTS` 静默跳过、多数表永远无索引
/// （历史上曾因此出现 update 退化为追加重复行、AI 批量修改把表写翻倍的事故）。
pub fn ensure_unique_indexes(conn: &Connection) -> Result<(), String> {
    for &(sheet_name, columns) in excel::SHEETS {
        let has_id = columns.iter().any(|&(_, key)| key == "id");
        if !has_id {
            continue;
        }
        // 清理历史上 ASCII 化产生的冲突索引名（下划线串），避免冗余残留。
        for name in [
            "idx______",
            "idx_______",
            "idx________",
            "idx_________",
            "idx__________",
        ] {
            let _ = conn.execute_batch(&format!("DROP INDEX IF EXISTS \"{}\";", name));
        }
        let idx_name = format!("idx_{}", sheet_name);
        conn.execute_batch(&format!(
            "CREATE UNIQUE INDEX IF NOT EXISTS \"{}\" ON {} ({});",
            idx_name,
            sheet_table(sheet_name),
            column_name("id")
        ))
        .map_err(|e| format!("创建表「{}」唯一索引失败: {}", sheet_name, e))?;
    }
    Ok(())
}

/// 按 id 更新记录（真正的 UPDATE 语义，不依赖 INSERT OR REPLACE 的冲突行为）。
/// data 中与 id 相同的键会被忽略，id 由 WHERE 条件确定。
pub fn update_row(
    conn: &Connection,
    sheet_name: &str,
    id: &str,
    data: &HashMap<String, String>,
) -> Result<(), String> {
    let keys = column_keys(sheet_name);
    let mut set_clauses: Vec<String> = Vec::new();
    let mut values: Vec<Option<String>> = Vec::new();
    for key in keys {
        if key == "id" {
            continue;
        }
        set_clauses.push(format!("{} = ?{}", column_name(key), set_clauses.len() + 1));
        values.push(data.get(key).cloned().filter(|v| !v.is_empty()));
    }
    if set_clauses.is_empty() {
        return Err("没有可更新的字段".to_string());
    }
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ?{}",
        sheet_table(sheet_name),
        set_clauses.join(", "),
        column_name("id"),
        set_clauses.len() + 1
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("准备更新「{}」失败: {}", sheet_name, e))?;
    let params_ref: Vec<&dyn rusqlite::ToSql> = values
        .iter()
        .map(|value| match value {
            Some(text) => text as &dyn rusqlite::ToSql,
            None => &rusqlite::types::Null as &dyn rusqlite::ToSql,
        })
        .collect();
    let mut final_params = params_ref;
    final_params.push(&id);
    stmt.execute(final_params.as_slice())
        .map_err(|e| format!("更新「{}」失败: {}", sheet_name, e))?;
    Ok(())
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

/// 校验数据库：可打开、快速完整性检查通过且全部业务表存在。
pub fn validate_db(db_path: &str) -> Result<(), String> {
    let conn = connect(db_path)?;
    let quick_check: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|e| format!("执行数据库快速完整性检查失败「{}」: {}", db_path, e))?;
    if quick_check != "ok" {
        return Err(format!(
            "数据库快速完整性检查失败「{}」: {}",
            db_path, quick_check
        ));
    }
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

/// 计算稳定的业务内容 SHA-256：只包含 schema 版本、表名、列名和按稳定顺序排列的值，
/// 不依赖 SQLite 页布局，因此应用反复启动/退出不会产生虚假“数据变化”。
pub fn content_sha256(db_path: &str) -> Result<String, String> {
    let conn = connect(db_path)?;
    let mut hasher = Sha256::new();
    hasher.update(b"mold-management-db-content-v1\0");
    hasher.update(SCHEMA_VERSION.as_bytes());
    hasher.update([0]);

    for &(sheet_name, columns) in excel::SHEETS {
        hasher.update(sheet_name.as_bytes());
        hasher.update([0]);
        let keys: Vec<&str> = columns.iter().map(|&(_, key)| key).collect();
        for key in &keys {
            hasher.update(key.as_bytes());
            hasher.update([0]);
        }

        let order_by = if keys.contains(&"id") {
            column_name("id")
        } else {
            "rowid".to_string()
        };
        let sql = format!(
            "SELECT * FROM {} ORDER BY {}",
            sheet_table(sheet_name),
            order_by
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("准备内容指纹查询失败「{}」: {}", sheet_name, e))?;
        let rows = stmt
            .query_map([], |row| {
                let mut values = Vec::with_capacity(keys.len());
                for index in 0..keys.len() {
                    let value: Option<String> = row.get(index)?;
                    values.push(value);
                }
                Ok(values)
            })
            .map_err(|e| format!("读取内容指纹数据失败「{}」: {}", sheet_name, e))?;
        for row in rows {
            for value in
                row.map_err(|e| format!("解析内容指纹数据失败「{}」: {}", sheet_name, e))?
            {
                match value {
                    Some(value) => {
                        hasher.update(b"V");
                        hasher.update((value.len() as u64).to_le_bytes());
                        hasher.update(value.as_bytes());
                    }
                    None => hasher.update(b"N"),
                }
            }
            hasher.update([0xff]);
        }
        hasher.update([0xfe]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// 为 WAL 模式数据库创建独立、一致的 SQLite 快照。
pub fn create_snapshot(db_path: &str) -> Result<PathBuf, String> {
    let source = Path::new(db_path);
    if !source.is_file() {
        return Err(format!("数据文件不存在「{}」", source.display()));
    }
    let snapshot = crate::storage::temporary_path(source, "snapshot.db")?;
    let conn = connect(db_path)?;
    let escaped = snapshot.to_string_lossy().replace('\'', "''");
    if let Err(error) = conn.execute_batch(&format!("VACUUM INTO '{}';", escaped)) {
        let _ = std::fs::remove_file(&snapshot);
        return Err(format!(
            "创建数据库一致性快照失败「{}」: {}",
            snapshot.display(),
            error
        ));
    }
    drop(conn);
    crate::storage::sync_file(&snapshot)?;
    validate_db(&snapshot.to_string_lossy())?;
    Ok(snapshot)
}

/// 从旧 Excel 一次性迁移全部表数据到数据库（仅当数据库尚无数据时执行）。
pub fn migrate_from_xlsx(conn: &Connection, xlsx_path: &str) -> Result<(), String> {
    if !Path::new(xlsx_path).is_file() {
        return Ok(());
    }
    // 以“是否有数据行”判断迁移条件：init_schema 会预先建表，不能用表数量判断。
    for &(sheet_name, _) in excel::SHEETS {
        let count: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM {}", sheet_table(sheet_name)),
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("检查「{}」数据失败: {}", sheet_name, e))?;
        if count > 0 {
            return Ok(()); // 库中已有数据，不重复迁移
        }
    }
    for &(sheet_name, _) in excel::SHEETS {
        let items = excel::read_xlsx_all(xlsx_path, sheet_name)?;
        for row in items {
            // 跳过与已迁移记录重复的 id（保留先迁移的一条），避免历史重复数据再次进入。
            let id = row.get("id").cloned().unwrap_or_default();
            if !id.is_empty() {
                let exists: bool = conn
                    .query_row(
                        &format!(
                            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = ?1)",
                            sheet_table(sheet_name),
                            column_name("id")
                        ),
                        params![&id],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);
                if exists {
                    continue;
                }
            }
            insert_row(conn, sheet_name, &row)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_db_path(tag: &str) -> String {
        std::env::temp_dir()
            .join(format!(
                "mold-db-fingerprint-{}-{}.db",
                tag,
                Uuid::new_v4().simple()
            ))
            .to_string_lossy()
            .to_string()
    }

    fn create_screw_rows(db_path: &str, rows: &[(&str, &str)]) -> Result<(), String> {
        let conn = connect(db_path)?;
        init_schema(&conn)?;
        for (id, name) in rows {
            conn.execute(
                "INSERT INTO \"螺丝规格表\" (id, customer, externalId, name, headType, \
                 punch, threadType, die, headSize, headHeight, length, threadDiameter, \
                 shankLength, wireMaterial, plating, remark) \
                 VALUES (?1, '', '', ?2, '', '', '', '', '', '', '', '', '', '', '', '')",
                params![id, name],
            )
            .map_err(|e| format!("插入测试数据失败: {}", e))?;
        }
        drop(conn);
        Ok(())
    }

    #[test]
    fn content_fingerprint_is_stable_across_physical_rebuilds() {
        let path_a = temp_db_path("a");
        let path_b = temp_db_path("b");
        let rows = [("S1", "M3x8 自攻"), ("S2", "M4x10 机牙")];
        create_screw_rows(&path_a, &rows).unwrap();
        create_screw_rows(&path_b, &rows).unwrap();
        assert_eq!(
            content_sha256(&path_a).unwrap(),
            content_sha256(&path_b).unwrap()
        );
        let _ = std::fs::remove_file(&path_a);
        let _ = std::fs::remove_file(&path_b);
    }

    #[test]
    fn content_fingerprint_ignores_insert_order_for_id_keyed_tables() {
        let path_a = temp_db_path("a");
        let path_b = temp_db_path("b");
        create_screw_rows(&path_a, &[("S1", "A"), ("S2", "B")]).unwrap();
        create_screw_rows(&path_b, &[("S2", "B"), ("S1", "A")]).unwrap();
        assert_eq!(
            content_sha256(&path_a).unwrap(),
            content_sha256(&path_b).unwrap()
        );
        let _ = std::fs::remove_file(&path_a);
        let _ = std::fs::remove_file(&path_b);
    }

    #[test]
    fn content_fingerprint_changes_when_business_data_changes() {
        let path = temp_db_path("c");
        create_screw_rows(&path, &[("S1", "A")]).unwrap();
        let before = content_sha256(&path).unwrap();
        let conn = connect(&path).unwrap();
        conn.execute("UPDATE \"螺丝规格表\" SET name = 'B' WHERE id = 'S1'", [])
            .unwrap();
        drop(conn);
        let after = content_sha256(&path).unwrap();
        assert_ne!(before, after);
        let _ = std::fs::remove_file(&path);
    }
}
