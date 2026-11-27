#![cfg(all(feature = "ui", not(target_os = "android")))]

use relay_price_calc_m::app::{create_window, Theme};
use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, PointerEventButton, WindowAdapter, WindowEvent,
    },
    ComponentHandle, Model, PhysicalSize, Rgb8Pixel,
};
use std::{fs, io::Write, rc::Rc};

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

#[test]
fn mobile_ui_callbacks_rendering_and_resizing() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(adapter.clone()))).unwrap();
    let window = create_window().unwrap();
    window.show().unwrap();
    assert_eq!(window.get_coefficient_one(), "0");
    assert_eq!(window.get_winner(), 0);
    for (index, text) in ["3", "15", "3.75", "0.3", "80"].iter().enumerate() {
        window.invoke_field_edited(0, index as i32, (*text).into());
    }
    window.invoke_field_edited(1, 1, "20".into());
    assert_eq!(window.get_coefficient_one(), "6.84");
    assert_eq!(window.get_coefficient_two(), "20");
    assert_eq!(window.get_winner(), 1);
    assert!(window.get_detail().contains("65.8%"));
    assert_eq!(window.get_model_one().row_count(), 5);
    assert_eq!(window.get_model_one().row_data(4).unwrap().value, "80");
    snapshot(&adapter, "light", 390, 844);
    snapshot(&adapter, "narrow", 320, 640);
    snapshot(&adapter, "keyboard", 390, 360);
    window.global::<Theme>().set_dark(true);
    snapshot(&adapter, "dark", 390, 844);
    window.invoke_field_edited(0, 4, "101".into());
    assert_eq!(window.get_winner(), -1);
    assert_eq!(window.get_coefficient_one(), "—");
    assert_eq!(window.get_coefficient_two(), "20");
    assert!(!window.get_model_one().row_data(4).unwrap().error.is_empty());
    snapshot(&adapter, "invalid", 390, 844);
    window.invoke_field_edited(0, 4, "８０％".into());
    assert_eq!(window.get_coefficient_one(), "6.84");
    window.set_show_about(true);
    snapshot(&adapter, "about", 390, 844);
    window.set_show_about(false);
    window.invoke_clear_requested();
    assert_eq!(window.get_winner(), 0);
    assert_eq!(window.get_coefficient_one(), "0");
    assert_eq!(window.get_coefficient_two(), "0");
    for model in [window.get_model_one(), window.get_model_two()] {
        for index in 0..5 {
            let row = model.row_data(index).unwrap();
            assert!(row.value.is_empty() && row.error.is_empty());
        }
    }
    // 根据390px布局的真实控件位置，执行按钮和输入事件。
    let click = |x, y| {
        let position = slint::LogicalPosition::new(x, y);
        adapter.dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        adapter.dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    };
    click(45.0, 94.0);
    assert!(!window.global::<Theme>().get_dark());
    click(340.0, 38.0);
    assert!(window.get_show_about());
    snapshot(&adapter, "about-click", 390, 844);
    click(280.0, 42.0);
    assert!(!window.get_show_about());
    snapshot(&adapter, "cleared", 390, 844);
    click(175.0, 335.0);
    adapter.dispatch_event(WindowEvent::KeyPressed { text: "7".into() });
    adapter.dispatch_event(WindowEvent::KeyReleased { text: "7".into() });
    assert_eq!(window.get_coefficient_one(), "7");
    click(120.0, 94.0);
    assert_eq!(window.get_coefficient_one(), "0");
    adapter.dispatch_event(WindowEvent::PointerScrolled {
        position: slint::LogicalPosition::new(200.0, 650.0),
        delta_x: 0.0,
        delta_y: -1600.0,
    });
    snapshot(&adapter, "scrolled", 390, 844);
    window.hide().unwrap();
}
