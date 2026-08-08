const SERVICE_NAME: &str = "mold-management";

pub const AGENT_API_KEY: &str = "agent-api-key";
pub const AGENT_GLM_API_KEY: &str = "agent-glm-api-key";
pub const AGENT_OPENAI_API_KEY: &str = "agent-openai-api-key";
pub const AGENT_ANTHROPIC_API_KEY: &str = "agent-anthropic-api-key";
pub const AGENT_DEEPSEEK_API_KEY: &str = "agent-deepseek-api-key";
pub const AGENT_QWEN_API_KEY: &str = "agent-qwen-api-key";
pub const AGENT_GEMINI_API_KEY: &str = "agent-gemini-api-key";
pub const AGENT_CUSTOM_ANTHROPIC_API_KEY: &str = "agent-custom-anthropic-api-key";
pub const WEBDAV_CREDENTIALS: &str = "webdav-credentials";

fn entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE_NAME, account)
        .map_err(|error| format!("打开系统凭据库失败（{}）: {}", platform_store_name(), error))
}

pub fn set(account: &str, secret: &str) -> Result<(), String> {
    if secret.is_empty() {
        return Err("不能保存空凭据".to_string());
    }
    entry(account)?.set_password(secret).map_err(|error| {
        format!(
            "保存凭据到{}失败: {}{}",
            platform_store_name(),
            error,
            platform_help()
        )
    })
}

pub fn get(account: &str) -> Result<String, String> {
    entry(account)?.get_password().map_err(|error| match error {
        keyring::Error::NoEntry => format!("系统凭据库中没有找到「{}」", account),
        other => format!(
            "从{}读取凭据失败: {}{}",
            platform_store_name(),
            other,
            platform_help()
        ),
    })
}

pub fn exists(account: &str) -> bool {
    get(account).is_ok()
}

pub fn platform_store_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows 凭据管理器"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS 钥匙串"
    }
    #[cfg(target_os = "linux")]
    {
        "Linux Secret Service"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "系统凭据库"
    }
}

fn platform_help() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "。请确认桌面会话已启动并解锁 GNOME Keyring、KWallet 或兼容的 Secret Service；无凭据服务时可本次填写，但系统不会降级为明文保存"
    }
    #[cfg(not(target_os = "linux"))]
    {
        ""
    }
}

// 仅用于从旧版 Windows DPAPI 配置迁移；新配置不再向 config.json 写入密文。
#[cfg(windows)]
pub fn unprotect_legacy(cipher_text: &str) -> Result<String, String> {
    use base64::Engine;
    use windows_sys::Win32::Foundation::{LocalFree, HLOCAL};
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(cipher_text)
        .map_err(|error| format!("解析旧版加密凭据失败: {}", error))?;
    let mut input = CRYPT_INTEGER_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let success = unsafe {
        CryptUnprotectData(
            &mut input,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if success == 0 {
        return Err(format!(
            "解密旧版 Windows 凭据失败: {}",
            std::io::Error::last_os_error()
        ));
    }
    let decrypted = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let value = String::from_utf8(decrypted.to_vec())
        .map_err(|error| format!("旧版凭据不是有效文本: {}", error));
    unsafe {
        if !output.pbData.is_null() {
            LocalFree(output.pbData as HLOCAL);
        }
    }
    value
}

#[cfg(not(windows))]
pub fn unprotect_legacy(_cipher_text: &str) -> Result<String, String> {
    Err("旧版 DPAPI 凭据只能在原 Windows 用户环境中迁移，请重新输入凭据".to_string())
}
