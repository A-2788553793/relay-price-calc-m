#![cfg(all(feature = "ui", not(target_os = "android")))]

use relay_price_calc_m::app::{create_window, MainWindow, Theme};
use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, PointerEventButton, WindowAdapter, WindowEvent,
    },
    ComponentHandle, Model, PhysicalSize, Rgb8Pixel,
};
use std::{cell::Cell, fs, io::Write, rc::Rc};

struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn snapshot(adapter: &MinimalSoftwareWindow, name: &str, width: u32, height: u32) {
    adapter.set_size(PhysicalSize::new(width, height));
    adapter.request_redraw();
    slint::platform::update_timers_and_animations();
    let mut pixels = vec![Rgb8Pixel::default(); (width * height) as usize];
    assert!(adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, width as usize);
    }));
    assert!(
        pixels.iter().any(|p| *p != pixels[0]),
        "nonempty rendered screen"
    );
    if std::env::var_os("RELAY_UI_SNAPSHOTS").is_some() {
        fs::create_dir_all("target/ui-shots").unwrap();
        let mut out = fs::File::create(format!("target/ui-shots/{name}.ppm")).unwrap();
        write!(out, "P6\n{width} {height}\n255\n").unwrap();
        let bytes: Vec<u8> = pixels.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
        out.write_all(&bytes).unwrap();
    }
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

fn assert_touch_target(name: &str, bounds: [f32; 4], viewport: [f32; 2]) {
    let [x, y, width, height] = bounds;
    let [window_width, window_height] = viewport;
    assert!(
        width >= 44.0 && height >= 44.0,
        "{name}: touch target too small: {bounds:?}"
    );
    assert!(
        x - width / 2.0 >= -0.5
            && y - height / 2.0 >= -0.5
            && x + width / 2.0 <= window_width + 0.5
            && y + height / 2.0 <= window_height + 0.5,
        "{name}: bounds {bounds:?} leave viewport {viewport:?}"
    );
}

fn assert_toolbar(window: &MainWindow, width: f32, height: f32) {
    for (name, bounds) in [
        (
            "settings",
            [
                window.get_settings_button_x(),
                window.get_settings_button_y(),
                window.get_settings_button_width(),
                window.get_settings_button_height(),
            ],
        ),
        (
            "about",
            [
                window.get_about_button_x(),
                window.get_about_button_y(),
                window.get_about_button_width(),
                window.get_about_button_height(),
            ],
        ),
        (
            "clear-all",
            [
                window.get_clear_all_button_x(),
                window.get_clear_all_button_y(),
                window.get_clear_all_button_width(),
                window.get_clear_all_button_height(),
            ],
        ),
    ] {
        assert_touch_target(name, bounds, [width, height]);
    }
}

fn drag(adapter: &MinimalSoftwareWindow, x: f32, from_y: f32, to_y: f32) {
    adapter.dispatch_event(WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(x, from_y),
        button: PointerEventButton::Left,
    });
    for step in 1..=12 {
        adapter.dispatch_event(WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(x, from_y + (to_y - from_y) * step as f32 / 12.0),
        });
    }
    adapter.dispatch_event(WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(x, to_y),
        button: PointerEventButton::Left,
    });
}

fn key(adapter: &MinimalSoftwareWindow, text: &str) {
    adapter.dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    adapter.dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}

fn coefficient(window: &MainWindow, index: usize) -> slint::SharedString {
    window.get_models().row_data(index).unwrap().coefficient
}

fn top(adapter: &MinimalSoftwareWindow, window: &MainWindow, height: f32) {
    for _ in 0..100 {
        if window.get_content_offset().abs() < 2.0 {
            break;
        }
        drag(adapter, 20.0, window.get_scroll_top() + 20.0, height - 25.0);
    }
    assert!(
        window.get_content_offset().abs() < 2.0,
        "reverse drag reaches top"
    );
}

