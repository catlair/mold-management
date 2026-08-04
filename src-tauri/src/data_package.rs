use crate::{attachments, excel, storage};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const PACKAGE_VERSION: u32 = 1;
const MANIFEST_NAME: &str = "manifest.json";
const WORKBOOK_NAME: &str = "mold-data.db";
const ATTACHMENT_PREFIX: &str = "attachments/";
const MAX_PACKAGE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 100 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_PACKAGE_ENTRIES: usize = 10_000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageManifest {
    format: String,
    version: u32,
    created_at: String,
    workbook: String,
    attachments: String,
}

pub fn export_package(data_file_path: &str) -> Result<Vec<u8>, String> {
    let workbook_path = Path::new(data_file_path);
    excel::validate_workbook(workbook_path)?;
    let workbook_bytes = fs::metadata(workbook_path)
        .map_err(|e| format!("读取数据文件大小失败「{}」: {}", workbook_path.display(), e))?
        .len();
    if workbook_bytes > MAX_ENTRY_BYTES {
        return Err("数据文件超过完整数据包单文件 100MB 限制".to_string());
    }

    let attachment_root = attachments::root_path(data_file_path)?;
    let mut files = Vec::new();
    if attachment_root.exists() {
        attachments::validate_root(&attachment_root)?;
        collect_files(&attachment_root, &mut files)?;
        files.sort();
    }
    if files.len() + 2 > MAX_PACKAGE_ENTRIES {
        return Err(format!("完整数据包条目过多: {}", files.len() + 2));
    }
    let mut total_bytes = workbook_bytes;
    for path in &files {
        let size = fs::metadata(path)
            .map_err(|e| format!("读取附件大小失败「{}」: {}", path.display(), e))?
            .len();
        if size > MAX_ENTRY_BYTES {
            return Err(format!(
                "附件超过完整数据包单文件 100MB 限制「{}」",
                path.display()
            ));
        }
        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| "完整数据包数据量溢出".to_string())?;
        if total_bytes > MAX_EXTRACTED_BYTES {
            return Err("完整数据包数据总量不能超过 2GB".to_string());
        }
    }

    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o600);

    let manifest = PackageManifest {
        format: "mold-management-data-package".to_string(),
        version: PACKAGE_VERSION,
        created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        workbook: WORKBOOK_NAME.to_string(),
        attachments: "attachments".to_string(),
    };
    archive
        .start_file(MANIFEST_NAME, options)
        .map_err(|e| format!("创建数据包清单失败: {}", e))?;
    archive
        .write_all(
            &serde_json::to_vec_pretty(&manifest)
                .map_err(|e| format!("序列化数据包清单失败: {}", e))?,
        )
        .map_err(|e| format!("写入数据包清单失败: {}", e))?;

    archive
        .start_file(WORKBOOK_NAME, options)
        .map_err(|e| format!("创建数据包 Excel 项失败: {}", e))?;
    let mut workbook = File::open(workbook_path)
        .map_err(|e| format!("打开数据文件失败「{}」: {}", workbook_path.display(), e))?;
    std::io::copy(&mut workbook, &mut archive)
        .map_err(|e| format!("写入数据包 Excel 失败: {}", e))?;

    for path in files {
        let relative = path
            .strip_prefix(&attachment_root)
            .map_err(|e| format!("计算附件相对路径失败: {}", e))?;
        let archive_name = format!(
            "{}{}",
            ATTACHMENT_PREFIX,
            relative.to_string_lossy().replace('\\', "/")
        );
        archive
            .start_file(archive_name, options)
            .map_err(|e| format!("创建数据包附件项失败「{}」: {}", path.display(), e))?;
        let mut source =
            File::open(&path).map_err(|e| format!("打开附件失败「{}」: {}", path.display(), e))?;
        std::io::copy(&mut source, &mut archive)
            .map_err(|e| format!("写入数据包附件失败「{}」: {}", path.display(), e))?;
    }

    let cursor = archive
        .finish()
        .map_err(|e| format!("完成数据包失败: {}", e))?;
    let bytes = cursor.into_inner();
    if bytes.len() as u64 > MAX_PACKAGE_BYTES {
        return Err("完整数据包压缩后不能超过 1GB".to_string());
    }
    Ok(bytes)
}

