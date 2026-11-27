//! relay-price-calc-m 的计算核心及 Android 入口。
pub mod calc;
pub mod model;

#[cfg(feature = "ui")]
pub mod app;

#[cfg(all(target_os = "android", feature = "ui"))]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).expect("Android 界面初始化失败");
    app::run().expect("手机界面运行失败");
}