#[test]
fn mobile_ui_callbacks_rendering_and_resizing() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(adapter.clone()))).unwrap();
    // create_window is intentionally memory-only: no test touches real app storage.
    let window = create_window().unwrap();
    window.show().unwrap();
    assert_eq!(window.get_models().row_count(), 4);
    assert_eq!(window.get_ranking().row_count(), 0);
    for model in window.get_models().iter() {
        assert_eq!(model.fields.row_count(), 5);
        assert_eq!(model.coefficient, "—");
    }
    for (index, text) in ["3", "15", "3.75", "0.3", "80"].iter().enumerate() {
        window.invoke_field_edited(0, index as i32, (*text).into());
    }
    for (index, price) in [(1, "20"), (2, "30"), (3, "40")] {
        window.invoke_field_edited(index, 1, price.into());
    }
    assert_eq!(coefficient(&window, 0), "6.84");
    assert_eq!(coefficient(&window, 1), "20");
    assert_eq!(coefficient(&window, 2), "30");
    assert_eq!(coefficient(&window, 3), "40");
    assert!(window.get_ranking().row_data(0).unwrap().best);
    assert_eq!(
        window.get_ranking().row_data(0).unwrap().coefficient,
        "6.84"
    );
    assert_eq!(
        window
            .get_models()
            .row_data(0)
            .unwrap()
            .fields
            .row_data(4)
            .unwrap()
            .value,
        "80"
    );
    snapshot(&adapter, "light", 390, 844);
    snapshot(&adapter, "narrow", 320, 640);
    assert_toolbar(&window, 320.0, 640.0);
    snapshot(&adapter, "keyboard", 390, 360);
    assert_toolbar(&window, 390.0, 360.0);
    window.set_dark_mode(true);
    snapshot(&adapter, "dark", 390, 844);
    assert!(window.global::<Theme>().get_dark());
    window.invoke_field_edited(0, 4, "101".into());
    assert_eq!(coefficient(&window, 0), "—");
    assert_eq!(coefficient(&window, 1), "20");
    assert!(!window
        .get_models()
        .row_data(0)
        .unwrap()
        .fields
        .row_data(4)
        .unwrap()
        .error
        .is_empty());
    snapshot(&adapter, "invalid", 390, 844);
    window.invoke_field_edited(0, 4, "８０％".into());
    assert_eq!(coefficient(&window, 0), "6.84");
    window.invoke_name_edited(3, "第四个模型".into());
    assert_eq!(window.get_models().row_data(3).unwrap().name, "第四个模型");
    window.invoke_clear_model_requested(2);
    assert_eq!(coefficient(&window, 2), "—");
    assert_eq!(window.get_ranking().row_count(), 3);
    assert_eq!(coefficient(&window, 0), "6.84");
    window.invoke_clear_requested();
    assert_eq!(window.get_ranking().row_count(), 0);
    for model in window.get_models().iter() {
        assert_eq!(model.coefficient, "—");
        for row in model.fields.iter() {
            assert!(row.value.is_empty() && row.error.is_empty());
        }
    }

    window.set_dark_mode(false);
    window.set_text_size(12);
    snapshot(&adapter, "small-toolbar", 320, 640);
    assert_toolbar(&window, 320.0, 640.0);
    window.set_text_size(14);
    window.invoke_field_edited(3, 0, "8".into());
    snapshot(&adapter, "before-clear-click", 390, 844);
    assert_toolbar(&window, 390.0, 844.0);
    click(
        &adapter,
        window.get_clear_all_button_x(),
        window.get_clear_all_button_y(),
    );
    assert_eq!(
        window.get_ranking().row_count(),
        0,
        "real clear-all click empties ranking"
    );
    assert_eq!(coefficient(&window, 3), "—");
    snapshot(&adapter, "cleared", 390, 844);
    // Fixed two-row toolbar at scale 1: Settings, About, Clear all.
    click(
        &adapter,
        window.get_settings_button_x(),
        window.get_settings_button_y(),
    );
    assert!(window.get_show_settings());
    snapshot(&adapter, "settings", 390, 844);
    // Observe actual clicked values while forwarding the normal settings contract.
    let settings_calls = Rc::new(Cell::new(0));
    let observed = Rc::new(Cell::new((false, 14)));
    let calls = settings_calls.clone();
    let values = observed.clone();
    let weak = window.as_weak();
    window.on_settings_edited(move |dark, size| {
        calls.set(calls.get() + 1);
        values.set((dark, size));
        if let Some(window) = weak.upgrade() {
            window.set_dark_mode(dark);
            window.set_text_size(size);
        }
    });
    click(&adapter, 180.0, window.get_theme_button_y());
    assert!(window.get_dark_mode());
    assert!(window.global::<Theme>().get_dark());
    assert_eq!(observed.get(), (true, 14));
    click(
        &adapter,
        window.get_larger_button_x(),
        window.get_larger_button_y(),
    );
    assert_eq!(window.get_text_size(), 15);
    assert_eq!(observed.get(), (true, 15));
    assert_eq!(settings_calls.get(), 2);
    window.set_text_size(14);
    assert_touch_target(
        "settings_close_button",
        [
            window.get_settings_close_button_x(),
            window.get_settings_close_button_y(),
            window.get_settings_close_button_width(),
            window.get_settings_close_button_height(),
        ],
        [390.0, 844.0],
    );
    click(
        &adapter,
        window.get_settings_close_button_x(),
        window.get_settings_close_button_y(),
    );
    assert!(!window.get_show_settings());
    click(
        &adapter,
        window.get_about_button_x(),
        window.get_about_button_y(),
    );
    assert!(window.get_show_about());
    snapshot(&adapter, "about-click", 390, 844);
    let project_opened = Rc::new(Cell::new(false));
    let opened = project_opened.clone();
    window.on_open_project_requested(move || opened.set(true));
    click(&adapter, 180.0, window.get_project_button_y());
    assert!(
        project_opened.get(),
        "project button calls external browser callback"
    );
    assert_touch_target(
        "about_close_button",
        [
            window.get_about_close_button_x(),
            window.get_about_close_button_y(),
            window.get_about_close_button_width(),
            window.get_about_close_button_height(),
        ],
        [390.0, 844.0],
    );
    click(
        &adapter,
        window.get_about_close_button_x(),
        window.get_about_close_button_y(),
    );
    assert!(!window.get_show_about());

    top(&adapter, &window, 844.0);
    snapshot(&adapter, "input-start", 390, 844);
    let field_y = window.get_first_field_y();
    assert!(
        field_y > window.get_scroll_top() && field_y < 800.0,
        "first input is visible: {field_y}"
    );
    click(&adapter, 180.0, field_y);
    key(&adapter, "7");
    assert_eq!(
        window.get_first_field_text(),
        "7",
        "visible editor receives typed value"
    );
    assert_eq!(
        coefficient(&window, 0),
        "7",
        "real input event changes coefficient"
    );
    click(
        &adapter,
        window.get_first_clear_x(),
        window.get_first_clear_y(),
    );
    assert_eq!(coefficient(&window, 0), "—", "real per-model clear button");
    assert_eq!(window.get_ranking().row_count(), 0);
    snapshot(&adapter, "typed-per-model-cleared", 390, 844);
    assert!(
        window.get_first_field_text().is_empty(),
        "per-model clear must empty the visible editor after actual typing"
    );
    // Repeated edits after clearing must keep both directions alive, including
    // external model changes while the editor is focused and invalid text.
    click(&adapter, 180.0, window.get_first_field_y());
    key(&adapter, "9");
    assert_eq!(window.get_first_field_text(), "9");
    assert_eq!(coefficient(&window, 0), "9");
    window.invoke_field_edited(0, 0, "12".into());
    snapshot(&adapter, "focused-external-update", 390, 844);
    assert_eq!(window.get_first_field_text(), "12");
    key(&adapter, "\u{f72b}"); // End
    key(&adapter, "\u{0008}");
    key(&adapter, "\u{0008}");
    key(&adapter, "-");
    key(&adapter, "3");
    assert_eq!(window.get_first_field_text(), "-3");
    assert_eq!(coefficient(&window, 0), "—");
    click(
        &adapter,
        window.get_clear_all_button_x(),
        window.get_clear_all_button_y(),
    );
    snapshot(&adapter, "typed-all-cleared", 390, 844);
    assert!(
        window.get_first_field_text().is_empty(),
        "clear-all must empty an edited invalid field on screen"
    );
    assert_eq!(window.get_ranking().row_count(), 0);
    click(&adapter, 180.0, window.get_first_field_y());
    key(&adapter, "4");
    assert_eq!(window.get_first_field_text(), "4");
    assert_eq!(coefficient(&window, 0), "4");
    click(
        &adapter,
        window.get_clear_all_button_x(),
        window.get_clear_all_button_y(),
    );
    snapshot(&adapter, "typed-all-cleared-again", 390, 844);
    assert!(
        window.get_first_field_text().is_empty(),
        "repeated clear-all must stay synchronized"
    );
    // Exercise text editing, not only an invoked Rust callback.
    click(&adapter, 180.0, window.get_first_name_y());
    key(&adapter, "甲");
    assert!(window.get_models().row_data(0).unwrap().name.contains('甲'));
    window.invoke_name_edited(0, "甲".repeat(29).into());
    snapshot(&adapter, "name-limit-start", 390, 844);
    click(&adapter, 180.0, window.get_first_name_y());
    key(&adapter, "\u{f72b}"); // End: append to the model name, independent of click cursor position.
    key(&adapter, "乙");
    assert_eq!(window.get_first_name_text().chars().count(), 30);
    key(&adapter, "丙"); // The rejected 31st character must not remain in the visible editor.
    assert_eq!(window.get_first_name_text().chars().count(), 30);
    assert_eq!(
        window.get_first_name_text(),
        window.get_models().row_data(0).unwrap().name
    );
    key(&adapter, "\u{f72b}");
    for _ in 0..30 {
        key(&adapter, "\u{0008}");
    }
    assert!(
        window.get_first_name_text().is_empty(),
        "clearing a name must not force its fallback display name into the editor"
    );
    assert!(window.get_models().row_data(0).unwrap().name.is_empty());
    key(&adapter, "新");
    key(&adapter, "名");
    assert_eq!(window.get_first_name_text(), "新名");
    assert_eq!(window.get_models().row_data(0).unwrap().name, "新名");
    window.invoke_name_edited(0, "模型1".into());
    // Slint changed callbacks run on the next event/render, not in a Rust setter.
    snapshot(&adapter, "external-name-updated", 390, 844);
    assert_eq!(
        window.get_first_name_text(),
        "模型1",
        "external name changes remain synchronized after normalization"
    );
    snapshot(&adapter, "real-input", 390, 844);
    let before = window.get_content_offset();
    drag(&adapter, 20.0, 760.0, 330.0);
    assert!(
        window.get_content_offset() < before - 100.0,
        "finger drag must move content"
    );
    for _ in 0..30 {
        drag(&adapter, 20.0, 760.0, 330.0);
    }
    let bottom = window.get_content_visible_height() - window.get_content_height();
    assert!(
        window.get_content_offset() <= bottom + 2.0,
        "rankings and bottom content are reachable"
    );
    snapshot(&adapter, "drag-bottom", 390, 844);
    top(&adapter, &window, 844.0);
    snapshot(&adapter, "drag-top", 390, 844);
    let input_y = window.get_first_field_y();
    drag(&adapter, 180.0, input_y, window.get_scroll_top() + 20.0);
    assert!(
        window.get_content_offset() < -80.0,
        "editable field must not trap vertical drags"
    );
    snapshot(&adapter, "keyboard-drag-start", 390, 360);
    top(&adapter, &window, 360.0);
    assert!(
        window.get_content_visible_height() > 150.0,
        "keyboard-sized viewport remains useful"
    );
    for _ in 0..100 {
        drag(&adapter, 20.0, 335.0, window.get_scroll_top() + 20.0);
    }
    let keyboard_bottom = window.get_content_visible_height() - window.get_content_height();
    assert!(
        window.get_content_offset() <= keyboard_bottom + 2.0,
        "bottom reachable in keyboard viewport"
    );
    snapshot(&adapter, "keyboard-drag-bottom", 390, 360);

    // Unbroken Chinese names wrap instead of eliding; all pages scale together.
    for index in 0..4 {
        window.invoke_name_edited(
            index,
            "这是一个用于验证窄屏大字号仍然完整换行显示的超长中文模型名称"
                .repeat(3)
                .into(),
        );
    }
    window.set_text_size(20);
    window.set_dark_mode(false);
    snapshot(&adapter, "large-long-light", 320, 640);
    assert_toolbar(&window, 320.0, 640.0);
    assert!((window.global::<Theme>().get_scale() - 20.0 / 14.0).abs() < 0.01);
    top(&adapter, &window, 640.0);
    snapshot(&adapter, "large-long-top", 320, 640);
    window.set_dark_mode(true);
    snapshot(&adapter, "large-long-dark", 320, 640);
    window.set_show_settings(true);
    snapshot(&adapter, "settings-large-keyboard", 390, 360);
    assert_touch_target(
        "settings-close-large",
        [
            window.get_settings_close_button_x(),
            window.get_settings_close_button_y(),
            window.get_settings_close_button_width(),
            window.get_settings_close_button_height(),
        ],
        [390.0, 360.0],
    );
    assert!(window.get_settings_height() > window.get_settings_visible_height());
    for _ in 0..40 {
        drag(&adapter, 20.0, 330.0, 120.0);
    }
    assert!(
        window.get_settings_offset()
            <= window.get_settings_visible_height() - window.get_settings_height() + 2.0
    );
    snapshot(&adapter, "settings-bottom", 390, 360);
    window.set_show_settings(false);
    window.set_show_about(true);
    snapshot(&adapter, "about-large-keyboard", 390, 360);
    assert_touch_target(
        "about-close-large",
        [
            window.get_about_close_button_x(),
            window.get_about_close_button_y(),
            window.get_about_close_button_width(),
            window.get_about_close_button_height(),
        ],
        [390.0, 360.0],
    );
    assert!(window.get_about_height() > window.get_about_visible_height());
    for _ in 0..40 {
        drag(&adapter, 20.0, 330.0, 120.0);
    }
    assert!(
        window.get_about_offset()
            <= window.get_about_visible_height() - window.get_about_height() + 2.0
    );
    snapshot(&adapter, "about-bottom", 390, 360);
    window.hide().unwrap();
}
