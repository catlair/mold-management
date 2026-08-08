use chrono::Local;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, ETAG, IF_MATCH, IF_NONE_MATCH};
use reqwest::{Method, StatusCode, Url};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

const MAX_PACKAGE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_DOCUMENT_BYTES: u64 = 100 * 1024 * 1024;
const REMOTE_PROTOCOL_VERSION: u32 = 2;
const DEFAULT_REMOTE_PATH: &str = "mold-management.moldpkg";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteComponent {
    Config,
    Document,
    Attachments,
}

impl RemoteComponent {
    fn key(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Document => "document",
            Self::Attachments => "attachments",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub url: String,
    pub remote_path: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatus {
    pub connected: bool,
    pub exists: bool,
    pub remote_path: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResult {
    pub success: bool,
    pub etag: Option<String>,
    pub uploaded_at: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDescriptor {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitManifest {
    pub format: String,
    pub version: u32,
    pub uploaded_at: String,
    pub config: ComponentDescriptor,
    pub document: ComponentDescriptor,
    pub attachments: ComponentDescriptor,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub config: Vec<u8>,
    pub document: Vec<u8>,
    pub attachments: Vec<u8>,
    pub etag: Option<String>,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct UploadComponent {
    pub kind: RemoteComponent,
    pub data: Vec<u8>,
    /// 稳定内容标识。文档组件使用业务内容哈希，避免 SQLite 文件布局变化导致重复上传。
    pub content_id: Option<String>,
}

pub fn default_remote_path() -> String {
    DEFAULT_REMOTE_PATH.to_string()
}

pub fn validate_settings(url: &str, remote_path: &str) -> Result<(), String> {
    let parsed = Url::parse(url.trim()).map_err(|e| format!("WebDAV 地址无效: {}", e))?;
    if parsed.scheme() != "https" {
        return Err("WebDAV 地址必须使用 HTTPS".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("WebDAV 地址缺少服务器域名".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("WebDAV 地址不能内嵌账户或密码，请使用独立凭据配置".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("WebDAV 地址不能包含查询参数或片段".to_string());
    }
    let path = remote_path.trim().trim_start_matches('/');
    if path.is_empty() || !path.to_ascii_lowercase().ends_with(".moldpkg") {
        return Err("远端文件必须是相对路径并以 .moldpkg 结尾".to_string());
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("远端文件路径包含无效目录段".to_string());
    }
    if path.contains(['?', '#', '%', '\\']) {
        return Err("远端文件路径不能包含 ?、#、% 或反斜杠".to_string());
    }
    Ok(())
}

pub fn development_credentials() -> Option<Credentials> {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let candidates = [
            manifest_dir.parent().map(|path| path.join(".dev")),
            std::env::current_dir().ok().map(|path| path.join(".dev")),
        ];
        for candidate in candidates.into_iter().flatten() {
            if let Ok(content) = fs::read_to_string(candidate) {
                let values = parse_dev_file(&content);
                let credentials = Credentials {
                    url: values.get("WEBDAV_URL")?.to_string(),
                    remote_path: values
                        .get("WEBDAV_REMOTE_PATH")
                        .cloned()
                        .unwrap_or_else(default_remote_path),
                    username: values.get("WEBDAV_USERNAME")?.to_string(),
                    password: values.get("WEBDAV_PASSWORD")?.to_string(),
                };
                if validate_settings(&credentials.url, &credentials.remote_path).is_ok()
                    && !credentials.username.is_empty()
                    && !credentials.password.is_empty()
                {
                    return Some(credentials);
                }
            }
        }
    }
    None
}

fn parse_dev_file(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub fn test_connection(credentials: &Credentials) -> Result<RemoteStatus, String> {
    validate_credentials(credentials)?;
    let client = build_client()?;
    let method = Method::from_bytes(b"PROPFIND").map_err(|e| e.to_string())?;
    let response = authenticate(
        client
            .request(method, base_url(credentials)?)
            .header("Depth", "0")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body("<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><prop><displayname/></prop></propfind>"),
        credentials,
    )
    .send()
    .map_err(|e| request_error("连接 WebDAV", e))?;
    let status = response.status();
    if status != StatusCode::MULTI_STATUS && !status.is_success() {
        return Err(status_error("连接 WebDAV", response));
    }
    remote_status(credentials)
}

/// 从 WebDAV PROPFIND XML 中提取指定属性的文本值（兼容 `d:` 等命名空间前缀）。
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let needle_open = format!(":{}>", tag);
    let index = xml.find(&needle_open)?;
    let after = &xml[index + needle_open.len()..];
    if after.starts_with("/>") {
        return None; // 自闭合属性，无值
    }
    let end = after.find("</")?;
    let value = after[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub fn remote_status(credentials: &Credentials) -> Result<RemoteStatus, String> {
    validate_credentials(credentials)?;
    let client = build_client()?;
    // 坚果云等服务器的 HEAD 响应可能不返回 Content-Length 与 ETag，
    // 改用 PROPFIND 获取真实大小（getcontentlength）、修改时间与 ETag。
    let method = Method::from_bytes(b"PROPFIND").map_err(|e| e.to_string())?;
    let response = authenticate(
        client
            .request(method, remote_url(credentials)?)
            .header("Depth", "0")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(
                "<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><prop><getcontentlength/><getlastmodified/><getetag/></prop></propfind>",
            ),
        credentials,
    )
    .send()
    .map_err(|e| request_error("查询 WebDAV 远端文件", e))?;
    let status = response.status();
    if status == StatusCode::NOT_FOUND {
        return Ok(RemoteStatus {
            connected: true,
            exists: false,
            remote_path: credentials.remote_path.clone(),
            etag: None,
            last_modified: None,
            size: None,
        });
    }
    if status != StatusCode::MULTI_STATUS && !status.is_success() {
        return Err(status_error("查询 WebDAV 远端文件", response));
    }
    let body = response
        .text()
        .map_err(|e| format!("读取 WebDAV 远端文件信息失败: {}", e))?;
    Ok(RemoteStatus {
        connected: true,
        exists: true,
        remote_path: credentials.remote_path.clone(),
        etag: extract_xml_value(&body, "getetag"),
        last_modified: extract_xml_value(&body, "getlastmodified"),
        size: extract_xml_value(&body, "getcontentlength")
            .and_then(|value| value.parse::<u64>().ok()),
    })
}

pub fn upload(
    credentials: &Credentials,
    components: Vec<UploadComponent>,
    expected_etag: Option<&str>,
    force_overwrite: bool,
) -> Result<UploadResult, String> {
    validate_credentials(credentials)?;
    if components.len() != 3 {
        return Err("WebDAV 上传必须包含配置、文档和附件三个组件".to_string());
    }
    let current = remote_status(credentials)?;
    if current.exists && !force_overwrite {
        match (expected_etag, current.etag.as_deref()) {
            (Some(expected), Some(actual)) if expected == actual => {}
            (Some(expected), None) if expected == "__NO_ETAG__" => {}
            (Some(_), Some(_)) => {
                return Err(
                    "WEBDAV_CONFLICT|远端清单已被其他设备更新，请先下载或明确覆盖远端".to_string(),
                )
            }
            _ => return Err(
                "WEBDAV_CONFLICT|远端已存在清单，但本机没有可验证的 ETag，请先下载或明确覆盖远端"
                    .to_string(),
            ),
        }
    }

    let client = build_client()?;
    // 远端清单可能是旧版本单文件格式或损坏：非强制上传时交给上层提示用户确认；
    // 用户选择强制覆盖后，把旧格式视为「无先前清单」继续上传，不再阻塞。
    let previous_manifest = match get_optional_manifest(&client, credentials) {
        Ok(manifest) => manifest,
        Err(error) if force_overwrite && error.starts_with("WEBDAV_LEGACY_MANIFEST|") => None,
        Err(error) => return Err(error),
    };
    let uploaded_at = Local::now().to_rfc3339();
    let mut descriptors = HashMap::new();
    let mut uploaded_paths = Vec::new();
    for component in components {
        let limit = if component.kind == RemoteComponent::Document {
            MAX_DOCUMENT_BYTES
        } else {
            MAX_PACKAGE_BYTES
        };
        if component.data.len() as u64 > limit {
            return Err(format!("WebDAV {}组件超过大小限制", component.kind.key()));
        }
        let sha256 = sha256_hex(&component.data);
        let object_id = component.content_id.as_deref().unwrap_or(&sha256);
        if object_id.len() != 64 || !object_id.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(format!("WebDAV {}组件内容标识无效", component.kind.key()));
        }
        let path = component_path(credentials, component.kind, object_id);
        let previous_matches = previous_manifest
            .as_ref()
            .map(|manifest| descriptor_for(manifest, component.kind))
            .is_some_and(|descriptor| descriptor.sha256 == sha256 && descriptor.path == path);
        if !previous_matches {
            put_bytes(&client, credentials, &path, component.data.clone(), None)?;
            uploaded_paths.push(path.clone());
        }
        descriptors.insert(
            component.kind.key(),
            ComponentDescriptor {
                path,
                size: component.data.len() as u64,
                sha256,
            },
        );
    }
    let manifest = SplitManifest {
        format: "mold-management-webdav-split".to_string(),
        version: REMOTE_PROTOCOL_VERSION,
        uploaded_at: uploaded_at.clone(),
        config: descriptors.remove("config").unwrap(),
        document: descriptors.remove("document").unwrap(),
        attachments: descriptors.remove("attachments").unwrap(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("生成 WebDAV 远端清单失败: {}", e))?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let condition = if force_overwrite {
        None
    } else if current.exists {
        Some((IF_MATCH, expected_etag.unwrap_or("__NO_ETAG__")))
    } else {
        Some((IF_NONE_MATCH, "*"))
    };
    if let Err(error) = put_bytes(
        &client,
        credentials,
        &credentials.remote_path,
        manifest_bytes,
        condition,
    ) {
        for path in uploaded_paths {
            let _ = delete_remote(&client, credentials, &path);
        }
        return Err(error);
    }
    let updated = remote_status(credentials)?;
    if let Some(previous) = previous_manifest {
        for descriptor in [&previous.config, &previous.document, &previous.attachments] {
            let still_used = [
                &manifest.config.path,
                &manifest.document.path,
                &manifest.attachments.path,
            ]
            .contains(&&descriptor.path);
            if !still_used {
                let _ = delete_remote(&client, credentials, &descriptor.path);
            }
        }
    }
    Ok(UploadResult {
        success: true,
        etag: updated.etag,
        uploaded_at,
        size: manifest.config.size + manifest.document.size + manifest.attachments.size,
        sha256: manifest_sha256,
    })
}

pub fn download(credentials: &Credentials) -> Result<DownloadResult, String> {
    validate_credentials(credentials)?;
    let client = build_client()?;
    let (manifest_bytes, etag) = get_bytes_with_etag(
        &client,
        credentials,
        &credentials.remote_path,
        MAX_DOCUMENT_BYTES,
        "下载 WebDAV 远端清单",
    )?;
    let manifest: SplitManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("解析 WebDAV 远端清单失败: {}", e))?;
    validate_manifest(&manifest)?;
    let config = get_component(&client, credentials, &manifest.config, MAX_DOCUMENT_BYTES)?;
    let document = get_component(&client, credentials, &manifest.document, MAX_DOCUMENT_BYTES)?;
    let attachments = get_component(
        &client,
        credentials,
        &manifest.attachments,
        MAX_PACKAGE_BYTES,
    )?;
    Ok(DownloadResult {
        config,
        document,
        attachments,
        etag,
        sha256: sha256_hex(&manifest_bytes),
    })
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建 WebDAV 客户端失败: {}", e))
}

fn validate_credentials(credentials: &Credentials) -> Result<(), String> {
    validate_settings(&credentials.url, &credentials.remote_path)?;
    if credentials.username.trim().is_empty() || credentials.password.is_empty() {
        return Err("WebDAV 账户和应用密码不能为空".to_string());
    }
    Ok(())
}

fn base_url(credentials: &Credentials) -> Result<Url, String> {
    let mut base =
        Url::parse(credentials.url.trim()).map_err(|e| format!("WebDAV 地址无效: {}", e))?;
    if !base.path().ends_with('/') {
        let next = format!("{}/", base.path());
        base.set_path(&next);
    }
    Ok(base)
}

fn remote_url_for(credentials: &Credentials, path: &str) -> Result<Url, String> {
    base_url(credentials)?
        .join(path.trim_start_matches('/'))
        .map_err(|e| format!("构造 WebDAV 远端路径失败: {}", e))
}

fn remote_url(credentials: &Credentials) -> Result<Url, String> {
    remote_url_for(credentials, &credentials.remote_path)
}

fn authenticate(
    builder: reqwest::blocking::RequestBuilder,
    credentials: &Credentials,
) -> reqwest::blocking::RequestBuilder {
    builder.basic_auth(&credentials.username, Some(&credentials.password))
}

fn component_path(credentials: &Credentials, kind: RemoteComponent, sha256: &str) -> String {
    let base = credentials
        .remote_path
        .strip_suffix(".moldpkg")
        .unwrap_or(&credentials.remote_path);
    let extension = match kind {
        RemoteComponent::Config => "json",
        RemoteComponent::Document => "db",
        RemoteComponent::Attachments => "zip",
    };
    format!("{}.{}.{}.{}", base, kind.key(), sha256, extension)
}

fn descriptor_for(manifest: &SplitManifest, kind: RemoteComponent) -> &ComponentDescriptor {
    match kind {
        RemoteComponent::Config => &manifest.config,
        RemoteComponent::Document => &manifest.document,
        RemoteComponent::Attachments => &manifest.attachments,
    }
}

fn validate_manifest(manifest: &SplitManifest) -> Result<(), String> {
    if manifest.format != "mold-management-webdav-split"
        || manifest.version != REMOTE_PROTOCOL_VERSION
    {
        return Err("WebDAV 远端清单格式或版本无效".to_string());
    }
    for descriptor in [&manifest.config, &manifest.document, &manifest.attachments] {
        if descriptor.path.trim().is_empty()
            || descriptor.sha256.len() != 64
            || descriptor.path.contains(['\\', '?', '#', '%'])
        {
            return Err("WebDAV 远端清单包含无效组件路径或哈希".to_string());
        }
        let normalized = crate::data_package::safe_relative_path(&descriptor.path)?;
        if normalized.to_string_lossy() != descriptor.path {
            return Err("WebDAV 远端清单组件路径未规范化".to_string());
        }
    }
    Ok(())
}

fn get_optional_manifest(
    client: &Client,
    credentials: &Credentials,
) -> Result<Option<SplitManifest>, String> {
    let response = authenticate(client.get(remote_url(credentials)?), credentials)
        .send()
        .map_err(|e| request_error("读取 WebDAV 现有清单", e))?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(status_error("读取 WebDAV 现有清单", response));
    }
    let manifest = response
        .json::<SplitManifest>()
        .map_err(|e| legacy_manifest_error(e))?;
    validate_manifest(&manifest)?;
    Ok(Some(manifest))
}

/// 远端文件存在但无法解析为新版清单时的可识别错误。
/// 典型场景：远端是旧版本单文件 .moldpkg（ZIP）或文件损坏。
fn legacy_manifest_error(detail: impl std::fmt::Display) -> String {
    format!(
        "WEBDAV_LEGACY_MANIFEST|远端快照不是新版格式（可能是旧版本数据或文件损坏，解析错误：{}）。如需上传，请选择「强制覆盖远端」。",
        detail
    )
}

fn get_bytes_with_etag(
    client: &Client,
    credentials: &Credentials,
    path: &str,
    max_bytes: u64,
    action: &str,
) -> Result<(Vec<u8>, Option<String>), String> {
    let response = authenticate(client.get(remote_url_for(credentials, path)?), credentials)
        .send()
        .map_err(|e| request_error(action, e))?;
    if response.status() == StatusCode::NOT_FOUND {
        return Err(format!("{}失败：远端文件不存在", action));
    }
    if !response.status().is_success() {
        return Err(status_error(action, response));
    }
    if response
        .content_length()
        .is_some_and(|size| size > max_bytes)
    {
        return Err(format!("{}失败：远端文件超过大小限制", action));
    }
    let etag = header_text(response.headers(), ETAG);
    let data = response
        .bytes()
        .map_err(|e| request_error(action, e))?
        .to_vec();
    if data.len() as u64 > max_bytes {
        return Err(format!("{}失败：远端文件超过大小限制", action));
    }
    Ok((data, etag))
}

fn get_component(
    client: &Client,
    credentials: &Credentials,
    descriptor: &ComponentDescriptor,
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    let (data, _) = get_bytes_with_etag(
        client,
        credentials,
        &descriptor.path,
        max_bytes,
        "下载 WebDAV 组件",
    )?;
    if data.len() as u64 != descriptor.size || sha256_hex(&data) != descriptor.sha256 {
        return Err(format!(
            "WebDAV 组件 SHA-256 校验失败「{}」，已停止恢复",
            descriptor.path
        ));
    }
    Ok(data)
}

fn put_bytes(
    client: &Client,
    credentials: &Credentials,
    path: &str,
    data: Vec<u8>,
    condition: Option<(reqwest::header::HeaderName, &str)>,
) -> Result<(), String> {
    let mut request = client.put(remote_url_for(credentials, path)?).body(data);
    if let Some((name, value)) = condition {
        if value != "__NO_ETAG__" {
            let header = HeaderValue::from_str(value)
                .map_err(|e| format!("构造 WebDAV 条件请求失败: {}", e))?;
            request = request.header(name, header);
        }
    }
    let response = authenticate(request, credentials)
        .send()
        .map_err(|e| request_error("上传 WebDAV 文件", e))?;
    if response.status().is_success() {
        Ok(())
    } else if response.status() == StatusCode::PRECONDITION_FAILED {
        Err("WEBDAV_CONFLICT|远端清单已被其他设备更新，请先下载或明确覆盖远端".to_string())
    } else {
        Err(status_error("上传 WebDAV 文件", response))
    }
}

/// 清理 WebDAV 远端文件。当前仅集成测试调用（正式上传已改为 PUT 直写），保留以便后续清理/回滚逻辑使用。
#[allow(dead_code)]
fn delete_remote(client: &Client, credentials: &Credentials, path: &str) -> Result<(), String> {
    let response = authenticate(
        client.delete(remote_url_for(credentials, path)?),
        credentials,
    )
    .send()
    .map_err(|e| request_error("清理 WebDAV 临时文件", e))?;
    if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
        Ok(())
    } else {
        Err(status_error("清理 WebDAV 临时文件", response))
    }
}

fn header_text(headers: &HeaderMap, name: reqwest::header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn request_error(action: &str, error: reqwest::Error) -> String {
    if error.is_timeout() {
        format!("{}超时，请检查网络与服务器地址", action)
    } else if error.is_connect() {
        format!("{}失败，无法连接服务器: {}", action, error)
    } else {
        format!("{}失败: {}", action, error)
    }
}

fn status_error(action: &str, response: Response) -> String {
    let status = response.status();
    let reason = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "账户或应用密码不正确，或没有访问权限",
        StatusCode::NOT_FOUND => "远端路径不存在",
        StatusCode::PRECONDITION_FAILED => "远端文件已变化",
        StatusCode::INSUFFICIENT_STORAGE => "WebDAV 可用空间不足",
        _ => "服务器拒绝了请求",
    };
    format!("{}失败（HTTP {}）：{}", action, status.as_u16(), reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn validates_secure_webdav_paths() {
        assert!(validate_settings("https://example.com/dav/", "mold-data.moldpkg").is_ok());
        assert!(validate_settings("http://example.com/dav/", "mold-data.moldpkg").is_err());
        assert!(validate_settings("https://example.com/dav/", "../data.moldpkg").is_err());
        assert!(validate_settings("https://example.com/dav/", "%2e%2e/data.moldpkg").is_err());
        assert!(validate_settings("https://user:secret@example.com/dav/", "data.moldpkg").is_err());
        assert!(
            validate_settings("https://example.com/dav/?token=secret", "data.moldpkg").is_err()
        );
        assert!(validate_settings("https://example.com/dav/", "data.zip").is_err());
    }

    #[test]
    fn parses_unquoted_development_environment() {
        let values = parse_dev_file(
            "WEBDAV_URL=https://example.com/dav/\n# ignored\nWEBDAV_USERNAME=user\n",
        );
        assert_eq!(values.get("WEBDAV_USERNAME").unwrap(), "user");
        assert_eq!(values.len(), 2);
    }

    #[test]
    #[ignore = "requires local .dev credentials and network access"]
    fn development_webdav_connection_is_valid() {
        let credentials =
            development_credentials().expect("missing development WebDAV credentials");
        let status = test_connection(&credentials).expect("WebDAV connection failed");
        assert!(status.connected);
    }

    #[test]
    #[ignore = "writes and removes a unique WebDAV test package"]
    fn development_webdav_round_trip_is_valid() {
        let mut credentials =
            development_credentials().expect("missing development WebDAV credentials");
        let filename = format!(
            "mold-management-integration-{}.moldpkg",
            Uuid::new_v4().simple()
        );
        credentials.remote_path = credentials
            .remote_path
            .rsplit_once('/')
            .map(|(parent, _)| format!("{}/{}", parent, filename))
            .unwrap_or(filename);
        let config_payload = format!("config:{}", Uuid::new_v4()).into_bytes();
        let document_payload = format!("document:{}", Uuid::new_v4()).into_bytes();
        let attachments_payload = format!("attachments:{}", Uuid::new_v4()).into_bytes();

        let outcome = (|| -> Result<SplitManifest, String> {
            let status = test_connection(&credentials)?;
            if !status.connected {
                return Err("WebDAV connection did not report connected".to_string());
            }
            upload(
                &credentials,
                vec![
                    UploadComponent {
                        kind: RemoteComponent::Config,
                        data: config_payload.clone(),
                        content_id: None,
                    },
                    UploadComponent {
                        kind: RemoteComponent::Document,
                        data: document_payload.clone(),
                        content_id: None,
                    },
                    UploadComponent {
                        kind: RemoteComponent::Attachments,
                        data: attachments_payload.clone(),
                        content_id: None,
                    },
                ],
                None,
                true,
            )?;
            let downloaded = download(&credentials)?;
            if downloaded.config != config_payload
                || downloaded.document != document_payload
                || downloaded.attachments != attachments_payload
            {
                return Err("WebDAV split round-trip payload mismatch".to_string());
            }
            let client = build_client()?;
            get_optional_manifest(&client, &credentials)?
                .ok_or_else(|| "uploaded manifest missing".to_string())
        })();

        let manifest = outcome.expect("WebDAV round trip failed");
        let client = build_client().expect("failed to create cleanup client");
        delete_remote(&client, &credentials, &credentials.remote_path)
            .expect("failed to remove WebDAV test manifest");
        for descriptor in [manifest.config, manifest.document, manifest.attachments] {
            delete_remote(&client, &credentials, &descriptor.path)
                .expect("failed to remove WebDAV test component");
        }
    }
}
