fn main() {
    #[cfg(feature = "ui")]
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".into()),
    )
    .expect("Slint 手机界面编译失败");
}
