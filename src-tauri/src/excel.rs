use calamine::{open_workbook_auto, Reader, Data};
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use chrono::Local;

pub const SHEETS: &[(&str, &[(&str, &str)])] = &[
    ("螺丝规格表", &[
        ("内部ID", "id"), ("客户名", "customer"), ("外部ID", "externalId"),
        ("螺丝名称", "name"), ("螺丝头型", "headType"), ("冲头", "punch"),
        ("牙型", "threadType"), ("牙板", "die"), ("头/垫片大小", "headSize"),
        ("头高", "headHeight"), ("长度", "length"), ("牙径", "threadDiameter"),
        ("光钉长度", "shankLength"), ("线材", "wireMaterial"), ("电镀", "plating"),
        ("其他备注", "remark"),
    ]),
    ("冲头信息表", &[("内部ID", "id"), ("名称", "name"), ("规格", "spec"), ("材质", "material"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("冲头订购记录", &[("订购ID", "id"), ("冲头ID", "punchId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("冲头领用记录", &[("领用ID", "id"), ("冲头ID", "punchId"), ("领用人", "user"), ("领用数量", "quantity"), ("领用时间", "useDate"), ("备注", "remark")]),
    ("冲头-螺丝规格关联", &[("关联ID", "id"), ("冲头ID", "punchId"), ("螺丝规格ID", "screwSpecId"), ("备注", "remark")]),
    ("冲头库存汇总", &[("冲头ID", "punchId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
    ("牙板信息表", &[("内部ID", "id"), ("名称", "name"), ("机型", "machineType"), ("线径", "wireDiameter"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("牙板订购记录", &[("订购ID", "id"), ("牙板ID", "dieId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("牙板领用记录", &[("领用ID", "id"), ("牙板ID", "dieId"), ("领用人", "user"), ("领用数量", "quantity"), ("领用时间", "useDate"), ("备注", "remark")]),
    ("牙板-螺丝规格关联", &[("关联ID", "id"), ("牙板ID", "dieId"), ("螺丝规格ID", "screwSpecId"), ("备注", "remark")]),
    ("牙板库存汇总", &[("牙板ID", "dieId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
    ("皮带信息表", &[("内部ID", "id"), ("名称", "name"), ("适用机器", "machine"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("皮带订购记录", &[("订购ID", "id"), ("皮带ID", "beltId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("皮带使用记录", &[("使用ID", "id"), ("皮带ID", "beltId"), ("使用人", "user"), ("使用数量", "quantity"), ("使用时间", "useDate"), ("备注", "remark")]),
    ("皮带库存汇总", &[("皮带ID", "beltId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
    ("主模具信息表", &[("内部ID", "id"), ("名称", "name"), ("孔径", "holeDiameter"), ("对应线材", "wireMaterial"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("主模具订购记录", &[("订购ID", "id"), ("主模具ID", "mainMoldId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("主模具使用记录", &[("使用ID", "id"), ("主模具ID", "mainMoldId"), ("使用人", "user"), ("使用数量", "quantity"), ("使用时间", "useDate"), ("备注", "remark")]),
    ("主模具-线材关联", &[("关联ID", "id"), ("主模具ID", "mainMoldId"), ("线材规格", "wireMaterial"), ("备注", "remark")]),
    ("主模具库存汇总", &[("主模具ID", "mainMoldId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
    ("剪刀信息表", &[("内部ID", "id"), ("名称", "name"), ("口径", "diameter"), ("对应线材", "wireMaterial"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("剪刀订购记录", &[("订购ID", "id"), ("剪刀ID", "scissorId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("剪刀使用记录", &[("使用ID", "id"), ("剪刀ID", "scissorId"), ("使用人", "user"), ("使用数量", "quantity"), ("使用时间", "useDate"), ("备注", "remark")]),
    ("剪刀-线材关联", &[("关联ID", "id"), ("剪刀ID", "scissorId"), ("线材规格", "wireMaterial"), ("备注", "remark")]),
    ("剪刀库存汇总", &[("剪刀ID", "scissorId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
    ("上冲信息表", &[("内部ID", "id"), ("名称", "name"), ("口径", "diameter"), ("对应线材", "wireMaterial"), ("安全库存", "safetyStock"), ("备注", "remark")]),
    ("上冲订购记录", &[("订购ID", "id"), ("上冲ID", "upperPunchId"), ("订购数量", "quantity"), ("订购时间", "orderDate"), ("到货状态", "status"), ("备注", "remark")]),
    ("上冲使用记录", &[("使用ID", "id"), ("上冲ID", "upperPunchId"), ("使用人", "user"), ("使用数量", "quantity"), ("使用时间", "useDate"), ("备注", "remark")]),
    ("上冲-线材关联", &[("关联ID", "id"), ("上冲ID", "upperPunchId"), ("线材规格", "wireMaterial"), ("备注", "remark")]),
    ("上冲库存汇总", &[("上冲ID", "upperPunchId"), ("名称", "name"), ("当前库存", "currentStock"), ("安全库存", "safetyStock"), ("库存状态", "status")]),
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
        Data::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
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
        "冲头订购记录" => "CG",
        "冲头领用记录" => "CL",
        "冲头-螺丝规格关联" => "GL",
        "牙板信息表" => "YB",
        "牙板订购记录" => "YG",
        "牙板领用记录" => "YL",
        "牙板-螺丝规格关联" => "YL",
        "皮带信息表" => "PD",
        "皮带订购记录" => "PG",
        "皮带使用记录" => "PS",
        "主模具信息表" => "ZM",
        "主模具订购记录" => "ZG",
        "主模具使用记录" => "ZS",
        "主模具-线材关联" => "ZL",
        "剪刀信息表" => "JD",
        "剪刀订购记录" => "JG",
        "剪刀使用记录" => "JS",
        "剪刀-线材关联" => "JL",
        "上冲信息表" => "SC",
        "上冲订购记录" => "SG",
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

fn format_radix(mut value: i64, base: u32) -> String {
    if value == 0 { return "0".to_string(); }
    let mut result = String::new();
    let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
    while value > 0 {
        let remainder = (value % base as i64) as u32;
        result.push(digits.chars().nth(remainder as usize).unwrap());
        value /= base as i64;
    }
    result.chars().rev().collect()
}

fn create_empty_workbook(file_path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let mut workbook = Workbook::new();
    for &(sheet_name, cols) in SHEETS {
        let sheet = workbook.add_worksheet()
            .set_name(sheet_name)
            .map_err(|e| e.to_string())?;
        for (i, &(header, _)) in cols.iter().enumerate() {
            sheet.write_string(0, i as u16, header).map_err(|e| e.to_string())?;
        }
    }
    workbook.save(file_path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_all(file_path: &str, sheet_name: &str) -> Result<Vec<HashMap<String, String>>, String> {
    if !Path::new(file_path).exists() {
        create_empty_workbook(file_path)?;
        return Ok(vec![]);
    }
    let mut workbook = open_workbook_auto(file_path).map_err(|e| e.to_string())?;
    let range = workbook.worksheet_range(sheet_name).map_err(|e| e.to_string())?;
    let keys = get_column_keys(sheet_name);
    let mut items = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 { continue; }
        let mut item = HashMap::new();
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx < keys.len() {
                item.insert(keys[col_idx].to_string(), cell_to_string(cell));
            }
        }
        items.push(item);
    }
    Ok(items)
}

pub fn get_by_id(file_path: &str, sheet_name: &str, id: &str) -> Result<Option<HashMap<String, String>>, String> {
    let items = get_all(file_path, sheet_name)?;
    Ok(items.into_iter().find(|item| item.get("id").map(|v| v == id).unwrap_or(false)))
}

pub fn add_row(file_path: &str, sheet_name: &str, item: &HashMap<String, String>) -> Result<HashMap<String, String>, String> {
    if !Path::new(file_path).exists() {
        create_empty_workbook(file_path)?;
    }
    let mut result = item.clone();
    if result.get("id").map(|v| v.is_empty()).unwrap_or(true) || !result.contains_key("id") {
        let prefix = get_sheet_prefix(sheet_name);
        result.insert("id".to_string(), generate_id(prefix));
    }
    let mut all_rows = get_all(file_path, sheet_name)?;
    all_rows.push(result.clone());
    write_sheet_data(file_path, sheet_name, &all_rows)?;
    Ok(result)
}

pub fn update_row(file_path: &str, sheet_name: &str, id: &str, data: &HashMap<String, String>) -> Result<HashMap<String, String>, String> {
    let mut all_rows = get_all(file_path, sheet_name)?;
    let mut found = false;
    for row in &mut all_rows {
        if row.get("id").map(|v| v == id).unwrap_or(false) {
            for (key, value) in data {
                row.insert(key.clone(), value.clone());
            }
            found = true;
            break;
        }
    }
    if !found { return Err("记录未找到".to_string()); }
    write_sheet_data(file_path, sheet_name, &all_rows)?;
    get_by_id(file_path, sheet_name, id)?.ok_or("更新后未找到记录".to_string())
}

pub fn delete_row(file_path: &str, sheet_name: &str, id: &str) -> Result<bool, String> {
    let mut all_rows = get_all(file_path, sheet_name)?;
    let original_len = all_rows.len();
    all_rows.retain(|row| row.get("id").map(|v| v != id).unwrap_or(true));
    if all_rows.len() == original_len { return Ok(false); }
    write_sheet_data(file_path, sheet_name, &all_rows)?;
    Ok(true)
}

fn write_sheet_data(file_path: &str, sheet_name: &str, rows: &[HashMap<String, String>]) -> Result<(), String> {
    let mut all_sheets_data: Vec<(String, Vec<HashMap<String, String>>)> = Vec::new();
    if Path::new(file_path).exists() {
        let mut wb = open_workbook_auto(file_path).map_err(|e| e.to_string())?;
        for &(sname, _) in SHEETS {
            if let Ok(range) = wb.worksheet_range(sname) {
                let keys = get_column_keys(sname);
                let mut sheet_rows = Vec::new();
                for (row_idx, row) in range.rows().enumerate() {
                    if row_idx == 0 { continue; }
                    let mut item = HashMap::new();
                    for (col_idx, cell) in row.iter().enumerate() {
                        if col_idx < keys.len() {
                            item.insert(keys[col_idx].to_string(), cell_to_string(cell));
                        }
                    }
                    sheet_rows.push(item);
                }
                all_sheets_data.push((sname.to_string(), sheet_rows));
            } else {
                all_sheets_data.push((sname.to_string(), vec![]));
            }
        }
    } else {
        for &(sname, _) in SHEETS {
            all_sheets_data.push((sname.to_string(), vec![]));
        }
    }
    for (name, data) in &mut all_sheets_data {
        if name == sheet_name { *data = rows.to_vec(); }
    }
    let mut workbook = Workbook::new();
    for &(sname, cols) in SHEETS {
        let sheet = workbook.add_worksheet()
            .set_name(sname)
            .map_err(|e| e.to_string())?;
        for (i, &(header, _)) in cols.iter().enumerate() {
            sheet.write_string(0, i as u16, header).map_err(|e| e.to_string())?;
        }
        if let Some((_, sheet_data)) = all_sheets_data.iter().find(|(n, _)| n == sname) {
            for (row_idx, row) in sheet_data.iter().enumerate() {
                for (col_idx, &(_, key)) in cols.iter().enumerate() {
                    if let Some(value) = row.get(key) {
                        sheet.write_string((row_idx + 1) as u32, col_idx as u16, value).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    workbook.save(file_path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn export_data(file_path: &str) -> Result<Vec<u8>, String> {
    if !Path::new(file_path).exists() {
        return Err("数据文件不存在".to_string());
    }
    fs::read(file_path).map_err(|e| e.to_string())
}

pub fn import_data(file_path: &str, data: &[u8]) -> Result<HashMap<String, i64>, String> {
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if Path::new(file_path).exists() {
        let backup = file_path.replace(".xlsx", &format!("-backup-{}.xlsx", Local::now().timestamp_millis()));
        let _ = fs::copy(file_path, &backup);
    }
    fs::write(file_path, data).map_err(|e| e.to_string())?;
    let mut stats = HashMap::new();
    let mut wb = open_workbook_auto(file_path).map_err(|e| e.to_string())?;
    for name in wb.sheet_names() {
        if let Ok(range) = wb.worksheet_range(&name) {
            let count = range.rows().count().saturating_sub(1) as i64;
            stats.insert(name.clone(), count);
        }
    }
    Ok(stats)
}

pub fn calculate_stock(file_path: &str, stock_type: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let (info_sheet, order_sheet, use_sheet, stock_sheet, item_id_key) = match stock_type {
        "punch" => ("冲头信息表", "冲头订购记录", "冲头领用记录", "冲头库存汇总", "punchId"),
        "die" => ("牙板信息表", "牙板订购记录", "牙板领用记录", "牙板库存汇总", "dieId"),
        "belt" => ("皮带信息表", "皮带订购记录", "皮带使用记录", "皮带库存汇总", "beltId"),
        "mainMold" => ("主模具信息表", "主模具订购记录", "主模具使用记录", "主模具库存汇总", "mainMoldId"),
        "scissor" => ("剪刀信息表", "剪刀订购记录", "剪刀使用记录", "剪刀库存汇总", "scissorId"),
        "upperPunch" => ("上冲信息表", "上冲订购记录", "上冲使用记录", "上冲库存汇总", "upperPunchId"),
        _ => return Err("未知类型".to_string()),
    };
    let info_items = get_all(file_path, info_sheet)?;
    let orders = get_all(file_path, order_sheet)?;
    let uses = get_all(file_path, use_sheet)?;
    let stock_data: Vec<HashMap<String, String>> = info_items.iter().map(|item| {
        let item_id = item.get("id").cloned().unwrap_or_default();
        let total_ordered: i64 = orders.iter()
            .filter(|o| o.get(item_id_key).map(|v| v == &item_id).unwrap_or(false)
                     && o.get("status").map(|v| v == "已到货").unwrap_or(false))
            .filter_map(|o| o.get("quantity").and_then(|q| q.parse::<i64>().ok()))
            .sum();
        let total_used: i64 = uses.iter()
            .filter(|u| u.get(item_id_key).map(|v| v == &item_id).unwrap_or(false))
            .filter_map(|u| u.get("quantity").and_then(|q| q.parse::<i64>().ok()))
            .sum();
        let current_stock = total_ordered - total_used;
        let safety_stock: i64 = item.get("safety_stock")
            .or_else(|| item.get("safetyStock"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mut row = HashMap::new();
        row.insert(item_id_key.to_string(), item_id);
        row.insert("name".to_string(), item.get("name").cloned().unwrap_or_default());
        row.insert("currentStock".to_string(), current_stock.to_string());
        row.insert("safetyStock".to_string(), safety_stock.to_string());
        row.insert("status".to_string(), if current_stock < safety_stock { "需订购".to_string() } else { "安全".to_string() });
        row
    }).collect();
    write_sheet_data(file_path, stock_sheet, &stock_data)?;
    Ok(stock_data)
}

pub fn get_default_file_path() -> String {
    let base = if cfg!(debug_assertions) {
        std::env::current_dir()
            .map(|p| p.join("..").join("data").to_string_lossy().to_string())
            .unwrap_or_else(|_| "./data".to_string())
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("data").to_string_lossy().to_string()))
            .unwrap_or_else(|| "./data".to_string())
    };
    format!("{}/mold-data.xlsx", base)
}
