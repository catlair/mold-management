use crate::{excel, storage};
use base64::Engine;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MAX_ATTACHMENT_BYTES: u64 = 50 * 1024 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "pdf"];
const MAX_ATTACHMENT_FILE_NAME: usize = 160;
const SCREW_SPEC_SHEET: &str = "螺丝规格表";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrewAttachment {
    pub id: String,
    pub screw_spec_id: String,
    pub display_name: String,
    pub file_name: String,
    pub mime_type: String,
    pub size: u64,
    pub relative_path: String,
    #[serde(default)]
    pub annotations: Vec<Value>,
    #[serde(default)]
    pub sort_order: usize,
    pub created_at: String,
    pub updated_at: String,
}

fn attachment_root(data_file_path: &str) -> Result<PathBuf, String> {
    let parent = Path::new(data_file_path)
        .parent()
        .ok_or_else(|| "无法确定数据文件目录".to_string())?;
    Ok(parent.join("attachments"))
}

pub fn root_path(data_file_path: &str) -> Result<PathBuf, String> {
    attachment_root(data_file_path)
}

fn index_path(root: &Path) -> PathBuf {
    root.join("index.json")
}

fn load_index(root: &Path) -> Result<Vec<ScrewAttachment>, String> {
    let path = index_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取附件索引失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析附件索引失败: {e}"))
}

pub fn validate_root(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    if !root.is_dir() {
        return Err(format!("附件路径不是目录「{}」", root.display()));
    }
    let attachments = load_index(root)?;
    for attachment in &attachments {
        resolve_stored_path(root, &attachment.relative_path).map_err(|error| {
            format!("附件索引记录无效「{}」: {}", attachment.display_name, error)
        })?;
    }
    Ok(())
}

pub fn count_index_entries(root: &Path) -> Result<usize, String> {
    Ok(load_index(root)?.len())
}

fn save_index(root: &Path, attachments: &[ScrewAttachment]) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("创建附件目录失败: {e}"))?;
    storage::atomic_write_json(&index_path(root), attachments)
        .map_err(|e| format!("保存附件索引失败: {e}"))
}

fn safe_segment(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

/// 生成可读附件文件名：螺丝ID_螺丝名_设置的名称[.扩展名]。
/// 清洗 Windows 非法字符、限制总长度；供导入、改名和备份打包共用。
pub(crate) fn build_attachment_file_name(
    screw_id: &str,
    screw_name: &str,
    display_name: &str,
    extension: Option<&str>,
) -> String {
    let id_part = sanitize_name_part(screw_id, 40);
    let name_part = sanitize_name_part(screw_name, 60);
    let display_part = sanitize_name_part(display_name, 80);
    let mut name = format!("{}_{}_{}", id_part, name_part, display_part);
    if Path::new(&display_part).extension().is_none() {
        if let Some(ext) = extension.filter(|value| !value.is_empty()) {
            name.push('.');
            name.push_str(&sanitize_name_part(ext, 10));
        }
    }
    truncate_file_name(&name, MAX_ATTACHMENT_FILE_NAME)
}

fn sanitize_name_part(value: &str, limit: usize) -> String {
    let cleaned: String = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => ch,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.chars().take(limit).collect()
    }
}

fn truncate_file_name(name: &str, limit: usize) -> String {
    if name.chars().count() <= limit {
        return name.to_string();
    }
    let extension = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value))
        .unwrap_or_default();
    let keep = limit.saturating_sub(extension.chars().count());
    let head: String = name.chars().take(keep).collect();
    format!("{}{}", head.trim_end_matches('.'), extension)
}

/// 同一目录内重名时追加序号（保持扩展名）。
fn unique_stored_name(base: &str, used: &std::collections::HashSet<String>) -> String {
    if !used.contains(base) {
        return base.to_string();
    }
    let (stem, extension) = match base.rfind('.') {
        Some(index) if index > 0 => {
            let (head, tail) = base.split_at(index);
            (head.to_string(), tail.to_string())
        }
        _ => (base.to_string(), String::new()),
    };
    let mut counter = 1;
    loop {
        let candidate = if extension.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}){}", stem, counter, extension)
        };
        if !used.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// 查询螺丝规格名；查询失败（测试数据或螺丝已删除）时回退为螺丝 ID。