pub fn import_package(data_file_path: &str, data: &[u8]) -> Result<serde_json::Value, String> {
    if data.len() as u64 > MAX_PACKAGE_BYTES {
        return Err("完整数据包不能超过 1GB".to_string());
    }

    let data_path = Path::new(data_file_path);
    let parent = data_path
        .parent()
        .ok_or_else(|| "无法确定数据文件目录".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("创建数据目录失败「{}」: {}", parent.display(), e))?;
    let staging = parent.join(format!(".mold-package-import-{}", Uuid::new_v4()));
    let staged_workbook = staging.join(WORKBOOK_NAME);
    let staged_attachments = staging.join("attachments");
    let result = (|| {
        fs::create_dir_all(&staging).map_err(|e| format!("创建数据包暂存目录失败: {}", e))?;
        extract_package(data, &staging)?;
        let manifest_path = staging.join(MANIFEST_NAME);
        let manifest: PackageManifest = serde_json::from_slice(
            &fs::read(&manifest_path).map_err(|e| format!("读取数据包清单失败: {}", e))?,
        )
        .map_err(|e| format!("解析数据包清单失败: {}", e))?;
        if manifest.format != "mold-management-data-package" || manifest.version != PACKAGE_VERSION
        {
            return Err(format!("不支持的数据包格式或版本: {}", manifest.version));
        }
        if manifest.workbook != WORKBOOK_NAME || manifest.attachments != "attachments" {
            return Err("数据包清单中的数据路径无效".to_string());
        }
        excel::validate_workbook(&staged_workbook)?;
        attachments::validate_root(&staged_attachments)?;

        let stats = excel::workbook_stats(&staged_workbook)?;
        let attachment_count = attachments::count_index_entries(&staged_attachments)?;

        let staged_excel_copy = storage::temporary_path(data_path, "xlsx")?;
        fs::copy(&staged_workbook, &staged_excel_copy).map_err(|e| {
            format!(
                "暂存数据包 Excel 失败「{}」: {}",
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

        let target_attachments = attachments::root_path(data_file_path)?;
        if let Err(error) = replace_directory(&staged_attachments, &target_attachments) {
            if previous_excel.exists() {
                storage::replace_file(&previous_excel, data_path).map_err(|rollback_error| {
                    format!("{}；同时回滚 Excel 失败: {}", error, rollback_error)
                })?;
            }
            return Err(error);
        }
        if previous_excel.exists() {
            if let Err(error) = fs::remove_file(&previous_excel) {
                eprintln!(
                    "完整数据包已导入，但清理 Excel 事务暂存文件失败「{}」: {}",
                    previous_excel.display(),
                    error
                );
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "stats": stats,
            "attachmentCount": attachment_count,
        }))
    })();
    let _ = fs::remove_dir_all(&staging);
    result
}

fn extract_package(data: &[u8], target: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开完整数据包失败: {}", e))?;
    if archive.len() > MAX_PACKAGE_ENTRIES {
        return Err(format!("完整数据包条目过多: {}", archive.len()));
    }
    let mut extracted_bytes = 0_u64;
    let mut extracted_paths = HashSet::new();
    let mut has_manifest = false;
    let mut has_workbook = false;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("读取数据包条目失败: {}", e))?;
        if entry.size() > MAX_ENTRY_BYTES {
            return Err(format!("数据包条目过大「{}」", entry.name()));
        }
        extracted_bytes = extracted_bytes
            .checked_add(entry.size())
            .ok_or_else(|| "数据包展开大小溢出".to_string())?;
        if extracted_bytes > MAX_EXTRACTED_BYTES {
            return Err("完整数据包展开后不能超过 2GB".to_string());
        }
        let relative = safe_relative_path(entry.name())?;
        let allowed = relative == Path::new(MANIFEST_NAME)
            || relative == Path::new(WORKBOOK_NAME)
            || relative.starts_with("attachments");
        if !allowed {
            return Err(format!("数据包包含未知条目「{}」", entry.name()));
        }
        let normalized_path = relative.to_string_lossy().to_ascii_lowercase();
        if !extracted_paths.insert(normalized_path) {
            return Err(format!("数据包包含重复条目「{}」", entry.name()));
        }
        if relative == Path::new(MANIFEST_NAME) {
            if entry.is_dir() {
                return Err("数据包清单不能是目录".to_string());
            }
            if entry.size() > MAX_MANIFEST_BYTES {
                return Err("数据包清单不能超过 1MB".to_string());
            }
            has_manifest = true;
        } else if relative == Path::new(WORKBOOK_NAME) {
            if entry.is_dir() {
                return Err("数据包 Excel 不能是目录".to_string());
            }
            has_workbook = true;
        }
        let destination = target.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|e| format!("创建数据包目录失败「{}」: {}", destination.display(), e))?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建数据包目录失败「{}」: {}", parent.display(), e))?;
        }
        let mut output = File::create(&destination)
            .map_err(|e| format!("创建数据包暂存文件失败「{}」: {}", destination.display(), e))?;
        std::io::copy(&mut entry, &mut output)
            .map_err(|e| format!("解压数据包条目失败「{}」: {}", entry.name(), e))?;
        output
            .sync_all()
            .map_err(|e| format!("刷新数据包暂存文件失败「{}」: {}", destination.display(), e))?;
    }
    if !has_manifest {
        return Err("完整数据包缺少 manifest.json".to_string());
    }
    if !has_workbook {
        return Err("完整数据包缺少 mold-data.xlsx".to_string());
    }
    Ok(())
}

