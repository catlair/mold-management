use crate::{attachments, excel, storage};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const BACKUP_FORMAT: &str = "mold-management-backup";
const BACKUP_VERSION: u32 = 1;
const MANIFEST_NAME: &str = "backup-manifest.json";
const ATTACHMENT_MAP_NAME: &str = "attachment-map.json";
const WORKBOOK_NAME: &str = "mold-data.xlsx";
const MAX_BACKUP_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 100 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_BACKUP_ENTRIES: usize = 10_000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    format: String,
    version: u32,
    created_at: String,
    reason: String,
    workbook: String,
    attachments: String,
}

/// 把 Excel 与全部附件打包成一个 ZIP 备份文件内容。
pub fn create_backup_zip(
    data_file_path: &str,
    reason: &str,
    created_at: &str,
) -> Result<Vec<u8>, String> {
    let workbook_path = Path::new(data_file_path);
    excel::validate_workbook(workbook_path)?;
    let workbook_bytes = fs::metadata(workbook_path)
        .map_err(|e| format!("读取数据文件大小失败「{}」: {}", workbook_path.display(), e))?
        .len();
    if workbook_bytes > MAX_ENTRY_BYTES {
        return Err("数据文件超过备份单文件 100MB 限制".to_string());
    }

    let attachment_root = attachments::root_path(data_file_path)?;
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    let mut attachment_map = HashMap::<String, String>::new();
    if attachment_root.exists() {
        attachments::validate_root(&attachment_root)?;
        let index_json = attachment_root.join("index.json");
        if index_json.is_file() {
            entries.push(("attachments/index.json".to_string(), index_json));
        }
        // 附件条目用「螺丝ID_螺丝名_设置名」命名（重名自动加序号），
        // 同时记录映射，恢复时按映射放回原相对路径，附件索引无需改动。
        let screw_names: HashMap<String, String> = excel::get_all(data_file_path, "螺丝规格表")
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| {
                row.get("id").cloned().map(|id| {
                    let name = row.get("name").cloned().unwrap_or_default();
                    (id, name)
                })
            })
            .collect();
        let mut used_names = HashSet::new();
        for meta in attachments::load_all(data_file_path)? {
            let dir = meta
                .relative_path
                .split('/')
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or("attachments");
            let screw_name = screw_names
                .get(&meta.screw_spec_id)
                .cloned()
                .unwrap_or_default();
            let entry_file = attachments::build_attachment_file_name(
                &meta.screw_spec_id,
                &screw_name,
                &meta.display_name,
                None,
            );
            let zip_name = unique_attachment_name(dir, &entry_file, &mut used_names);
            let source = attachment_root.join(&meta.relative_path);
            if !source.is_file() {
                return Err(format!("附件文件缺失「{}」", source.display()));
            }
            attachment_map.insert(zip_name.clone(), meta.relative_path.clone());
            entries.push((zip_name, source));
        }
    }
    if entries.len() + 3 > MAX_BACKUP_ENTRIES {
        return Err(format!("备份条目过多: {}", entries.len() + 3));
    }
    let mut total_bytes = workbook_bytes;
    for (_, path) in &entries {
        let size = fs::metadata(path)
            .map_err(|e| format!("读取附件大小失败「{}」: {}", path.display(), e))?
            .len();
        if size > MAX_ENTRY_BYTES {
            return Err(format!(
                "附件超过备份单文件 100MB 限制「{}」",
                path.display()
            ));
        }
        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| "备份数据量溢出".to_string())?;
        if total_bytes > MAX_EXTRACTED_BYTES {
            return Err("备份数据总量不能超过 2GB".to_string());
        }
    }

    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o600);

    let manifest = BackupManifest {
        format: BACKUP_FORMAT.to_string(),
        version: BACKUP_VERSION,
        created_at: created_at.to_string(),
        reason: reason.to_string(),
        workbook: WORKBOOK_NAME.to_string(),
        attachments: "attachments".to_string(),
    };
    archive
        .start_file(MANIFEST_NAME, options)
        .map_err(|e| format!("创建备份清单失败: {}", e))?;
    archive
        .write_all(
            &serde_json::to_vec_pretty(&manifest)
                .map_err(|e| format!("序列化备份清单失败: {}", e))?,
        )
        .map_err(|e| format!("写入备份清单失败: {}", e))?;

    archive
        .start_file(ATTACHMENT_MAP_NAME, options)
        .map_err(|e| format!("创建附件映射清单失败: {}", e))?;
    archive
        .write_all(
            &serde_json::to_vec(&attachment_map)
                .map_err(|e| format!("序列化附件映射清单失败: {}", e))?,
        )
        .map_err(|e| format!("写入附件映射清单失败: {}", e))?;

    archive
        .start_file(WORKBOOK_NAME, options)
        .map_err(|e| format!("创建备份 Excel 项失败: {}", e))?;
    let mut workbook = File::open(workbook_path)
        .map_err(|e| format!("打开数据文件失败「{}」: {}", workbook_path.display(), e))?;
    std::io::copy(&mut workbook, &mut archive)
        .map_err(|e| format!("写入备份 Excel 失败: {}", e))?;

    for (archive_name, path) in entries {
        archive
            .start_file(archive_name, options)
            .map_err(|e| format!("创建备份附件项失败「{}」: {}", path.display(), e))?;
        let mut source =
            File::open(&path).map_err(|e| format!("打开附件失败「{}」: {}", path.display(), e))?;
        std::io::copy(&mut source, &mut archive)
            .map_err(|e| format!("写入备份附件失败「{}」: {}", path.display(), e))?;
    }

    let cursor = archive
        .finish()
        .map_err(|e| format!("完成备份失败: {}", e))?;
    let bytes = cursor.into_inner();
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err("备份压缩后不能超过 1GB".to_string());
    }
    Ok(bytes)
}

