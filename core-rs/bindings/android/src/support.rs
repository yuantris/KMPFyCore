use jni::objects::JString;
use jni::sys::jlong;
use jni::JNIEnv;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

pub(crate) fn next_id() -> Option<u64> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| value.checked_add(1)).ok()
}

pub(crate) fn lock<'a, T>(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(crate) fn require_handle(handle: jlong, kind: &str) -> Result<u64, String> {
    if handle <= 0 {
        return Err(format!("invalid {kind} handle"));
    }
    Ok(handle as u64)
}

pub(crate) fn string_arg(env: &mut JNIEnv<'_>, value: JString) -> Result<String, ()> {
    env.get_string(&value).map(|value| value.into()).map_err(|error| {
        throw(env, error.to_string());
    })
}

pub(crate) fn throw(env: &mut JNIEnv<'_>, message: impl Into<String>) {
    let _ = env.throw_new("java/lang/RuntimeException", message.into());
}