pub(crate) fn safe_relative_path(name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(format!("数据包包含绝对路径「{}」", name));
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => safe.push(segment),
            Component::CurDir => {}
            _ => return Err(format!("数据包包含不安全路径「{}」", name)),
        }
    }
    if safe.as_os_str().is_empty() {
        return Err("数据包包含空路径".to_string());
    }
    Ok(safe)
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|e| format!("读取附件目录失败「{}」: {}", directory.display(), e))?
    {
        let path = entry
            .map_err(|e| format!("读取附件目录项失败: {}", e))?
            .path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

pub(crate) fn replace_directory(source: &Path, target: &Path) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("无法确定附件目录父路径「{}」", target.display()))?;
    let incoming = parent.join(format!(".attachments-incoming-{}", Uuid::new_v4()));
    let previous = parent.join(format!(".attachments-previous-{}", Uuid::new_v4()));
    if let Err(error) = copy_directory(source, &incoming) {
        let _ = fs::remove_dir_all(&incoming);
        return Err(error);
    }

    if target.exists() {
        fs::rename(target, &previous)
            .map_err(|e| format!("暂存当前附件目录失败「{}」: {}", target.display(), e))?;
    }
    if let Err(error) = fs::rename(&incoming, target) {
        let rollback_error = if previous.exists() {
            fs::rename(&previous, target).err()
        } else {
            None
        };
        let _ = fs::remove_dir_all(&incoming);
        return match rollback_error {
            Some(rollback_error) => Err(format!(
                "替换附件目录失败「{}」: {}；回滚原附件目录也失败: {}",
                target.display(),
                error,
                rollback_error
            )),
            None => Err(format!(
                "替换附件目录失败「{}」: {}",
                target.display(),
                error
            )),
        };
    }
    if previous.exists() {
        if let Err(error) = fs::remove_dir_all(&previous) {
            eprintln!(
                "附件目录已替换，但清理旧附件暂存目录失败「{}」: {}",
                previous.display(),
                error
            );
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|e| format!("创建附件暂存目录失败「{}」: {}", target.display(), e))?;
    if !source.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(source)
        .map_err(|e| format!("读取附件目录失败「{}」: {}", source.display(), e))?
    {
        let entry = entry.map_err(|e| format!("读取附件目录项失败: {}", e))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)
                .map_err(|e| format!("复制附件失败「{}」: {}", source_path.display(), e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_database(path: &Path) {
        let conn = crate::db::connect(&path.to_string_lossy()).unwrap();
        crate::db::init_schema(&conn).unwrap();
    }

    #[test]
    fn package_round_trip_preserves_excel_and_attachments() {
        let root = std::env::temp_dir().join(format!("mold-package-test-{}", Uuid::new_v4()));
        let source_dir = root.join("source");
        let target_dir = root.join("target");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        let source_excel = source_dir.join(WORKBOOK_NAME);
        let target_excel = target_dir.join(WORKBOOK_NAME);
        create_test_database(&source_excel);
        create_test_database(&target_excel);

        let attachment_dir = source_dir.join("attachments").join("LS0001");
        fs::create_dir_all(&attachment_dir).unwrap();
        fs::write(attachment_dir.join("test.png"), b"attachment-data").unwrap();
        fs::write(source_dir.join("attachments").join("index.json"), b"[]").unwrap();

        let package = export_package(source_excel.to_str().unwrap()).unwrap();
        let result = import_package(target_excel.to_str().unwrap(), &package).unwrap();
        assert_eq!(result["attachmentCount"], 0);
        assert_eq!(
            fs::read(
                target_dir
                    .join("attachments")
                    .join("LS0001")
                    .join("test.png")
            )
            .unwrap(),
            b"attachment-data"
        );
        excel::validate_workbook(&target_excel).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn package_rejects_path_traversal() {
        assert!(safe_relative_path("../outside.txt").is_err());
        assert!(safe_relative_path("/absolute.txt").is_err());
        assert!(safe_relative_path("attachments/file.png").is_ok());
    }
}
