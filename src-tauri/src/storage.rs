use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn temporary_path(target: &Path, extension: &str) -> Result<PathBuf, String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("无法确定目标文件目录「{}」", target.display()))?;
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    Ok(parent.join(format!(
        ".{}.{}.{}",
        file_name,
        Uuid::new_v4(),
        extension.trim_start_matches('.')
    )))
}

pub fn atomic_write(target: &Path, data: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("无法确定目标文件目录「{}」", target.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("创建目录失败「{}」: {}", parent.display(), e))?;

    let temporary = temporary_path(target, "tmp")?;
    let result = (|| {
        let mut file = File::create(&temporary)
            .map_err(|e| format!("创建临时文件失败「{}」: {}", temporary.display(), e))?;
        file.write_all(data)
            .map_err(|e| format!("写入临时文件失败「{}」: {}", temporary.display(), e))?;
        file.sync_all()
            .map_err(|e| format!("刷新临时文件失败「{}」: {}", temporary.display(), e))?;
        drop(file);
        replace_file(&temporary, target)
    })();

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn atomic_write_json<T: serde::Serialize + ?Sized>(
    target: &Path,
    value: &T,
) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("序列化 JSON 失败「{}」: {}", target.display(), e))?;
    atomic_write(target, &content)
}

#[cfg(windows)]
pub fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target_wide: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        return Err(format!(
            "原子替换文件失败「{}」→「{}」: {}",
            source.display(),
            target.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    fs::rename(source, target).map_err(|e| {
        format!(
            "原子替换文件失败「{}」→「{}」: {}",
            source.display(),
            target.display(),
            e
        )
    })
}

pub fn sync_file(path: &Path) -> Result<(), String> {
    File::options()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("刷新文件失败「{}」: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_existing_content() {
        let root = std::env::temp_dir().join(format!("mold-storage-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let target = root.join("config.json");
        fs::write(&target, b"old").unwrap();
        atomic_write(&target, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert!(fs::read_dir(&root).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")));
        fs::remove_dir_all(root).unwrap();
    }
}