fn screw_display_name(data_file_path: &str, screw_spec_id: &str) -> String {
    excel::get_by_id(data_file_path, SCREW_SPEC_SHEET, screw_spec_id)
        .ok()
        .flatten()
        .and_then(|row| row.get("name").cloned())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| screw_spec_id.to_string())
}

fn mime_type(extension: &str) -> &'static str {
    match extension {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn resolve_stored_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let candidate = root.join(relative_path);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|_| "附件文件不存在".to_string())?;
    if !canonical_candidate.starts_with(canonical_root) {
        return Err("附件路径无效".to_string());
    }
    Ok(canonical_candidate)
}

pub fn list(data_file_path: &str, screw_spec_id: &str) -> Result<Vec<ScrewAttachment>, String> {
    let root = attachment_root(data_file_path)?;
    let mut items: Vec<_> = load_index(&root)?
        .into_iter()
        .filter(|item| item.screw_spec_id == screw_spec_id)
        .collect();
    items.sort_by_key(|item| item.sort_order);
    Ok(items)
}

pub fn counts(data_file_path: &str) -> Result<std::collections::HashMap<String, usize>, String> {
    let root = attachment_root(data_file_path)?;
    let mut result = std::collections::HashMap::new();
    for item in load_index(&root)? {
        *result.entry(item.screw_spec_id).or_insert(0) += 1;
    }
    Ok(result)
}

/// 读取全部附件元数据（备份/数据包打包时用于生成可读条目名）。
pub fn load_all(data_file_path: &str) -> Result<Vec<ScrewAttachment>, String> {
    let root = attachment_root(data_file_path)?;
    load_index(&root)
}

pub fn import(
    data_file_path: &str,
    screw_spec_id: &str,
    source_path: &str,
) -> Result<ScrewAttachment, String> {
    if screw_spec_id.trim().is_empty() {
        return Err("请先保存螺丝规格，再添加附件".to_string());
    }

    let source = Path::new(source_path);
    if !source.is_file() {
        return Err("所选附件不存在".to_string());
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "无法识别附件类型".to_string())?;
    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err("仅支持 PNG、JPG、WEBP、GIF 和 PDF 文件".to_string());
    }
    let metadata = fs::metadata(source).map_err(|e| format!("读取附件信息失败: {e}"))?;
    if metadata.len() > MAX_ATTACHMENT_BYTES {
        return Err("单个附件不能超过 50MB".to_string());
    }

    let root = attachment_root(data_file_path)?;
    let screw_dir_name = safe_segment(screw_spec_id);
    let screw_dir = root.join(&screw_dir_name);
    fs::create_dir_all(&screw_dir).map_err(|e| format!("创建附件目录失败: {e}"))?;

    let file_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("附件")
        .to_string();
    let mut index = load_index(&root)?;
    let screw_name = screw_display_name(data_file_path, screw_spec_id);
    let base_name =
        build_attachment_file_name(screw_spec_id, &screw_name, &file_name, Some(&extension));
    let used: std::collections::HashSet<String> = index
        .iter()
        .filter(|item| item.screw_spec_id == screw_spec_id)
        .filter_map(|item| item.relative_path.rsplit('/').next())
        .map(|value| value.to_string())
        .collect();
    let stored_name = unique_stored_name(&base_name, &used);

    let id = Uuid::new_v4().to_string();
    let target = screw_dir.join(&stored_name);
    fs::copy(source, &target).map_err(|e| format!("复制附件失败: {e}"))?;

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let sort_order = index
        .iter()
        .filter(|item| item.screw_spec_id == screw_spec_id)
        .count();
    let attachment = ScrewAttachment {
        id,
        screw_spec_id: screw_spec_id.to_string(),
        display_name: file_name.clone(),
        file_name,
        mime_type: mime_type(&extension).to_string(),
        size: metadata.len(),
        relative_path: format!("{}/{}", screw_dir_name, stored_name),
        annotations: Vec::new(),
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    };
    index.push(attachment.clone());
    save_index(&root, &index)?;
    Ok(attachment)
}

