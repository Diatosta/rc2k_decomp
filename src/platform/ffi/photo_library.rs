//! Puts a picture file into the phone's photo library, so the Photos (iOS) or Gallery
//! (Android) app shows it. On other systems it does nothing. A failure is logged and never
//! stops the game.

#![allow(unsafe_code)]

use std::path::Path;

/// Adds the picture file `path` to the photo library (on iOS the first time asks the player for
/// the add-only permission).
pub fn add(path: &Path) {
    backend::add(path);
}

#[cfg(any(target_os = "android", target_os = "ios"))]
/// Logs one line about the outcome.
fn note(what: &str) {
    crate::logfile::program_log::log(&format!("platform: photo library: {what}"));
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod backend {
    pub fn add(_path: &std::path::Path) {}
}

/// Android: `Photos.save` (tools/android/app/java) inserts the file into the media store under
/// Pictures/Rally Championship; it is reached through JNI on the calling thread.
#[cfg(target_os = "android")]
mod backend {
    use super::note;
    use std::ffi::{CString, c_char, c_void};
    use std::path::Path;

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

    pub fn add(path: &Path) {
        let Some(text) = path.to_str().and_then(|t| CString::new(t).ok()) else {
            note("the path is not usable");
            return;
        };
        // SAFETY: SDL's JNI environment of this thread and its activity (null checked); every
        // call below uses the table slot of its own signature, the names and descriptors are
        // those of the Android classes and of `Photos.java`, and each local reference made is
        // deleted before returning.
        let saved = unsafe { save(&text) };
        match saved {
            Some(true) => note(&format!("added {}", path.display())),
            Some(false) => note(&format!("{} was refused", path.display())),
            None => note("the Java side could not be called"),
        }
    }

    /// `Photos.save(activity, path)`; None when the call could not be made.
    ///
    /// # Safety
    /// As in `add`.
    unsafe fn save(path: &CString) -> Option<bool> {
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
            let object_class: unsafe extern "C" fn(Env, Obj) -> Obj =
                function(env, GET_OBJECT_CLASS);
            let method_id: unsafe extern "C" fn(Env, Obj, *const c_char, *const c_char) -> Obj =
                function(env, GET_METHOD_ID);
            let static_method_id: unsafe extern "C" fn(
                Env,
                Obj,
                *const c_char,
                *const c_char,
            ) -> Obj = function(env, GET_STATIC_METHOD_ID);
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
            let class_name = new_string(env, c"org.rc2k.rally.Photos".as_ptr());
            let by_name = [Value { object: class_name }];
            let photos = call_object(env, loader, load, by_name.as_ptr());
            let mut result = None;
            if !failed(env) && !photos.is_null() {
                let save = static_method_id(
                    env,
                    photos,
                    c"save".as_ptr(),
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
}

/// iOS: `UIImageWriteToSavedPhotosAlbum` on the main thread, which adds the picture to the
/// camera roll with the add-only permission (`NSPhotoLibraryAddUsageDescription`).
#[cfg(target_os = "ios")]
mod backend {
    use super::note;
    use std::ffi::{CString, c_char, c_void};
    use std::path::Path;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    #[link(name = "UIKit", kind = "framework")]
    unsafe extern "C" {
        fn UIImageWriteToSavedPhotosAlbum(
            image: Id,
            target: Id,
            selector: Sel,
            context: *mut c_void,
        );
    }

    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
    }

    /// Runs on the main thread: `context` is the path as a leaked `CString`.
    extern "C" fn save(context: *mut c_void) {
        // SAFETY: `context` came from `CString::into_raw` in `add` and is taken back once.
        let path = unsafe { CString::from_raw(context.cast()) };
        // SAFETY: the Objective-C runtime's calls with the types of the methods named
        // (`objc_msgSend` cast to each message's signature); the classes are looked up by name
        // and null checked, the image is checked before use and lives in the main queue's
        // autorelease pool, which the save call retains it past.
        let image = unsafe {
            let string_class = objc_getClass(c"NSString".as_ptr());
            let image_class = objc_getClass(c"UIImage".as_ptr());
            if string_class.is_null() || image_class.is_null() {
                std::ptr::null_mut()
            } else {
                let send_string: unsafe extern "C" fn(Id, Sel, *const c_char) -> Id =
                    std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                let send_image: unsafe extern "C" fn(Id, Sel, Id) -> Id =
                    std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                let ns_path = send_string(
                    string_class,
                    sel_registerName(c"stringWithUTF8String:".as_ptr()),
                    path.as_ptr(),
                );
                send_image(
                    image_class,
                    sel_registerName(c"imageWithContentsOfFile:".as_ptr()),
                    ns_path,
                )
            }
        };
        if image.is_null() {
            note(&format!(
                "{} could not be read as a picture",
                path.to_string_lossy()
            ));
            return;
        }
        // SAFETY: a UIImage (not null) with no completion target, which the function allows.
        unsafe {
            UIImageWriteToSavedPhotosAlbum(
                image,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        note(&format!(
            "{} sent to the camera roll",
            path.to_string_lossy()
        ));
    }

    pub fn add(path: &Path) {
        let Some(text) = path.to_str().and_then(|t| CString::new(t).ok()) else {
            note("the path is not usable");
            return;
        };
        // SAFETY: the main queue is a constant of libdispatch; the context is a `CString` that
        // `save` takes back.
        unsafe {
            dispatch_async_f(&raw const _dispatch_main_q, text.into_raw().cast(), save);
        }
    }
}
