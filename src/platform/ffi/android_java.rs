//! Calls a static method of the app's own Java classes (`tools/android/app/java`) from the
//! game's thread, through JNI: `boolean method(Activity, String)`. Used for the photo library
//! and the log's share sheet.

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_void};

type Env = *mut *const *const c_void;
type Obj = *mut c_void;

/// A JNI argument (`jvalue`): only the object member is used.
#[repr(C)]
union Value {
    object: Obj,
    wide: u64,
}

/// Slots of the JNI function table.
const EXCEPTION_CLEAR: usize = 17;
const DELETE_LOCAL_REF: usize = 23;
const GET_OBJECT_CLASS: usize = 31;
const GET_METHOD_ID: usize = 33;
const CALL_OBJECT_METHOD_A: usize = 36;
const GET_STATIC_METHOD_ID: usize = 113;
const CALL_STATIC_BOOLEAN_METHOD_A: usize = 119;
const NEW_STRING_UTF: usize = 167;
const EXCEPTION_CHECK: usize = 228;

/// Function `slot` of the table, as the type `F` it has.
///
/// # Safety
/// `env` is a valid `JNIEnv*` of this thread, and `F` is the signature of that slot.
unsafe fn function<F: Copy>(env: Env, slot: usize) -> F {
    // SAFETY: by the caller's promise; the table has 235 entries.
    unsafe { std::mem::transmute_copy::<*const c_void, F>(&(*env).add(slot).read()) }
}

/// Whether a Java exception is pending (it is cleared).
///
/// # Safety
/// `env` as `function` asks.
unsafe fn failed(env: Env) -> bool {
    // SAFETY: by the caller's promise, with the signatures of the two slots.
    unsafe {
        let check: unsafe extern "C" fn(Env) -> u8 = function(env, EXCEPTION_CHECK);
        if check(env) == 0 {
            return false;
        }
        let clear: unsafe extern "C" fn(Env) = function(env, EXCEPTION_CLEAR);
        clear(env);
        true
    }
}

/// `class.method(activity, path)` (a static `(Landroid/app/Activity;Ljava/lang/String;)Z` method
/// of the app); None when the call could not be made.
pub fn call_with_path(class: &CStr, method: &CStr, path: &CString) -> Option<bool> {
    // SAFETY: SDL's JNI environment of this thread and its activity (null checked); every
    // call below uses the table slot of its own signature, the names and descriptors are
    // those of the Android classes and of the app's Java, and each local reference made is
    // deleted before returning.
    unsafe { call(class, method, path) }
}

/// A static Java method `class.method(Activity, String) -> boolean` of the app, called with
/// the activity and `path`; None when the call could not be made.
///
/// # Safety
/// As in `call_with_path`.
unsafe fn call(class: &CStr, method: &CStr, path: &CString) -> Option<bool> {
    use sdl3_sys::everything::{SDL_GetAndroidActivity, SDL_GetAndroidJNIEnv};
    // SAFETY: as the caller says.
    unsafe {
        let env = SDL_GetAndroidJNIEnv() as Env;
        let activity = SDL_GetAndroidActivity() as Obj;
        if env.is_null() || activity.is_null() {
            return None;
        }
        let new_string: unsafe extern "C" fn(Env, *const c_char) -> Obj =
            function(env, NEW_STRING_UTF);
        let object_class: unsafe extern "C" fn(Env, Obj) -> Obj = function(env, GET_OBJECT_CLASS);
        let method_id: unsafe extern "C" fn(Env, Obj, *const c_char, *const c_char) -> Obj =
            function(env, GET_METHOD_ID);
        let static_method_id: unsafe extern "C" fn(Env, Obj, *const c_char, *const c_char) -> Obj =
            function(env, GET_STATIC_METHOD_ID);
        let call_object: unsafe extern "C" fn(Env, Obj, Obj, *const Value) -> Obj =
            function(env, CALL_OBJECT_METHOD_A);
        let call_static_bool: unsafe extern "C" fn(Env, Obj, Obj, *const Value) -> u8 =
            function(env, CALL_STATIC_BOOLEAN_METHOD_A);
        let delete: unsafe extern "C" fn(Env, Obj) = function(env, DELETE_LOCAL_REF);

        // The app's class loader (a native thread's own cannot see the app's classes), and
        // through it the class `Photos`.
        let activity_class = object_class(env, activity);
        let class_class = object_class(env, activity_class);
        let get_loader = method_id(
            env,
            class_class,
            c"getClassLoader".as_ptr(),
            c"()Ljava/lang/ClassLoader;".as_ptr(),
        );
        let none = [Value { wide: 0 }];
        let loader = call_object(env, activity_class, get_loader, none.as_ptr());
        let loader_class = object_class(env, loader);
        let load = method_id(
            env,
            loader_class,
            c"loadClass".as_ptr(),
            c"(Ljava/lang/String;)Ljava/lang/Class;".as_ptr(),
        );
        let class_name = new_string(env, class.as_ptr());
        let by_name = [Value { object: class_name }];
        let photos = call_object(env, loader, load, by_name.as_ptr());
        let mut result = None;
        if !failed(env) && !photos.is_null() {
            let save = static_method_id(
                env,
                photos,
                method.as_ptr(),
                c"(Landroid/app/Activity;Ljava/lang/String;)Z".as_ptr(),
            );
            if !failed(env) && !save.is_null() {
                let java_path = new_string(env, path.as_ptr());
                let arguments = [Value { object: activity }, Value { object: java_path }];
                let ok = call_static_bool(env, photos, save, arguments.as_ptr());
                result = Some(ok != 0 && !failed(env));
                delete(env, java_path);
            }
        }
        failed(env);
        for local in [
            photos,
            class_name,
            loader_class,
            loader,
            class_class,
            activity_class,
        ] {
            if !local.is_null() {
                delete(env, local);
            }
        }
        result
    }
}