pub fn read_content(data_file_path: &str, attachment_id: &str) -> Result<Value, String> {
    let root = attachment_root(data_file_path)?;
    let index = load_index(&root)?;
    let attachment = index
        .into_iter()
        .find(|item| item.id == attachment_id)
        .ok_or_else(|| "附件记录不存在".to_string())?;
    let path = resolve_stored_path(&root, &attachment.relative_path)?;
    let bytes = fs::read(path).map_err(|e| format!("读取附件失败: {e}"))?;
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(json!({ "attachment": attachment, "data": data }))
}

pub fn update(
    data_file_path: &str,
    attachment_id: &str,
    display_name: Option<String>,
    annotations: Option<Vec<Value>>,
    sort_order: Option<usize>,
) -> Result<ScrewAttachment, String> {
    let root = attachment_root(data_file_path)?;
    let mut index = load_index(&root)?;
    let Some(position) = index.iter().position(|item| item.id == attachment_id) else {
        return Err("附件记录不存在".to_string());
    };
    let mut renamed = false;
    let mut previous_relative_path: Option<String> = None;
    let mut renamed_from: Option<PathBuf> = None;
    let mut renamed_to: Option<PathBuf> = None;

    if let Some(ref name) = display_name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("附件名称不能为空".to_string());
        }
        let current = &index[position];
        let current_display = current.display_name.clone();
        let current_relative = current.relative_path.clone();
        let current_screw = current.screw_spec_id.clone();
        if trimmed != current_display {
            // 设置名称变化时同步重命名物理文件，保持「螺丝ID_螺丝名_设置名」一致。
            let extension = Path::new(&current_relative)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string());
            let screw_name = screw_display_name(data_file_path, &current_screw);
            let base_name = build_attachment_file_name(
                &current_screw,
                &screw_name,
                trimmed,
                extension.as_deref(),
            );
            let used: std::collections::HashSet<String> = index
                .iter()
                .enumerate()
                .filter(|(candidate_index, candidate)| {
                    *candidate_index != position && candidate.screw_spec_id == current_screw
                })
                .filter_map(|(_, candidate)| candidate.relative_path.rsplit('/').next())
                .map(|value| value.to_string())
                .collect();
            let stored_name = unique_stored_name(&base_name, &used);
            let old_path = resolve_stored_path(&root, &current_relative)?;
            let dir = current_relative.split('/').next().unwrap_or("attachments");
            let new_relative = format!("{}/{}", dir, stored_name);
            let new_path = root.join(&new_relative);
            fs::rename(&old_path, &new_path).map_err(|e| format!("重命名附件文件失败: {e}"))?;
            renamed = true;
            previous_relative_path = Some(current_relative);
            renamed_from = Some(old_path);
            renamed_to = Some(new_path);
        }
    }

    let item = &mut index[position];
    if let Some(ref name) = display_name {
        item.display_name = name.trim().to_string();
    }
    if let Some(value) = annotations {
        item.annotations = value;
    }
    if let Some(value) = sort_order {
        item.sort_order = value;
    }
    item.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if renamed {
        item.relative_path = renamed_to
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|value| value.to_str())
            .map(|file_name| {
                let dir = previous_relative_path
                    .as_deref()
                    .and_then(|value| value.split('/').next())
                    .unwrap_or("attachments");
                format!("{}/{}", dir, file_name)
            })
            .unwrap_or_default();
    }
    let result = item.clone();
    if let Err(error) = save_index(&root, &index) {
        // 索引写入失败时回滚文件重命名，避免索引指向不存在的文件。
        if renamed {
            if let (Some(from), Some(to)) = (renamed_from, renamed_to) {
                let _ = fs::rename(&to, &from);
            }
        }
        return Err(error);
    }
    Ok(result)
}

pub fn delete(data_file_path: &str, attachment_id: &str) -> Result<bool, String> {
    let root = attachment_root(data_file_path)?;
    let mut index = load_index(&root)?;
    let Some(position) = index.iter().position(|item| item.id == attachment_id) else {
        return Ok(false);
    };
    let item = index.remove(position);
    if let Ok(path) = resolve_stored_path(&root, &item.relative_path) {
        fs::remove_file(path).map_err(|e| format!("删除附件失败: {e}"))?;
    }
    save_index(&root, &index)?;
    Ok(true)
}

