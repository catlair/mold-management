//! 操作日志：记录用户对业务数据的增删改与导入操作。
//! - 独立于 excel::SHEETS 的业务表，不参与 Excel 导入导出、指纹计算与库存汇总。
//! - 日志写入同一 SQLite 数据文件，随备份与 WebDAV 同步一起保存。
//! - 保留策略：超过 RETAIN_DAYS 的旧日志被清理；总量超过 MAX_ROWS 时保留最新部分。

use rusqlite::{params, Connection};
use serde_json::{json, Value};

pub const LOG_TABLE: &str = "operation_log";
/// 日志保留天数（"保留长一点"：默认一年）
pub const RETAIN_DAYS: i64 = 365;
/// 日志最多保留条数
pub const MAX_ROWS: i64 = 50_000;

/// 幂等创建操作日志表。
pub fn ensure_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS operation_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts INTEGER NOT NULL,
            table_name TEXT NOT NULL,
            operation TEXT NOT NULL,
            record_id TEXT,
            summary TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("创建操作日志表失败: {}", e))?;
    Ok(())
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 写入一条操作日志（随后顺带清理过期日志）。
pub fn log_operation(
    conn: &Connection,
    table_name: &str,
    operation: &str,
    record_id: &str,
    summary: &str,
) -> Result<(), String> {
    ensure_table(conn)?;
    conn.execute(
        &format!(
            "INSERT INTO {} (ts, table_name, operation, record_id, summary) VALUES (?1, ?2, ?3, ?4, ?5)",
            LOG_TABLE
        ),
        params![now_unix(), table_name, operation, record_id, summary],
    )
    .map_err(|e| format!("写入操作日志失败: {}", e))?;
    let _ = cleanup(conn);
    Ok(())
}

/// 清理过期/超量日志（每次写入后顺带执行，成本低）。
fn cleanup(conn: &Connection) -> Result<(), String> {
    let cutoff = now_unix() - RETAIN_DAYS * 86_400;
    conn.execute(
        &format!("DELETE FROM {} WHERE ts < ?1", LOG_TABLE),
        params![cutoff],
    )
    .map_err(|e| format!("清理过期操作日志失败: {}", e))?;
    conn.execute(
        &format!(
            "DELETE FROM {} WHERE id NOT IN (SELECT id FROM {} ORDER BY id DESC LIMIT ?1)",
            LOG_TABLE, LOG_TABLE
        ),
        params![MAX_ROWS],
    )
    .map_err(|e| format!("限制操作日志数量失败: {}", e))?;
    Ok(())
}

/// 日志总条数。
pub fn count_logs(conn: &Connection) -> Result<i64, String> {
    ensure_table(conn)?;
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", LOG_TABLE), [], |row| {
        row.get::<_, i64>(0)
    })
    .map_err(|e| format!("统计操作日志失败: {}", e))
}

/// 按时间倒序取日志（最新在前）。
pub fn get_logs(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<Value>, String> {
    ensure_table(conn)?;
    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, ts, table_name, operation, record_id, summary FROM {} ORDER BY id DESC LIMIT ?1 OFFSET ?2",
            LOG_TABLE
        ))
        .map_err(|e| format!("查询操作日志失败: {}", e))?;
    let rows = stmt
        .query_map(params![limit, offset], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "ts": row.get::<_, i64>(1)?,
                "tableName": row.get::<_, String>(2)?,
                "operation": row.get::<_, String>(3)?,
                "recordId": row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                "summary": row.get::<_, String>(5)?,
            }))
        })
        .map_err(|e| format!("查询操作日志失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取操作日志失败: {}", e))?;
    Ok(rows)
}

/// 清空全部操作日志。
pub fn clear_logs(conn: &Connection) -> Result<(), String> {
    ensure_table(conn)?;
    conn.execute(&format!("DELETE FROM {}", LOG_TABLE), [])
        .map_err(|e| format!("清空操作日志失败: {}", e))?;
    Ok(())
}
