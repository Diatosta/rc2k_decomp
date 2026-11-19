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
/// Pictures/Rally Championship; it is reached through JNI on the calling thread
/// (`android_java`).
#[cfg(target_os = "android")]
mod backend {
    use super::note;
    use std::ffi::CString;
    use std::path::Path;

    pub fn add(path: &Path) {
        let Some(text) = path.to_str().and_then(|t| CString::new(t).ok()) else {
            note("the path is not usable");
            return;
        };
        match crate::platform::ffi::android_java::call_with_path(
            c"org.rc2k.rally.Photos",
            c"save",
            &text,
        ) {
            Some(true) => note(&format!("added {}", path.display())),
            Some(false) => note(&format!("{} was refused", path.display())),
            None => note("the Java side could not be called"),
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
