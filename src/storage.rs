//! 原始输入及设置的本地保存；不保存派生计算结果，不访问网络。
use crate::model::FormState;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub const DEFAULT_TEXT_SIZE: i32 = 14;
pub const MIN_TEXT_SIZE: i32 = 12;
pub const MAX_TEXT_SIZE: i32 = 20;
pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const SCHEMA_VERSION: u32 = 1;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub form: FormState,
    pub dark_mode: bool,
    pub text_size: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            form: FormState::default(),
            dark_mode: false,
            text_size: DEFAULT_TEXT_SIZE,
        }
    }
}

impl AppConfig {
    pub fn sanitize(&mut self) {
        self.text_size = self.text_size.clamp(MIN_TEXT_SIZE, MAX_TEXT_SIZE);
        for index in 0..self.form.models.len() {
            let name = self.form.model_name(index);
            self.form.rename(index, name);
        }
    }
}

pub struct LoadedConfig {
    pub config: AppConfig,
    pub notice: String,
}

/// 内存模式用于测试/桌面预览；Android 只能传入系统提供的私有目录。
pub struct ConfigStore {
    path: Option<PathBuf>,
    blocked: bool,
    unavailable_reason: Option<String>,
}

impl ConfigStore {
    pub fn memory() -> Self {
        Self {
            path: None,
            blocked: false,
            unavailable_reason: None,
        }
    }

    pub fn unavailable(reason: &str) -> Self {
        Self {
            path: None,
            blocked: true,
            unavailable_reason: Some(reason.into()),
        }
    }

    pub fn in_directory(directory: impl AsRef<Path>) -> Self {
        Self {
            path: Some(directory.as_ref().join("config.json")),
            blocked: false,
            unavailable_reason: None,
        }
    }

    pub fn is_persistent(&self) -> bool {
        self.path.is_some() && !self.blocked
    }

    pub fn load(&mut self) -> LoadedConfig {
        self.blocked = self.unavailable_reason.is_some();
        let Some(path) = &self.path else {
            return LoadedConfig {
                config: AppConfig::default(),
                notice: self
                    .unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "预览模式：不保存输入".into()),
            };
        };
        let read = (|| -> io::Result<Vec<u8>> {
            let file = File::open(path)?;
            let mut bytes = Vec::new();
            file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes)?;
            Ok(bytes)
        })();
        let bytes = match read {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return LoadedConfig {
                    config: AppConfig::default(),
                    notice: "输入和设置将在本机自动保存".into(),
                };
            }
            Err(_) => return self.block_load("无法读取已保存数据；本次不覆盖原文件"),
        };
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return self.block_load("保存文件过大；本次不覆盖原文件");
        }
        // Read the version before interpreting fields: future schemas may change their types.
        let json = serde_json::from_slice::<serde_json::Value>(&bytes);
        if let Ok(value) = &json {
            if value
                .get("version")
                .is_some_and(|version| version.as_u64() != Some(u64::from(SCHEMA_VERSION)))
            {
                return self.block_load("保存文件版本不兼容；本次不覆盖原文件");
            }
        }
        let parsed = json.and_then(serde_json::from_value::<AppConfig>);
        match parsed {
            Ok(mut config) => {
                config.sanitize();
                LoadedConfig {
                    config,
                    notice: "已恢复本机输入和设置".into(),
                }
            }
            Err(_) => {
                // 必须先完整备份并同步，之后的新编辑才可以替换损坏配置。
                if backup(path, &bytes).is_ok() {
                    LoadedConfig {
                        config: AppConfig::default(),
                        notice: "保存数据损坏，已保留备份；当前使用默认值".into(),
                    }
                } else {
                    self.block_load("保存数据损坏且备份失败；本次不覆盖原文件")
                }
            }
        }
    }

    fn block_load(&mut self, notice: &str) -> LoadedConfig {
        self.blocked = true;
        LoadedConfig {
            config: AppConfig::default(),
            notice: notice.into(),
        }
    }

    /// 每次编辑立即提交，不能依赖 Android 正常退出或生命周期回调。
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        if self.blocked {
            return Err("自动保存不可用；原文件保持不变".into());
        }
        let Some(path) = &self.path else {
            return Ok(());
        };
        if config.version != SCHEMA_VERSION {
            return Err("不能保存不兼容版本".into());
        }
        let bytes = serde_json::to_vec(config).map_err(|_| "无法编码保存数据")?;
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return Err("输入过长，未保存；之前的数据仍保留".into());
        }
        atomic_write(path, &bytes)
            .map_err(|_| "自动保存失败，请检查存储空间；当前输入仍可计算".into())
    }
}

fn unique_suffix() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{}-{time}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

fn create_private(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn backup(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let backup_path = path.with_file_name(format!("config.broken-{}.json", unique_suffix()));
    let mut file = create_private(&backup_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    // 若备份失败，原文件仍然保留；不自动尝试覆盖。
    sync_directory(path.parent().unwrap_or_else(|| Path::new(".")))
}

struct TemporaryFile(PathBuf);
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temporary = TemporaryFile(parent.join(format!(".config-{}.tmp", unique_suffix())));
    let mut file = create_private(&temporary.0)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary.0, path)?;
    sync_directory(parent)
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}
