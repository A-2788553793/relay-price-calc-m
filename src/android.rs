//! 只调用系统浏览器；应用自身不联网，也不申请网络权限。
use jni::{
    objects::{JObject, JValue},
    JavaVM,
};

pub fn open_project(app: &slint::android::AndroidApp) -> Result<(), String> {
    // SAFETY: VM 与 Activity 由仍存活的 AndroidApp 提供；不获取引用所有权。
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }
        .map_err(|_| "无法连接系统浏览器接口")?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|_| "无法连接系统浏览器接口")?;
    let result = env.with_local_frame(16, |env| -> jni::errors::Result<()> {
        let raw_activity = app.activity_as_ptr().cast::<jni::sys::_jobject>();
        // SAFETY: JObject is repr(transparent) over jobject. Borrow AndroidApp's
        // global reference while app is alive; create a local reference in this frame.
        let borrowed: &JObject<'_> =
            unsafe { &*(&raw_activity as *const jni::sys::jobject).cast::<JObject<'_>>() };
        let activity = env.new_local_ref(borrowed)?;
        let action = env.new_string("android.intent.action.VIEW")?;
        let url = env.new_string(crate::app::PROJECT_URL)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(url.as_ref())],
            )?
            .l()?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[JValue::Object(action.as_ref()), JValue::Object(&uri)],
        )?;
        env.call_method(
            &activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )?;
        Ok(())
    });
    if result.is_err() {
        // ActivityNotFound/SecurityException 不能遗留为 pending JNI exception。
        let _ = env.exception_clear();
        return Err("未能打开浏览器，请手动访问关于页项目地址".into());
    }
    Ok(())
}