pub fn delete_for_screw(data_file_path: &str, screw_spec_id: &str) -> Result<(), String> {
    let root = attachment_root(data_file_path)?;
    let mut index = load_index(&root)?;
    let removed: Vec<_> = index
        .iter()
        .filter(|item| item.screw_spec_id == screw_spec_id)
        .cloned()
        .collect();
    index.retain(|item| item.screw_spec_id != screw_spec_id);
    for item in removed {
        if let Ok(path) = resolve_stored_path(&root, &item.relative_path) {
            let _ = fs::remove_file(path);
        }
    }
    save_index(&root, &index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use serde_json::json;

    fn test_paths(label: &str) -> (PathBuf, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("mold-attachment-test-{}-{}", label, Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        (root.join("mold-data.xlsx"), root)
    }

    #[test]
    fn attachment_lifecycle_preserves_original_and_metadata() {
        let (data_file, root) = test_paths("lifecycle");
        fs::write(&data_file, b"test workbook").unwrap();
        let source = root.join("source.png");
        let original = b"not-a-real-png-but-valid-storage-test";
        fs::write(&source, original).unwrap();

        let imported = import(
            data_file.to_str().unwrap(),
            "screw-1",
            source.to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(counts(data_file.to_str().unwrap()).unwrap()["screw-1"], 1);
        assert_eq!(
            list(data_file.to_str().unwrap(), "screw-1").unwrap().len(),
            1
        );
        // 物理文件名遵循「螺丝ID_螺丝名_原始文件名」规则（测试数据查不到螺丝名时回退为 ID）
        assert!(imported
            .relative_path
            .contains("screw-1_screw-1_source.png"));

        let content = read_content(data_file.to_str().unwrap(), &imported.id).unwrap();
        let encoded = content.get("data").and_then(Value::as_str).unwrap();
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
            original
        );

        let annotations = vec![json!({
            "id": "annotation-1",
            "page": 1,
            "tool": "rectangle",
            "color": "#ef4444",
            "strokeWidth": 4,
            "x": 0.1,
            "y": 0.2,
            "endX": 0.4,
            "endY": 0.5
        })];
        let updated = update(
            data_file.to_str().unwrap(),
            &imported.id,
            Some("已标注图纸".to_string()),
            Some(annotations),
            None,
        )
        .unwrap();
        assert_eq!(updated.display_name, "已标注图纸");
        assert_eq!(updated.annotations.len(), 1);
        // 设置名称变化后物理文件同步重命名，索引指向新文件名
        assert!(updated.relative_path.contains("已标注图纸"));
        assert!(!updated.relative_path.contains("source.png"));
        assert!(root
            .join("attachments")
            .join("screw-1")
            .join(&updated.relative_path.split('/').next_back().unwrap())
            .is_file());

        assert!(delete(data_file.to_str().unwrap(), &imported.id).unwrap());
        assert!(list(data_file.to_str().unwrap(), "screw-1")
            .unwrap()
            .is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deleting_screw_removes_only_its_attachments() {
        let (data_file, root) = test_paths("cleanup");
        fs::write(&data_file, b"test workbook").unwrap();
        let source = root.join("source.pdf");
        fs::write(&source, b"%PDF-1.4 test").unwrap();

        import(
            data_file.to_str().unwrap(),
            "screw-a",
            source.to_str().unwrap(),
        )
        .unwrap();
        import(
            data_file.to_str().unwrap(),
            "screw-b",
            source.to_str().unwrap(),
        )
        .unwrap();
        delete_for_screw(data_file.to_str().unwrap(), "screw-a").unwrap();

        assert!(list(data_file.to_str().unwrap(), "screw-a")
            .unwrap()
            .is_empty());
        assert_eq!(
            list(data_file.to_str().unwrap(), "screw-b").unwrap().len(),
            1
        );
        fs::remove_dir_all(root).unwrap();
    }
}