/// 轻量校验备份 ZIP：可打开、清单合法、Excel 条目存在。
pub fn validate_backup_zip(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("读取备份失败「{}」: {}", path.display(), e))?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err("备份文件超过 1GB".to_string());
    }
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开备份失败: {}", e))?;
    if archive.len() > MAX_BACKUP_ENTRIES {
        return Err(format!("备份条目过多: {}", archive.len()));
    }
    let mut has_manifest = false;
    let mut has_workbook = false;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| format!("读取备份条目失败: {}", e))?;
        if entry.size() > MAX_ENTRY_BYTES {
            return Err(format!("备份条目过大「{}」", entry.name()));
        }
        let relative = crate::data_package::safe_relative_path(entry.name())?;
        let allowed = relative == Path::new(MANIFEST_NAME)
            || relative == Path::new(ATTACHMENT_MAP_NAME)
            || relative == Path::new(WORKBOOK_NAME)
            || relative.starts_with("attachments");
        if !allowed {
            return Err(format!("备份包含未知条目「{}」", entry.name()));
        }
        if relative == Path::new(MANIFEST_NAME) {
            if entry.is_dir() {
                return Err("备份清单不能是目录".to_string());
            }
            if entry.size() > MAX_MANIFEST_BYTES {
                return Err("备份清单不能超过 1MB".to_string());
            }
            has_manifest = true;
        } else if relative == Path::new(ATTACHMENT_MAP_NAME) {
            if entry.is_dir() || entry.size() > MAX_MANIFEST_BYTES {
                return Err("备份附件映射清单无效".to_string());
            }
        } else if relative == Path::new(WORKBOOK_NAME) {
            if entry.is_dir() {
                return Err("备份 Excel 不能是目录".to_string());
            }
            has_workbook = true;
        }
    }
    if !has_manifest {
        return Err("备份缺少 backup-manifest.json".to_string());
    }
    if !has_workbook {
        return Err("备份缺少 mold-data.xlsx".to_string());
    }
    Ok(())
}

