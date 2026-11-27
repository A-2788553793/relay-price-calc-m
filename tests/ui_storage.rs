#![cfg(all(feature = "ui", not(target_os = "android")))]
use relay_price_calc_m::{app::create_window_with_store, storage::ConfigStore};
use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, PointerEventButton, WindowAdapter, WindowEvent,
    },
    ComponentHandle, Model, PhysicalSize, Rgb8Pixel,
};
use std::{fs, path::PathBuf, rc::Rc};
struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
struct TemporaryDirectory(PathBuf);
impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        assert!(self
            .0
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("relay-ui-storage-"));
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn render(adapter: &MinimalSoftwareWindow) {
    adapter.set_size(PhysicalSize::new(390, 844));
    adapter.request_redraw();
    slint::platform::update_timers_and_animations();
    let mut pixels = vec![Rgb8Pixel::default(); 390 * 844];
    assert!(adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, 390);
    }));
}
fn click(adapter: &MinimalSoftwareWindow, x: f32, y: f32) {
    let position = slint::LogicalPosition::new(x, y);
    adapter.dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    adapter.dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
}
fn key(adapter: &MinimalSoftwareWindow, text: &str) {
    adapter.dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    adapter.dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}
#[test]
fn callbacks_save_immediately_and_reopened_window_restores_inputs_names_and_settings() {
    let dir = TemporaryDirectory(
        std::env::temp_dir().join(format!("relay-ui-storage-{}", std::process::id())),
    );
    fs::create_dir(&dir.0).unwrap();
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(adapter.clone()))).unwrap();
    let window = create_window_with_store(ConfigStore::in_directory(&dir.0)).unwrap();
    assert!(
        !dir.0.join("config.json").exists(),
        "opening alone must not overwrite any file"
    );
    window.invoke_name_edited(3, "保存模型四".into());
    window.invoke_field_edited(3, 0, " ３.５ ".into());
    window.invoke_field_edited(0, 4, "not-a-number".into());
    window.invoke_settings_edited(true, 18);
    let loaded = ConfigStore::in_directory(&dir.0).load().config;
    assert_eq!(loaded.form.models[3].values[0], " ３.５ ");
    assert_eq!(loaded.form.models[0].values[4], "not-a-number");
    assert_eq!(loaded.form.models[3].name, "保存模型四");
    assert!(loaded.dark_mode);
    assert_eq!(loaded.text_size, 18);
    assert!(window.get_storage_status().contains("已自动保存"));
    drop(window);
    let reopened = create_window_with_store(ConfigStore::in_directory(&dir.0)).unwrap();
    assert!(reopened.get_dark_mode());
    assert_eq!(reopened.get_text_size(), 18);
    assert_eq!(
        reopened.get_models().row_data(3).unwrap().name,
        "保存模型四"
    );
    assert_eq!(
        reopened
            .get_models()
            .row_data(3)
            .unwrap()
            .fields
            .row_data(0)
            .unwrap()
            .value,
        " ３.５ "
    );
    assert!(!reopened.get_models().row_data(0).unwrap().error.is_empty());
    reopened.invoke_clear_model_requested(3);
    let cleared = ConfigStore::in_directory(&dir.0).load().config;
    assert_eq!(cleared.form.models[3].name, "保存模型四");
    assert!(cleared.form.models[3].values.iter().all(String::is_empty));
    assert_eq!(cleared.form.models[0].values[4], "not-a-number");
    reopened.invoke_clear_requested();
    let cleared = ConfigStore::in_directory(&dir.0).load().config;
    assert!(cleared
        .form
        .models
        .iter()
        .all(|m| m.values.iter().all(String::is_empty)));
    assert!(cleared.dark_mode);
    assert_eq!(cleared.text_size, 18);
    let before = fs::read(dir.0.join("config.json")).unwrap();
    reopened.invoke_field_edited(-1, 0, "9".into());
    reopened.invoke_name_edited(4, "bad".into());
    reopened.invoke_clear_model_requested(-1);
    assert_eq!(fs::read(dir.0.join("config.json")).unwrap(), before);
    drop(reopened);
    // Actual pointer/key events: verify the visible editor, saved JSON and a
    // reopened editor agree, not just callbacks or derived coefficients.
    let typed = create_window_with_store(ConfigStore::in_directory(&dir.0)).unwrap();
    typed.invoke_settings_edited(false, 14);
    typed.invoke_name_edited(0, "清空保留名称".into());
    typed.show().unwrap();
    render(&adapter);
    assert!(typed.get_first_field_text().is_empty());
    click(&adapter, 180.0, typed.get_first_field_y());
    key(&adapter, "7");
    assert_eq!(typed.get_first_field_text(), "7");
    assert_eq!(
        ConfigStore::in_directory(&dir.0).load().config.form.models[0].values[0],
        "7"
    );
    click(
        &adapter,
        typed.get_first_clear_x(),
        typed.get_first_clear_y(),
    );
    render(&adapter);
    assert!(typed.get_first_field_text().is_empty());
    assert!(
        ConfigStore::in_directory(&dir.0).load().config.form.models[0]
            .values
            .iter()
            .all(String::is_empty)
    );
    click(&adapter, 180.0, typed.get_first_field_y());
    key(&adapter, "-");
    key(&adapter, "9");
    assert_eq!(typed.get_first_field_text(), "-9");
    assert!(!typed
        .get_models()
        .row_data(0)
        .unwrap()
        .fields
        .row_data(0)
        .unwrap()
        .error
        .is_empty());
    typed.invoke_field_edited(3, 4, "80".into());
    click(
        &adapter,
        typed.get_clear_all_button_x(),
        typed.get_clear_all_button_y(),
    );
    render(&adapter);
    assert!(typed.get_first_field_text().is_empty());
    assert_eq!(typed.get_ranking().row_count(), 0);
    let saved = ConfigStore::in_directory(&dir.0).load().config;
    assert!(saved
        .form
        .models
        .iter()
        .all(|model| model.values.iter().all(String::is_empty)));
    assert_eq!(saved.form.models[0].name, "清空保留名称");
    assert_eq!(saved.text_size, 14);
    typed.hide().unwrap();
    drop(typed);
    let restored_clear = create_window_with_store(ConfigStore::in_directory(&dir.0)).unwrap();
    restored_clear.show().unwrap();
    render(&adapter);
    assert!(
        restored_clear.get_first_field_text().is_empty(),
        "cleared visible input must stay empty after reopening"
    );
    assert_eq!(
        restored_clear.get_models().row_data(0).unwrap().name,
        "清空保留名称"
    );
    assert_eq!(
        restored_clear.get_models().row_data(3).unwrap().name,
        "保存模型四"
    );
    restored_clear.hide().unwrap();
    drop(restored_clear);
    fs::write(dir.0.join("config.json"), r#"{"version":999}"#).unwrap();
    let blocked = create_window_with_store(ConfigStore::in_directory(&dir.0)).unwrap();
    assert!(blocked.get_storage_status().contains("版本"));
    blocked.invoke_field_edited(1, 0, "9".into());
    assert_eq!(blocked.get_models().row_data(1).unwrap().coefficient, "9");
    assert!(blocked.get_storage_status().contains("不可用"));
    assert_eq!(
        fs::read_to_string(dir.0.join("config.json")).unwrap(),
        r#"{"version":999}"#
    );
    drop(blocked);

    let failing_dir = dir.0.join("write-failure");
    fs::create_dir(&failing_dir).unwrap();
    let failing = create_window_with_store(ConfigStore::in_directory(&failing_dir)).unwrap();
    // Introduce an obstructing directory after load, so the save itself fails.
    fs::create_dir(failing_dir.join("config.json")).unwrap();
    failing.invoke_field_edited(2, 0, "7".into());
    assert_eq!(failing.get_models().row_data(2).unwrap().coefficient, "7");
    assert!(failing.get_storage_status().contains("自动保存失败"));
    assert!(!failing.get_storage_status().contains("已自动保存"));
    assert!(failing_dir.join("config.json").is_dir());
}
