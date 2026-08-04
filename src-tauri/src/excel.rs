use crate::db;
use calamine::{open_workbook_auto, Data, Reader};
use chrono::Local;
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// 全局 Excel 文件访问锁：桌面命令并发执行时，同一时刻只允许一个命令持有
/// mold-data.xlsx 的文件句柄，避免“读取句柄未释放时原子替换”被自身阻塞。
static EXCEL_IO_LOCK: Mutex<()> = Mutex::new(());

fn lock_excel() -> Result<MutexGuard<'static, ()>, String> {
    EXCEL_IO_LOCK
        .lock()
        .map_err(|error| format!("获取数据文件访问锁失败: {}", error))
}

pub const SHEETS: &[(&str, &[(&str, &str)])] = &[
    (
        "螺丝规格表",
        &[
            ("内部ID", "id"),
            ("客户名", "customer"),
            ("外部ID", "externalId"),
            ("螺丝名称", "name"),
            ("螺丝头型", "headType"),
            ("冲头", "punch"),
            ("牙型", "threadType"),
            ("牙板", "die"),
            ("头/垫片大小", "headSize"),
            ("头高", "headHeight"),
            ("长度", "length"),
            ("牙径", "threadDiameter"),
            ("光钉长度", "shankLength"),
            ("线材", "wireMaterial"),
            ("电镀", "plating"),
            ("其他备注", "remark"),
        ],
    ),
    (
        "冲头信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("规格", "spec"),
            ("材质", "material"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "冲头入库记录",
        &[
            ("入库ID", "id"),
            ("冲头ID", "punchId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "冲头领用记录",
        &[
            ("领用ID", "id"),
            ("冲头ID", "punchId"),
            ("领用人", "user"),
            ("领用数量", "quantity"),
            ("领用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "冲头-螺丝规格关联",
        &[
            ("关联ID", "id"),
            ("冲头ID", "punchId"),
            ("螺丝规格ID", "screwSpecId"),
            ("备注", "remark"),
        ],
    ),
    (
        "冲头库存汇总",
        &[
            ("冲头ID", "punchId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
    (
        "牙板信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("机型", "machineType"),
            ("线径", "wireDiameter"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "牙板入库记录",
        &[
            ("入库ID", "id"),
            ("牙板ID", "dieId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "牙板领用记录",
        &[
            ("领用ID", "id"),
            ("牙板ID", "dieId"),
            ("领用人", "user"),
            ("领用数量", "quantity"),
            ("领用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "牙板-螺丝规格关联",
        &[
            ("关联ID", "id"),
            ("牙板ID", "dieId"),
            ("螺丝规格ID", "screwSpecId"),
            ("备注", "remark"),
        ],
    ),
    (
        "牙板库存汇总",
        &[
            ("牙板ID", "dieId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
    (
        "皮带信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("适用机器", "machine"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "皮带入库记录",
        &[
            ("入库ID", "id"),
            ("皮带ID", "beltId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "皮带使用记录",
        &[
            ("使用ID", "id"),
            ("皮带ID", "beltId"),
            ("使用人", "user"),
            ("使用数量", "quantity"),
            ("使用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "皮带库存汇总",
        &[
            ("皮带ID", "beltId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
    (
        "主模具信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("孔径", "holeDiameter"),
            ("对应线材", "wireMaterial"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "主模具入库记录",
        &[
            ("入库ID", "id"),
            ("主模具ID", "mainMoldId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "主模具使用记录",
        &[
            ("使用ID", "id"),
            ("主模具ID", "mainMoldId"),
            ("使用人", "user"),
            ("使用数量", "quantity"),
            ("使用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "主模具-线材关联",
        &[
            ("关联ID", "id"),
            ("主模具ID", "mainMoldId"),
            ("线材规格", "wireMaterial"),
            ("备注", "remark"),
        ],
    ),
    (
        "主模具库存汇总",
        &[
            ("主模具ID", "mainMoldId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
    (
        "剪刀信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("口径", "diameter"),
            ("对应线材", "wireMaterial"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "剪刀入库记录",
        &[
            ("入库ID", "id"),
            ("剪刀ID", "scissorId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "剪刀使用记录",
        &[
            ("使用ID", "id"),
            ("剪刀ID", "scissorId"),
            ("使用人", "user"),
            ("使用数量", "quantity"),
            ("使用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "剪刀-线材关联",
        &[
            ("关联ID", "id"),
            ("剪刀ID", "scissorId"),
            ("线材规格", "wireMaterial"),
            ("备注", "remark"),
        ],
    ),
    (
        "剪刀库存汇总",
        &[
            ("剪刀ID", "scissorId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
    (
        "上冲信息表",
        &[
            ("内部ID", "id"),
            ("名称", "name"),
            ("口径", "diameter"),
            ("对应线材", "wireMaterial"),
            ("安全库存", "safetyStock"),
            ("备注", "remark"),
        ],
    ),
    (
        "上冲入库记录",
        &[
            ("入库ID", "id"),
            ("上冲ID", "upperPunchId"),
            ("入库数量", "quantity"),
            ("入库时间", "orderDate"),
            ("到货状态", "status"),
            ("备注", "remark"),
        ],
    ),
    (
        "上冲使用记录",
        &[
            ("使用ID", "id"),
            ("上冲ID", "upperPunchId"),
            ("使用人", "user"),
            ("使用数量", "quantity"),
            ("使用时间", "useDate"),
            ("备注", "remark"),
        ],
    ),
    (
        "上冲-线材关联",
        &[
            ("关联ID", "id"),
            ("上冲ID", "upperPunchId"),
            ("线材规格", "wireMaterial"),
            ("备注", "remark"),
        ],
    ),
    (
        "上冲库存汇总",
        &[
            ("上冲ID", "upperPunchId"),
            ("名称", "name"),
            ("当前库存", "currentStock"),
            ("安全库存", "safetyStock"),
            ("库存状态", "status"),
        ],
    ),
];

fn get_column_keys(sheet_name: &str) -> Vec<&'static str> {
    for &(name, cols) in SHEETS {
        if name == sheet_name {
            return cols.iter().map(|&(_, key)| key).collect();
        }
    }
    vec![]
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Data::Empty => String::new(),
        Data::Error(e) => format!("ERROR:{:?}", e),
        Data::DateTime(d) => format!("{}", d),
        Data::DateTimeIso(d) => d.clone(),
        Data::DurationIso(d) => d.clone(),
    }
}

fn get_sheet_prefix(sheet_name: &str) -> &'static str {
    match sheet_name {
        "螺丝规格表" => "LS",
        "冲头信息表" => "CT",
        "冲头入库记录" => "CG",
        "冲头领用记录" => "CL",
        "冲头-螺丝规格关联" => "GL",
        "牙板信息表" => "YB",
        "牙板入库记录" => "YG",
        "牙板领用记录" => "YL",
        "牙板-螺丝规格关联" => "YL",
        "皮带信息表" => "PD",
        "皮带入库记录" => "PG",
        "皮带使用记录" => "PS",
        "主模具信息表" => "ZM",
        "主模具入库记录" => "ZG",
        "主模具使用记录" => "ZS",
        "主模具-线材关联" => "ZL",
        "剪刀信息表" => "JD",
        "剪刀入库记录" => "JG",
        "剪刀使用记录" => "JS",
        "剪刀-线材关联" => "JL",
        "上冲信息表" => "SC",
        "上冲入库记录" => "SG",
        "上冲使用记录" => "SS",
        "上冲-线材关联" => "SL",
        _ => "ID",
    }
}

fn generate_id(prefix: &str) -> String {
    let now = Local::now();
    let date_part = now.format("%y%m%d").to_string();
    let time_part = now.format("%H%M%S").to_string();
    let seq = (now.timestamp_millis() % 1000) as u32;
    format!("{}{}{}{:03}", prefix, date_part, time_part, seq)
}

/// 校验数据文件（现为 SQLite 数据库）：可打开且业务表齐全。
pub fn validate_workbook(path: &Path) -> Result<(), String> {
    let _guard = lock_excel()?;
    db::validate_db(&path.to_string_lossy())
}

/// 统计各业务表行数。
pub fn workbook_stats(path: &Path) -> Result<HashMap<String, i64>, String> {
    let _guard = lock_excel()?;
    workbook_stats_inner(path)
}

fn workbook_stats_inner(path: &Path) -> Result<HashMap<String, i64>, String> {
    let conn = db::connect(&path.to_string_lossy())?;
    let mut stats = HashMap::new();
    for &(sheet_name, _) in SHEETS {
        let count = db::get_all(&conn, sheet_name)?.len() as i64;
        stats.insert(sheet_name.to_string(), count);
    }
    Ok(stats)
}

/// 从旧版 Excel 文件读取单个工作表（一次性迁移到数据库时使用）。
pub fn read_xlsx_all(
    xlsx_path: &str,
    sheet_name: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    if !Path::new(xlsx_path).is_file() {
        return Ok(vec![]);
    }
    let mut workbook = open_workbook_auto(xlsx_path)
        .map_err(|e| format!("打开旧数据文件失败「{}」: {}", xlsx_path, e))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| format!("读取工作表「{}」失败: {}", sheet_name, e))?;
    let keys = get_column_keys(sheet_name);
    let mut items = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            continue;
        }
        let mut item = HashMap::new();
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx < keys.len() {
                item.insert(
                    keys[col_idx].to_string(),
                    normalize_value(cell_to_string(cell)),
                );
            }
        }
        items.push(item);
    }
    Ok(items)
}

/// 清洗 JSON 数组格式的字符串，如 ["30R特"] → 30R特
fn normalize_value(val: String) -> String {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
            return arr.join(",");
        }
    }
    val
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            let normalized = match ch {
                '\u{3000}' => ' ',
                '、' => ',',
                '\u{FF01}'..='\u{FF5E}' => char::from_u32(ch as u32 - 0xFEE0).unwrap_or(ch),
                _ => ch,
            };
            normalized.to_lowercase()
        })
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

fn normalize_dimension(value: &str) -> String {
    let mut normalized = normalize_text(value)
        .replace('×', "x")
        .replace('＊', "x")
        .replace('*', "x")
        .replace('φ', "");

    for suffix in ["毫米", "mm"] {
        if normalized.ends_with(suffix) {
            normalized.truncate(normalized.len() - suffix.len());
            break;
        }
    }

    if let Ok(number) = normalized.parse::<f64>() {
        if number.is_finite() {
            return if number.fract() == 0.0 {
                format!("{}", number as i64)
            } else {
                let formatted = format!("{:.12}", number);
                formatted
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            };
        }
    }

    normalized
}

fn normalize_punch_name(value: &str) -> String {
    let normalized = normalize_text(value);
    let chars: Vec<char> = normalized.chars().collect();
    let digit_count = chars.iter().take_while(|ch| ch.is_ascii_digit()).count();

    if digit_count > 0 && digit_count < chars.len() {
        let letter = chars[digit_count];
        if matches!(letter, 'p' | 'b' | 't' | 'f' | 'x' | 'r') {
            let suffix: String = chars[digit_count + 1..].iter().collect();
            if suffix.is_empty() || suffix == "特" {
                let number: String = chars[..digit_count].iter().collect();
                return format!("jm{}m{}{}", letter, number, suffix);
            }
        }
    }

    normalized
}

fn field<'a>(record: &'a HashMap<String, String>, key: &str) -> &'a str {
    record.get(key).map(String::as_str).unwrap_or("")
}

fn business_unique_key(sheet_name: &str, record: &HashMap<String, String>) -> Option<String> {
    match sheet_name {
        "冲头信息表" => Some(
            [
                normalize_punch_name(field(record, "name")),
                normalize_dimension(field(record, "spec")),
                normalize_text(field(record, "material")),
            ]
            .join("|"),
        ),
        "牙板信息表" => Some(
            [
                normalize_text(field(record, "name")),
                normalize_text(field(record, "machineType")),
                normalize_dimension(field(record, "wireDiameter")),
            ]
            .join("|"),
        ),
        _ => None,
    }
}

fn ensure_unique_record(
    sheet_name: &str,
    candidate: &HashMap<String, String>,
    existing_rows: &[HashMap<String, String>],
    exclude_id: Option<&str>,
) -> Result<(), String> {
    let Some(candidate_key) = business_unique_key(sheet_name, candidate) else {
        return Ok(());
    };

    if let Some(existing) = existing_rows.iter().find(|row| {
        let is_current = exclude_id.is_some_and(|id| field(row, "id") == id);
        !is_current
            && business_unique_key(sheet_name, row).as_deref() == Some(candidate_key.as_str())
    }) {
        let resource = if sheet_name == "冲头信息表" {
            "冲头"
        } else {
            "牙板"
        };
        return Err(format!(
            "DUPLICATE_RECORD|{}|{}",
            resource,
            field(existing, "id")
        ));
    }

    Ok(())
}

pub fn get_all(file_path: &str, sheet_name: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let _guard = lock_excel()?;
    get_all_inner(file_path, sheet_name)
}

fn get_all_inner(
    file_path: &str,
    sheet_name: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let conn = db::connect(file_path)?;
    db::get_all(&conn, sheet_name)
}

pub fn get_by_id(
    file_path: &str,
    sheet_name: &str,
    id: &str,
) -> Result<Option<HashMap<String, String>>, String> {
    let _guard = lock_excel()?;
    let conn = db::connect(file_path)?;
    db::get_by_id(&conn, sheet_name, id)
}

pub fn add_row(
    file_path: &str,
    sheet_name: &str,
    item: &HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let _guard = lock_excel()?;
    add_row_inner(file_path, sheet_name, item)
}

fn add_row_inner(
    file_path: &str,
    sheet_name: &str,
    item: &HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let conn = db::connect(file_path)?;
    let mut result = item.clone();
    if result.get("id").map(|v| v.is_empty()).unwrap_or(true) || !result.contains_key("id") {
        let prefix = get_sheet_prefix(sheet_name);
        result.insert("id".to_string(), generate_id(prefix));
    }
    let all_rows = db::get_all(&conn, sheet_name)?;
    ensure_unique_record(sheet_name, &result, &all_rows, None)?;
    db::insert_row(&conn, sheet_name, &result)?;
    Ok(result)
}

pub fn update_row(
    file_path: &str,
    sheet_name: &str,
    id: &str,
    data: &HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let _guard = lock_excel()?;
    update_row_inner(file_path, sheet_name, id, data)
}

fn update_row_inner(
    file_path: &str,
    sheet_name: &str,
    id: &str,
    data: &HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let conn = db::connect(file_path)?;
    let all_rows = db::get_all(&conn, sheet_name)?;
    let Some(current) = all_rows.iter().find(|row| field(row, "id") == id) else {
        return Err("记录未找到".to_string());
    };

    let mut candidate = current.clone();
    for (key, value) in data {
        candidate.insert(key.clone(), value.clone());
    }
    candidate.insert("id".to_string(), id.to_string());
    if business_unique_key(sheet_name, current) != business_unique_key(sheet_name, &candidate) {
        ensure_unique_record(sheet_name, &candidate, &all_rows, Some(id))?;
    }

    db::insert_row(&conn, sheet_name, &candidate)?;
    Ok(candidate)
}

pub fn delete_row(file_path: &str, sheet_name: &str, id: &str) -> Result<bool, String> {
    let _guard = lock_excel()?;
    let conn = db::connect(file_path)?;
    db::delete_row(&conn, sheet_name, id)
}

/// 导出分组定义：每组包含的业务表（用于按业务拆分 Excel 导入导出）。
pub const EXPORT_GROUPS: &[(&str, &[&str])] = &[
    ("螺丝规格", &["螺丝规格表"]),
    (
        "冲头",
        &[
            "冲头信息表",
            "冲头入库记录",
            "冲头领用记录",
            "冲头-螺丝规格关联",
            "冲头库存汇总",
        ],
    ),
    (
        "牙板",
        &[
            "牙板信息表",
            "牙板入库记录",
            "牙板领用记录",
            "牙板-螺丝规格关联",
            "牙板库存汇总",
        ],
    ),
    (
        "皮带",
        &["皮带信息表", "皮带入库记录", "皮带使用记录", "皮带库存汇总"],
    ),
    (
        "主模具",
        &[
            "主模具信息表",
            "主模具入库记录",
            "主模具使用记录",
            "主模具-线材关联",
            "主模具库存汇总",
        ],
    ),
    (
        "剪刀",
        &[
            "剪刀信息表",
            "剪刀入库记录",
            "剪刀使用记录",
            "剪刀-线材关联",
            "剪刀库存汇总",
        ],
    ),
    (
        "上冲",
        &[
            "上冲信息表",
            "上冲入库记录",
            "上冲使用记录",
            "上冲-线材关联",
            "上冲库存汇总",
        ],
    ),
];

/// 生成单个分组的 Excel 文件内容（该组全部业务表写入一个工作簿）。
pub fn export_group_xlsx(file_path: &str, group_sheets: &[&str]) -> Result<Vec<u8>, String> {
    let _guard = lock_excel()?;
    let conn = db::connect(file_path)?;
    let mut workbook = Workbook::new();
    for &sheet_name in group_sheets {
        let Some((_, cols)) = SHEETS.iter().find(|(name, _)| *name == sheet_name) else {
            continue;
        };
        let sheet = workbook
            .add_worksheet()
            .set_name(sheet_name)
            .map_err(|e| e.to_string())?;
        for (i, &(header, _)) in cols.iter().enumerate() {
            sheet
                .write_string(0, i as u16, header)
                .map_err(|e| e.to_string())?;
        }
        let rows = db::get_all(&conn, sheet_name)?;
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, &(_, key)) in cols.iter().enumerate() {
                if let Some(value) = row.get(key) {
                    sheet
                        .write_string((row_idx + 1) as u32, col_idx as u16, value)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| format!("生成 Excel 失败: {}", e))?;
    Ok(buffer)
}

/// 列出 Excel 文件中属于本系统业务表的工作表名（导入时供用户勾选）。
pub fn list_excel_sheets(xlsx_path: &str) -> Result<Vec<String>, String> {
    if !Path::new(xlsx_path).is_file() {
        return Err("所选文件不存在".to_string());
    }
    let workbook = open_workbook_auto(xlsx_path)
        .map_err(|e| format!("打开 Excel 文件失败「{}」: {}", xlsx_path, e))?;
    let names = workbook.sheet_names().to_vec();
    let known: Vec<String> = names
        .into_iter()
        .filter(|name| SHEETS.iter().any(|(sheet, _)| sheet == name))
        .collect();
    if known.is_empty() {
        return Err("Excel 文件中没有可识别的业务表".to_string());
    }
    Ok(known)
}

/// 从 Excel 文件导入选中的工作表（整表替换，事务内完成）。
pub fn import_sheets_from_xlsx(
    file_path: &str,
    xlsx_path: &str,
    selected_sheets: &[String],
) -> Result<HashMap<String, i64>, String> {
    let _guard = lock_excel()?;
    let mut conn = db::connect(file_path)?;
    let mut stats = HashMap::new();
    let tx = conn
        .transaction()
        .map_err(|e| format!("开启事务失败: {}", e))?;
    for sheet_name in selected_sheets {
        let rows = read_xlsx_all(xlsx_path, sheet_name)?;
        db::replace_all_in_tx(&tx, sheet_name, &rows)?;
        stats.insert(sheet_name.clone(), rows.len() as i64);
    }
    tx.commit().map_err(|e| format!("提交导入失败: {}", e))?;
    Ok(stats)
}

/// 导出全部业务表为单个 Excel（保留旧入口兼容，推荐改用分组导出）。
pub fn export_data(file_path: &str) -> Result<Vec<u8>, String> {
    let all_sheets: Vec<&str> = SHEETS.iter().map(|(name, _)| *name).collect();
    export_group_xlsx(file_path, &all_sheets)
}

pub fn calculate_stock(
    file_path: &str,
    stock_type: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let _guard = lock_excel()?;
    calculate_stock_inner(file_path, stock_type)
}

fn calculate_stock_inner(
    file_path: &str,
    stock_type: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let conn = db::connect(file_path)?;
    let (info_sheet, order_sheet, use_sheet, stock_sheet, item_id_key) = match stock_type {
        "punch" => (
            "冲头信息表",
            "冲头入库记录",
            "冲头领用记录",
            "冲头库存汇总",
            "punchId",
        ),
        "die" => (
            "牙板信息表",
            "牙板入库记录",
            "牙板领用记录",
            "牙板库存汇总",
            "dieId",
        ),
        "belt" => (
            "皮带信息表",
            "皮带入库记录",
            "皮带使用记录",
            "皮带库存汇总",
            "beltId",
        ),
        "mainMold" => (
            "主模具信息表",
            "主模具入库记录",
            "主模具使用记录",
            "主模具库存汇总",
            "mainMoldId",
        ),
        "scissor" => (
            "剪刀信息表",
            "剪刀入库记录",
            "剪刀使用记录",
            "剪刀库存汇总",
            "scissorId",
        ),
        "upperPunch" => (
            "上冲信息表",
            "上冲入库记录",
            "上冲使用记录",
            "上冲库存汇总",
            "upperPunchId",
        ),
        _ => return Err("未知类型".to_string()),
    };
    let info_items = db::get_all(&conn, info_sheet)?;
    let orders = db::get_all(&conn, order_sheet)?;
    let uses = db::get_all(&conn, use_sheet)?;
    let stock_data: Vec<HashMap<String, String>> = info_items
        .iter()
        .map(|item| {
            let item_id = item.get("id").cloned().unwrap_or_default();
            let total_ordered: i64 = orders
                .iter()
                .filter(|o| {
                    o.get(item_id_key).map(|v| v == &item_id).unwrap_or(false)
                        && o.get("status").map(|v| v == "已到货").unwrap_or(false)
                })
                .filter_map(|o| o.get("quantity").and_then(|q| q.parse::<i64>().ok()))
                .sum();
            let total_used: i64 = uses
                .iter()
                .filter(|u| u.get(item_id_key).map(|v| v == &item_id).unwrap_or(false))
                .filter_map(|u| u.get("quantity").and_then(|q| q.parse::<i64>().ok()))
                .sum();
            let current_stock = total_ordered - total_used;
            let safety_stock: i64 = item
                .get("safety_stock")
                .or_else(|| item.get("safetyStock"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let mut row = HashMap::new();
            row.insert(item_id_key.to_string(), item_id);
            row.insert(
                "name".to_string(),
                item.get("name").cloned().unwrap_or_default(),
            );
            row.insert("currentStock".to_string(), current_stock.to_string());
            row.insert("safetyStock".to_string(), safety_stock.to_string());
            row.insert(
                "status".to_string(),
                if current_stock < safety_stock {
                    "需入库".to_string()
                } else {
                    "安全".to_string()
                },
            );
            row
        })
        .collect();
    let mut conn = conn;
    db::replace_all(&mut conn, stock_sheet, &stock_data)?;
    Ok(stock_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn record(fields: &[(&str, &str)]) -> HashMap<String, String> {
        fields
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn excel_import_test_helper(
        db_path: &str,
        xlsx_path: &Path,
    ) -> Result<HashMap<String, i64>, String> {
        let sheets = list_excel_sheets(xlsx_path.to_str().unwrap())?;
        import_sheets_from_xlsx(db_path, xlsx_path.to_str().unwrap(), &sheets)
    }

    #[test]
    fn punch_short_and_full_names_share_unique_key() {
        let short = record(&[("name", "30R"), ("spec", "14.0 mm"), ("material", "SKD11")]);
        let full = record(&[("name", "JMR M30"), ("spec", "14"), ("material", "skd11")]);
        assert_eq!(
            business_unique_key("冲头信息表", &short),
            business_unique_key("冲头信息表", &full)
        );
    }

    #[test]
    fn die_numeric_diameters_share_unique_key() {
        let left = record(&[
            ("name", "牙板 A"),
            ("machineType", "M12"),
            ("wireDiameter", "Φ14.0 mm"),
        ]);
        let right = record(&[
            ("name", "牙板A"),
            ("machineType", "m12"),
            ("wireDiameter", "14"),
        ]);
        assert_eq!(
            business_unique_key("牙板信息表", &left),
            business_unique_key("牙板信息表", &right)
        );
    }

    #[test]
    fn invalid_import_rejected() {
        // 数据库创建后导入非法 Excel 不应影响数据库结构
        let root =
            std::env::temp_dir().join(format!("mold-excel-import-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("mold-data.db");
        let conn = db::connect(path.to_str().unwrap()).unwrap();
        db::init_schema(&conn).unwrap();
        drop(conn);
        let invalid = root.join("invalid.xlsx");
        fs::write(&invalid, b"not-an-xlsx").unwrap();
        let result = excel_import_test_helper(path.to_str().unwrap(), &invalid);
        assert!(result.is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_excludes_current_record_but_rejects_other_duplicate() {
        let rows = vec![
            record(&[
                ("id", "CT1"),
                ("name", "30R"),
                ("spec", "14"),
                ("material", "SKD11"),
            ]),
            record(&[
                ("id", "CT2"),
                ("name", "31R"),
                ("spec", "15"),
                ("material", "SKD11"),
            ]),
        ];
        assert!(ensure_unique_record("冲头信息表", &rows[0], &rows, Some("CT1")).is_ok());

        let duplicate = record(&[
            ("id", "CT2"),
            ("name", "JMR M30"),
            ("spec", "14.0"),
            ("material", "skd11"),
        ]);
        assert_eq!(
            ensure_unique_record("冲头信息表", &duplicate, &rows, Some("CT2")),
            Err("DUPLICATE_RECORD|冲头|CT1".to_string())
        );
    }
}

pub fn get_default_file_path() -> String {
    let base = if cfg!(debug_assertions) {
        std::env::current_dir()
            .map(|p| p.join("..").join("data").to_string_lossy().to_string())
            .unwrap_or_else(|_| "./data".to_string())
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .map(|d| d.join("data").to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "./data".to_string())
    };
    format!("{}/mold-data.db", base)
}