/// 从备份 ZIP 恢复：安全解压到暂存目录，校验后联合替换 Excel 与附件，失败回滚。
/// 返回附件是否被恢复（备份中没有附件快照时为 false）。
pub fn restore_backup_zip(backup_path: &Path, data_file_path: &str) -> Result<bool, String> {
    validate_backup_zip(backup_path)?;
    let bytes = fs::read(backup_path)
        .map_err(|e| format!("读取备份失败「{}」: {}", backup_path.display(), e))?;

    let data_path = Path::new(data_file_path);
    let parent = data_path
        .parent()
        .ok_or_else(|| "无法确定数据文件目录".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("创建数据目录失败「{}」: {}", parent.display(), e))?;
    let staging = parent.join(format!(".mold-backup-restore-{}", Uuid::new_v4()));
    let staged_workbook = staging.join(WORKBOOK_NAME);
    let staged_attachments = staging.join("attachments");

    let result = (|| {
        fs::create_dir_all(&staging).map_err(|e| format!("创建备份暂存目录失败: {}", e))?;
        extract_backup(&bytes, &staging)?;
        // 新版备份的附件按可读文件名打包；按映射清单还原到原相对路径，保持附件索引一致。
        apply_attachment_map(&staging)?;
        let manifest_path = staging.join(MANIFEST_NAME);
        let manifest: BackupManifest = serde_json::from_slice(
            &fs::read(&manifest_path).map_err(|e| format!("读取备份清单失败: {}", e))?,
        )
        .map_err(|e| format!("解析备份清单失败: {}", e))?;
        if manifest.format != BACKUP_FORMAT || manifest.version != BACKUP_VERSION {
            return Err(format!("不支持的备份格式或版本: {}", manifest.version));
        }
        if manifest.workbook != WORKBOOK_NAME || manifest.attachments != "attachments" {
            return Err("备份清单中的数据路径无效".to_string());
        }
        excel::validate_workbook(&staged_workbook)?;
        let has_attachments = staged_attachments.exists();
        if has_attachments {
            attachments::validate_root(&staged_attachments)?;
        }

        let staged_excel_copy = storage::temporary_path(data_path, "xlsx")?;
        fs::copy(&staged_workbook, &staged_excel_copy).map_err(|e| {
            format!(
                "暂存备份 Excel 失败「{}」: {}",
                staged_excel_copy.display(),
                e
            )
        })?;
        storage::sync_file(&staged_excel_copy)?;

        let previous_excel = storage::temporary_path(data_path, "previous.xlsx")?;
        if data_path.exists() {
            fs::copy(data_path, &previous_excel).map_err(|e| {
                format!("暂存当前 Excel 失败「{}」: {}", previous_excel.display(), e)
            })?;
            storage::sync_file(&previous_excel)?;
        }
        storage::replace_file(&staged_excel_copy, data_path)?;

        if has_attachments {
            let target_attachments = attachments::root_path(data_file_path)?;
            if let Err(error) =
                crate::data_package::replace_directory(&staged_attachments, &target_attachments)
            {
                if previous_excel.exists() {
                    storage::replace_file(&previous_excel, data_path).map_err(
                        |rollback_error| {
                            format!("{}；同时回滚 Excel 失败: {}", error, rollback_error)
                        },
                    )?;
                }
                return Err(error);
            }
        }

        if previous_excel.exists() {
            if let Err(error) = fs::remove_file(&previous_excel) {
                eprintln!(
                    "备份已恢复，但清理 Excel 事务暂存文件失败「{}」: {}",
                    previous_excel.display(),
                    error
                );
            }
        }
        Ok(has_attachments)
    })();

    let _ = fs::remove_dir_all(&staging);
    result
}

fn extract_backup(data: &[u8], target: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开备份失败: {}", e))?;
    if archive.len() > MAX_BACKUP_ENTRIES {
        return Err(format!("备份条目过多: {}", archive.len()));
    }
    let mut extracted_bytes = 0_u64;
    let mut extracted_paths = HashSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| format!("读取备份条目失败: {}", e))?;
        if entry.size() > MAX_ENTRY_BYTES {
            return Err(format!("备份条目过大「{}」", entry.name()));
        }
        extracted_bytes = extracted_bytes
            .checked_add(entry.size())
            .ok_or_else(|| "备份展开大小溢出".to_string())?;
        if extracted_bytes > MAX_EXTRACTED_BYTES {
            return Err("备份展开后不能超过 2GB".to_string());
        }
        let relative = crate::data_package::safe_relative_path(entry.name())?;
        let allowed = relative == Path::new(MANIFEST_NAME)
            || relative == Path::new(ATTACHMENT_MAP_NAME)
            || relative == Path::new(WORKBOOK_NAME)
            || relative.starts_with("attachments");
        if !allowed {
            return Err(format!("备份包含未知条目「{}」", entry.name()));
        }
        let normalized = relative.to_string_lossy().to_ascii_lowercase();
        if !extracted_paths.insert(normalized) {
            return Err(format!("备份包含重复条目「{}」", entry.name()));
        }
        let destination = target.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|e| format!("创建备份目录失败「{}」: {}", destination.display(), e))?;
            continue;
        }
        if let Some(destination_parent) = destination.parent() {
            fs::create_dir_all(destination_parent).map_err(|e| {
                format!(
                    "创建备份目录失败「{}」: {}",
                    destination_parent.display(),
                    e
                )
            })?;
        }
        let mut output = File::create(&destination)
            .map_err(|e| format!("创建备份暂存文件失败「{}」: {}", destination.display(), e))?;
        let mut reader = entry;
        std::io::copy(&mut reader, &mut output)
            .map_err(|e| format!("解压备份条目失败「{}」: {}", reader.name(), e))?;
        output
            .sync_all()
            .map_err(|e| format!("刷新备份暂存文件失败「{}」: {}", destination.display(), e))?;
    }
    Ok(())
}

