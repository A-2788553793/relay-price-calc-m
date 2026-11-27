use relay_price_calc_m::storage::{AppConfig, ConfigStore, MAX_CONFIG_BYTES};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static ID: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "relay-storage-test-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> ConfigStore {
        ConfigStore::in_directory(&self.0)
    }
    fn config(&self) -> PathBuf {
        self.0.join("config.json")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        assert!(self
            .0
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("relay-storage-test-"));
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn missing_file_uses_defaults_without_writing() {
    let dir = Directory::new();
    let mut store = dir.store();
    assert_eq!(store.load().config, AppConfig::default());
    assert!(!dir.config().exists());
    assert!(store.is_persistent());
}
#[test]
fn round_trip_keeps_two_models_names_raw_invalid_values_and_settings() {
    let dir = Directory::new();
    let mut config = AppConfig::default();
    for index in 0..2 {
        config.form.rename(index, format!("中文模型{index}"));
        config.form.set(index, 0, " ３.５ ".into());
        config.form.set(index, 4, "invalid%".into());
    }
    config.dark_mode = true;
    config.text_size = 20;
    dir.store().save(&config).unwrap();
    assert_eq!(dir.store().load().config, config);
}
#[test]
fn legacy_four_model_config_loads_first_two_and_saves_only_two() {
    let dir = Directory::new();
    let original = r#"{"version":1,"form":{"models":[{"name":"保留甲","values":[" ３.５ ","15","3.75","0.3","80"]},{"name":"保留乙","values":["invalid","20"]},{"name":"旧模型三","values":["0"]},{"name":"旧模型四","values":["0"]}]},"dark_mode":true,"text_size":18}"#;
    fs::write(dir.config(), original).unwrap();
    let mut store = dir.store();
    let loaded = store.load();
    assert!(store.is_persistent());
    assert_eq!(loaded.config.form.models.len(), 2);
    assert_eq!(loaded.config.form.models[0].name, "保留甲");
    assert_eq!(loaded.config.form.models[0].values[0], " ３.５ ");
    assert_eq!(loaded.config.form.models[1].name, "保留乙");
    assert_eq!(loaded.config.form.models[1].values[0], "invalid");
    assert!(loaded.config.dark_mode);
    assert_eq!(loaded.config.text_size, 18);
    assert_eq!(fs::read_to_string(dir.config()).unwrap(), original);
    store.save(&loaded.config).unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.config()).unwrap()).unwrap();
    assert_eq!(saved["form"]["models"].as_array().unwrap().len(), 2);
    assert_eq!(dir.store().load().config, loaded.config);
}
#[test]
fn edits_replace_atomically_without_leftover_temporary_files() {
    let dir = Directory::new();
    let store = dir.store();
    let mut config = AppConfig::default();
    for value in ["1", "1.", "12.34", "", "101%"] {
        config.form.set(1, 4, value.into());
        store.save(&config).unwrap();
        assert_eq!(dir.store().load().config, config);
    }
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
#[test]
fn independent_and_global_clear_persist_without_losing_names_or_settings() {
    let dir = Directory::new();
    let mut config = AppConfig {
        dark_mode: true,
        text_size: 18,
        ..AppConfig::default()
    };
    for index in 0..2 {
        config.form.rename(index, format!("保留{index}"));
        config.form.set(index, 1, "20".into());
    }
    config.form.clear_model(0);
    dir.store().save(&config).unwrap();
    let loaded = dir.store().load().config;
    assert!(loaded.form.models[0].values.iter().all(String::is_empty));
    assert_eq!(loaded.form.models[1].values[1], "20");
    config.form.clear();
    dir.store().save(&config).unwrap();
    assert_eq!(dir.store().load().config, config);
    assert_eq!(config.form.model_name(0), "保留0");
    assert!(config.dark_mode);
}
#[test]
fn omitted_fields_and_short_arrays_are_defaulted() {
    let dir = Directory::new();
    fs::write(dir.config(), r#"{"form":{"models":[{"values":["5"]}]}}"#).unwrap();
    let config = dir.store().load().config;
    assert_eq!(config.form.models[0].values[0], "5");
    assert_eq!(config.form.model_name(1), "模型2");
    assert_eq!(config.text_size, 14);
}
#[test]
fn settings_and_names_are_sanitized_on_load() {
    let dir = Directory::new();
    let mut config = AppConfig {
        text_size: 100,
        ..AppConfig::default()
    };
    config.form.models[0].name = " 中 ".repeat(40);
    dir.store().save(&config).unwrap();
    let loaded = dir.store().load().config;
    assert_eq!(loaded.text_size, 20);
    assert_eq!(loaded.form.models[0].name.chars().count(), 30);
    config.text_size = -9;
    dir.store().save(&config).unwrap();
    assert_eq!(dir.store().load().config.text_size, 12);
}
#[test]
fn damaged_json_is_backed_up_before_first_edit() {
    let dir = Directory::new();
    let damaged = b"{broken";
    fs::write(dir.config(), damaged).unwrap();
    let mut store = dir.store();
    let loaded = store.load();
    assert_eq!(loaded.config, AppConfig::default());
    assert!(loaded.notice.contains("备份"));
    assert_eq!(fs::read(dir.config()).unwrap(), damaged);
    let backups = fs::read_dir(&dir.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("config.broken-")
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(&backups[0]).unwrap(), damaged);
    store.save(&loaded.config).unwrap();
    assert_eq!(dir.store().load().config, loaded.config);
    assert_eq!(fs::read(&backups[0]).unwrap(), damaged);
}
#[test]
fn unknown_version_is_not_overwritten_or_backed_up_as_corrupt() {
    for original in [
        r#"{"version":99,"text_size":16}"#,
        r#"{"version":99,"form":false}"#,
        r#"{"version":99,"text_size":"large"}"#,
        r#"{"version":"future","form":false}"#,
        r#"{"version":null}"#,
        r#"{"version":-1}"#,
    ] {
        let dir = Directory::new();
        fs::write(dir.config(), original).unwrap();
        let mut store = dir.store();
        assert!(store.load().notice.contains("版本"));
        assert!(!store.is_persistent());
        assert!(store.save(&AppConfig::default()).is_err());
        assert_eq!(fs::read_to_string(dir.config()).unwrap(), original);
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
    }
}
#[test]
fn oversized_file_is_bounded_and_never_overwritten() {
    let dir = Directory::new();
    let bytes = vec![b' '; MAX_CONFIG_BYTES as usize + 1];
    fs::write(dir.config(), &bytes).unwrap();
    let mut store = dir.store();
    assert!(store.load().notice.contains("过大"));
    assert!(store.save(&AppConfig::default()).is_err());
    assert_eq!(fs::read(dir.config()).unwrap(), bytes);
}
#[test]
fn oversized_input_does_not_destroy_previous_save() {
    let dir = Directory::new();
    let mut config = AppConfig::default();
    dir.store().save(&config).unwrap();
    let original = fs::read(dir.config()).unwrap();
    config.form.set(0, 0, "1".repeat(MAX_CONFIG_BYTES as usize));
    assert!(dir.store().save(&config).is_err());
    assert_eq!(fs::read(dir.config()).unwrap(), original);
}
#[test]
fn read_failure_disables_writing_without_crashing() {
    let dir = Directory::new();
    fs::create_dir(dir.config()).unwrap();
    let mut store = dir.store();
    assert!(store.load().notice.contains("无法读取"));
    assert!(store.save(&AppConfig::default()).is_err());
    assert!(dir.config().is_dir());
}
#[test]
fn failed_rename_cleans_temporary_file_and_keeps_existing_target() {
    let dir = Directory::new();
    fs::create_dir(dir.config()).unwrap();
    assert!(dir.store().save(&AppConfig::default()).is_err());
    assert!(dir.config().is_dir());
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
#[test]
fn invalid_parent_reports_write_error() {
    let dir = Directory::new();
    let parent = dir.0.join("not-a-directory");
    fs::write(&parent, "keep").unwrap();
    assert!(ConfigStore::in_directory(&parent)
        .save(&AppConfig::default())
        .is_err());
    assert_eq!(fs::read_to_string(parent).unwrap(), "keep");
}
#[test]
fn memory_and_unavailable_stores_are_explicit() {
    let mut memory = ConfigStore::memory();
    assert!(!memory.is_persistent());
    assert!(memory.load().notice.contains("预览"));
    memory.save(&AppConfig::default()).unwrap();
    let mut unavailable = ConfigStore::unavailable("无法取得私有目录");
    assert!(unavailable.load().notice.contains("私有目录"));
    assert!(unavailable.save(&AppConfig::default()).is_err());
}
#[test]
fn private_file_permissions_are_not_world_readable() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let dir = Directory::new();
        dir.store().save(&AppConfig::default()).unwrap();
        assert_eq!(
            fs::metadata(dir.config()).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