/// 生成可读的 ZIP 附件条目名：attachments/<目录>/<文件名>；同目录重名自动加序号。
fn unique_attachment_name(dir: &str, file_name: &str, used: &mut HashSet<String>) -> String {
    let cleaned = sanitize_file_name(file_name);
    let base = format!("attachments/{}/{}", dir, cleaned);
    if used.insert(base.clone()) {
        return base;
    }
    let (stem, extension) = match cleaned.rfind('.') {
        Some(index) if index > 0 => {
            let (head, tail) = cleaned.split_at(index);
            (head.to_string(), tail.to_string())
        }
        _ => (cleaned.clone(), String::new()),
    };
    let mut counter = 1;
    loop {
        let candidate = if extension.is_empty() {
            format!("attachments/{}/{} ({})", dir, stem, counter)
        } else {
            format!("attachments/{}/{} ({}){}", dir, stem, counter, extension)
        };
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

/// 清洗文件名中的路径分隔与 Windows 非法字符，防止 ZIP 条目路径逃逸或无法落盘。
fn sanitize_file_name(file_name: &str) -> String {
    let cleaned: String = file_name
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => ch,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "attachment".to_string()
    } else {
        trimmed
    }
}

/// 按附件映射清单，把暂存目录中以可读文件名解压的附件还原到原相对路径。
/// 旧版备份没有映射清单时直接跳过（条目本身即原相对路径）。
fn apply_attachment_map(staging: &Path) -> Result<(), String> {
    let map_path = staging.join(ATTACHMENT_MAP_NAME);
    if !map_path.is_file() {
        return Ok(());
    }
    let content =
        fs::read_to_string(&map_path).map_err(|e| format!("读取附件映射清单失败: {}", e))?;
    let map: HashMap<String, String> =
        serde_json::from_str(&content).map_err(|e| format!("解析附件映射清单失败: {}", e))?;
    let attachments_dir = staging.join("attachments");
    for (zip_name, relative_path) in &map {
        let source = staging.join(zip_name);
        let destination = attachments_dir.join(relative_path);
        if !source.is_file() {
            return Err(format!("备份附件缺失「{}」", zip_name));
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建附件目录失败「{}」: {}", parent.display(), e))?;
        }
        fs::rename(&source, &destination).map_err(|e| {
            format!(
                "还原备份附件失败「{}」→「{}」: {}",
                source.display(),
                destination.display(),
                e
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use rust_xlsxwriter::Workbook;

    fn create_test_workbook(path: &Path) {
        let mut workbook = Workbook::new();
        for &(sheet_name, columns) in excel::SHEETS {
            let sheet = workbook.add_worksheet();
            sheet.set_name(sheet_name).unwrap();
            for (index, &(header, _)) in columns.iter().enumerate() {
                sheet.write_string(0, index as u16, header).unwrap();
            }
        }
        workbook.save(path).unwrap();
    }

    #[test]
    fn backup_zip_round_trip_restores_excel_and_attachments() {
        let root = std::env::temp_dir().join(format!("mold-backup-zip-test-{}", Uuid::new_v4()));
        let source_dir = root.join("source");
        let target_dir = root.join("target");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        let source_excel = source_dir.join(WORKBOOK_NAME);
        let target_excel = target_dir.join(WORKBOOK_NAME);
        create_test_workbook(&source_excel);
        create_test_workbook(&target_excel);

        // 构造真实附件索引：物理文件名为 UUID，原始文件名为 test.png。
        let attachment_id = Uuid::new_v4().to_string();
        let relative_path = format!("LS0001/{}.png", attachment_id);
        let attachment_dir = source_dir.join("attachments").join("LS0001");
        fs::create_dir_all(&attachment_dir).unwrap();
        fs::write(
            attachment_dir.join(format!("{}.png", attachment_id)),
            b"attachment-data",
        )
        .unwrap();
        let meta = crate::attachments::ScrewAttachment {
            id: attachment_id,
            screw_spec_id: "LS0001".to_string(),
            display_name: "规格图纸.png".to_string(),
            file_name: "test.png".to_string(),
            mime_type: "image/png".to_string(),
            size: 15,
            relative_path: relative_path.clone(),
            annotations: vec![],
            sort_order: 0,
            created_at: "2026-08-03 00:00:00".to_string(),
            updated_at: "2026-08-03 00:00:00".to_string(),
        };
        fs::write(
            source_dir.join("attachments").join("index.json"),
            serde_json::to_vec(&vec![meta]).unwrap(),
        )
        .unwrap();

        let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let bytes =
            create_backup_zip(source_excel.to_str().unwrap(), "测试备份", &created_at).unwrap();
        let zip_path = root.join("mold-backup-20260803_204014_123.zip");
        fs::write(&zip_path, bytes).unwrap();
        validate_backup_zip(&zip_path).unwrap();

        // ZIP 内附件使用「螺丝ID_螺丝名_设置名」可读命名，而不是 UUID 物理名。
        let cursor = std::io::Cursor::new(fs::read(&zip_path).unwrap());
        let mut archive = ZipArchive::new(cursor).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|index| archive.by_index(index).unwrap().name().to_string())
            .collect();
        let attachment_entries: Vec<&String> = names
            .iter()
            .filter(|name| name.starts_with("attachments/LS0001/"))
            .collect();
        assert_eq!(attachment_entries.len(), 1);
        assert!(attachment_entries[0].starts_with("attachments/LS0001/LS0001_"));
        assert!(attachment_entries[0].ends_with("规格图纸.png"));
        assert!(!names.contains(&format!("attachments/{}", relative_path)));
        assert!(names.contains(&"attachment-map.json".to_string()));

        let restored = restore_backup_zip(&zip_path, target_excel.to_str().unwrap()).unwrap();
        assert!(restored);
        // 恢复后附件还原到原相对路径，附件索引保持一致。
        assert_eq!(
            fs::read(target_dir.join("attachments").join(&relative_path)).unwrap(),
            b"attachment-data"
        );
        excel::validate_workbook(&target_excel).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn attachment_file_name_combines_screw_id_name_and_display_name() {
        assert_eq!(
            attachments::build_attachment_file_name(
                "LS0001",
                "M3*8自攻",
                "规格图纸.png",
                Some("png")
            ),
            // Windows 非法字符 * 被替换为 _
            "LS0001_M3_8自攻_规格图纸.png"
        );
        // 显示名不带扩展名时追加原扩展名
        assert_eq!(
            attachments::build_attachment_file_name("LS0001", "M3*8自攻", "图纸", Some("png")),
            "LS0001_M3_8自攻_图纸.png"
        );
        // 非法字符被清洗，结果不含路径分隔符
        let cleaned = attachments::build_attachment_file_name("A/B", "x:y", "a<b>c", Some("png"));
        assert!(!cleaned.contains('/') && !cleaned.contains(':') && !cleaned.contains('<'));
        assert!(cleaned.ends_with(".png"));
    }

    #[test]
    fn backup_zip_without_attachments_keeps_current_attachments() {
        let root =
            std::env::temp_dir().join(format!("mold-backup-zip-noattach-{}", Uuid::new_v4()));
        let source_dir = root.join("source");
        let target_dir = root.join("target");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        let source_excel = source_dir.join(WORKBOOK_NAME);
        let target_excel = target_dir.join(WORKBOOK_NAME);
        create_test_workbook(&source_excel);
        create_test_workbook(&target_excel);
        fs::write(target_dir.join("keep.txt"), b"keep").unwrap();

        let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let bytes =
            create_backup_zip(source_excel.to_str().unwrap(), "测试备份", &created_at).unwrap();
        let zip_path = root.join("mold-backup-20260803_204014_456.zip");
        fs::write(&zip_path, bytes).unwrap();

        let restored = restore_backup_zip(&zip_path, target_excel.to_str().unwrap()).unwrap();
        assert!(!restored);
        assert!(target_dir.join("keep.txt").is_file());
        excel::validate_workbook(&target_excel).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
