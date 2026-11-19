//! Puts a picture file into the phone's photo library, so the Photos (iOS) or Gallery
//! (Android) app shows it. On other systems it does nothing. A failure is logged and never
//! stops the game.

#![allow(unsafe_code)]

use std::path::Path;

/// Adds the picture file `path` to the photo library (on iOS into the album "Rally Championship"; the first time
/// asks the player for access to the photos).
pub fn add(path: &Path) {
    backend::add(path);
}

#[cfg(phone)]
/// Logs one line about the outcome.
fn note(what: &str) {
    crate::program_log::log(&format!("platform: photo library: {what}"));
}

#[cfg(not(phone))]
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

/// iOS: the picture goes into the album "Rally Championship" (made when missing) through the
/// Photos framework, which needs the read-write permission (`NSPhotoLibraryUsageDescription`).
/// When the player refuses it, or the framework fails, the picture goes into the camera roll
/// with `UIImageWriteToSavedPhotosAlbum` (the add-only permission,
/// `NSPhotoLibraryAddUsageDescription`), as before.
///
/// Everything is one job at a time, started and finished on the main thread: the pictures wait
/// in `STATE.paths`; the framework's two completion points (the permission question, the
/// change) hop back to the main thread through `dispatch_async_f`.
///
/// The framework's calls take Objective-C blocks. Every block here is a *global* block: one that
/// captures nothing, which is a constant struct the runtime never copies or frees (the block
/// ABI: `isa`, `flags`, `reserved`, the function, a descriptor with the struct's size). What a
/// block needs is read from the statics `STATE` and `JOB`.
#[cfg(target_os = "ios")]
mod backend {
    use super::note;
    use std::collections::VecDeque;
    use std::ffi::{CString, c_char, c_void};
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

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

    // The framework is linked through its class `PHPhotoLibrary`, whose address is the class.
    #[link(name = "Photos", kind = "framework")]
    unsafe extern "C" {
        #[link_name = "OBJC_CLASS_$_PHPhotoLibrary"]
        static PHOTO_LIBRARY_CLASS: c_void;
    }

    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        static _NSConcreteGlobalBlock: c_void;
    }

    /// `PHAccessLevelReadWrite`.
    const ACCESS_READ_WRITE: isize = 2;
    /// `PHAuthorizationStatusNotDetermined`, `Authorized` and `Limited`.
    const STATUS_NOT_DETERMINED: isize = 0;
    const STATUS_AUTHORIZED: isize = 3;
    const STATUS_LIMITED: isize = 4;
    /// `PHAssetCollectionTypeAlbum` and `PHAssetCollectionSubtypeAlbumRegular`: the player's
    /// own albums.
    const TYPE_ALBUM: isize = 1;
    const SUBTYPE_ALBUM_REGULAR: isize = 2;
    /// The album's name.
    const ALBUM_NAME: &std::ffi::CStr = c"Rally Championship";

    /// The block ABI's flag for a global block.
    const BLOCK_IS_GLOBAL: i32 = 1 << 28;

    #[repr(C)]
    struct Descriptor {
        reserved: usize,
        size: usize,
    }

    /// An Objective-C block with no captured values; `F` is the function it runs, which takes the
    /// block itself first.
    #[repr(C)]
    struct Block<F> {
        isa: *const c_void,
        flags: i32,
        reserved: i32,
        invoke: F,
        descriptor: &'static Descriptor,
    }

    // SAFETY: a global block is a constant: its pointers name the runtime's class symbol, a
    // function and a constant descriptor, and nothing ever writes it.
    unsafe impl<F> Sync for Block<F> {}

    impl<F> Block<F> {
        const fn global(invoke: F, descriptor: &'static Descriptor) -> Self {
            Self {
                // SAFETY of the address: only the symbol's address is taken, never its contents.
                isa: &raw const _NSConcreteGlobalBlock,
                flags: BLOCK_IS_GLOBAL,
                reserved: 0,
                invoke,
                descriptor,
            }
        }

        /// The block as the `id` a message takes.
        fn id(&'static self) -> Id {
            std::ptr::from_ref(self).cast_mut().cast()
        }
    }

    const fn descriptor_for<F>() -> Descriptor {
        Descriptor {
            reserved: 0,
            size: size_of::<Block<F>>(),
        }
    }

    /// Runs on the Photos framework's queue; see `changes`.
    type ChangesFn = extern "C" fn(*const c_void);
    /// `(BOOL success, NSError *error)`.
    type CompletionFn = extern "C" fn(*const c_void, bool, Id);
    /// `(PHAuthorizationStatus status)`.
    type AuthFn = extern "C" fn(*const c_void, isize);

    static CHANGES_DESCRIPTOR: Descriptor = descriptor_for::<ChangesFn>();
    static COMPLETION_DESCRIPTOR: Descriptor = descriptor_for::<CompletionFn>();
    static AUTH_DESCRIPTOR: Descriptor = descriptor_for::<AuthFn>();
    static CHANGES_BLOCK: Block<ChangesFn> = Block::global(changes, &CHANGES_DESCRIPTOR);
    static COMPLETION_BLOCK: Block<CompletionFn> = Block::global(completed, &COMPLETION_DESCRIPTOR);
    static AUTH_BLOCK: Block<AuthFn> = Block::global(authorised, &AUTH_DESCRIPTOR);

    /// The pictures waiting, the first being the one in work.
    struct State {
        paths: VecDeque<CString>,
        busy: bool,
    }
    static STATE: Mutex<State> = Mutex::new(State {
        paths: VecDeque::new(),
        busy: false,
    });
    /// What `changes` works on: the picture's path and the album found (retained, as a number),
    /// or 0 when it has to be made.
    struct Job {
        path: Option<CString>,
        album: usize,
    }
    static JOB: Mutex<Job> = Mutex::new(Job {
        path: None,
        album: 0,
    });
    static AUTH_STATUS: AtomicIsize = AtomicIsize::new(STATUS_NOT_DETERMINED);
    static LAST_OK: AtomicBool = AtomicBool::new(false);

    fn sel(name: &std::ffi::CStr) -> Sel {
        // SAFETY: `name` is a NUL-terminated string; the call registers it.
        unsafe { sel_registerName(name.as_ptr()) }
    }

    fn class(name: &std::ffi::CStr) -> Id {
        // SAFETY: `name` is a NUL-terminated string; a class not found gives null, which every
        // user below checks.
        unsafe { objc_getClass(name.as_ptr()) }
    }

    /// `objc_msgSend` as the function type of the message being sent.
    ///
    /// # Safety
    /// `F` must be the exact C signature (receiver, selector, arguments, result) of the method
    /// the selector names on the receiver sent to.
    unsafe fn msg<F: Copy>() -> F {
        const { assert!(size_of::<F>() == size_of::<unsafe extern "C" fn()>()) };
        // SAFETY: both are function pointers of one size; the caller vouches for the signature.
        unsafe {
            std::mem::transmute_copy::<unsafe extern "C" fn(), F>(
                &(objc_msgSend as unsafe extern "C" fn()),
            )
        }
    }

    /// `[recv name]`.
    fn send0(recv: Id, name: &std::ffi::CStr) -> Id {
        // SAFETY: a message with no arguments that returns an object (every use is a getter
        // or a constructor of that shape); the receiver is a live object or class.
        unsafe { msg::<unsafe extern "C" fn(Id, Sel) -> Id>()(recv, sel(name)) }
    }

    /// `[recv name:arg]` with an object argument and result.
    fn send1(recv: Id, name: &std::ffi::CStr, arg: Id) -> Id {
        // SAFETY: a message with one object argument returning an object, to a live receiver.
        unsafe { msg::<unsafe extern "C" fn(Id, Sel, Id) -> Id>()(recv, sel(name), arg) }
    }

    fn ns_string(text: &std::ffi::CStr) -> Id {
        // SAFETY: `+stringWithUTF8String:` takes a NUL-terminated string and copies it.
        unsafe {
            msg::<unsafe extern "C" fn(Id, Sel, *const c_char) -> Id>()(
                class(c"NSString"),
                sel(c"stringWithUTF8String:"),
                text.as_ptr(),
            )
        }
    }

    fn responds(recv: Id, name: &std::ffi::CStr) -> bool {
        // SAFETY: `-respondsToSelector:` returns a BOOL (a `bool` on arm64).
        unsafe {
            msg::<unsafe extern "C" fn(Id, Sel, Sel) -> bool>()(
                recv,
                sel(c"respondsToSelector:"),
                sel(name),
            )
        }
    }

    fn library_class() -> Id {
        (&raw const PHOTO_LIBRARY_CLASS).cast_mut().cast()
    }

    fn on_main(work: extern "C" fn(*mut c_void)) {
        // SAFETY: the main queue is a constant of libdispatch; the work takes no context.
        unsafe { dispatch_async_f(&raw const _dispatch_main_q, std::ptr::null_mut(), work) };
    }

    pub fn add(path: &Path) {
        let Some(text) = path.to_str().and_then(|t| CString::new(t).ok()) else {
            note("the path is not usable");
            return;
        };
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        state.paths.push_back(text);
        if !state.busy {
            state.busy = true;
            on_main(begin);
        }
    }

    /// Main thread: starts the job on the first waiting picture.
    extern "C" fn begin(_: *mut c_void) {
        let library = library_class();
        if !responds(library, c"authorizationStatusForAccessLevel:") {
            note("this iOS has no read-write photo access");
            finish_failed();
            return;
        }
        // SAFETY: a class method taking a `PHAccessLevel` (an integer) and returning one.
        let status = unsafe {
            msg::<unsafe extern "C" fn(Id, Sel, isize) -> isize>()(
                library,
                sel(c"authorizationStatusForAccessLevel:"),
                ACCESS_READ_WRITE,
            )
        };
        if status == STATUS_NOT_DETERMINED {
            // SAFETY: a class method taking an access level and a handler block, which is a
            // global block of the signature `(PHAuthorizationStatus)`.
            unsafe {
                msg::<unsafe extern "C" fn(Id, Sel, isize, Id)>()(
                    library,
                    sel(c"requestAuthorizationForAccessLevel:handler:"),
                    ACCESS_READ_WRITE,
                    AUTH_BLOCK.id(),
                );
            }
        } else {
            proceed(status);
        }
    }

    /// The framework's queue: the player answered the permission question.
    extern "C" fn authorised(_: *const c_void, status: isize) {
        AUTH_STATUS.store(status, Ordering::SeqCst);
        on_main(after_authorisation);
    }

    extern "C" fn after_authorisation(_: *mut c_void) {
        proceed(AUTH_STATUS.load(Ordering::SeqCst));
    }

    /// Main thread: with the permission known, finds the album and asks for the change.
    fn proceed(status: isize) {
        if status != STATUS_AUTHORIZED && status != STATUS_LIMITED {
            note("no read-write access to the photos");
            finish_failed();
            return;
        }
        let Some(path) = STATE.lock().ok().and_then(|s| s.paths.front().cloned()) else {
            return;
        };
        let album = find_album();
        if let Ok(mut job) = JOB.lock() {
            job.path = Some(path);
            job.album = album as usize;
        }
        let library = send0(library_class(), c"sharedPhotoLibrary");
        if library.is_null() {
            note("no photo library");
            finish_failed();
            return;
        }
        // SAFETY: `-performChanges:completionHandler:` takes two blocks, the first with no
        // arguments, the second `(BOOL, NSError *)`: `CHANGES_BLOCK` and `COMPLETION_BLOCK` are
        // global blocks of those signatures.
        unsafe {
            msg::<unsafe extern "C" fn(Id, Sel, Id, Id)>()(
                library,
                sel(c"performChanges:completionHandler:"),
                CHANGES_BLOCK.id(),
                COMPLETION_BLOCK.id(),
            );
        }
    }

    /// The player's album with our name, retained, or null when there is none.
    fn find_album() -> Id {
        let collection = class(c"PHAssetCollection");
        if collection.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: `+fetchAssetCollectionsWithType:subtype:options:` takes two integers and an
        // options object (null for none) and returns a `PHFetchResult`.
        let found = unsafe {
            msg::<unsafe extern "C" fn(Id, Sel, isize, isize, Id) -> Id>()(
                collection,
                sel(c"fetchAssetCollectionsWithType:subtype:options:"),
                TYPE_ALBUM,
                SUBTYPE_ALBUM_REGULAR,
                std::ptr::null_mut(),
            )
        };
        if found.is_null() {
            return std::ptr::null_mut();
        }
        let name = ns_string(ALBUM_NAME);
        // SAFETY: `-count` returns an `NSUInteger`, `-objectAtIndex:` an object for an index
        // below it, `-localizedTitle` a string (or nil), `-isEqualToString:` a BOOL.
        unsafe {
            let count = msg::<unsafe extern "C" fn(Id, Sel) -> usize>()(found, sel(c"count"));
            for index in 0..count {
                let album = msg::<unsafe extern "C" fn(Id, Sel, usize) -> Id>()(
                    found,
                    sel(c"objectAtIndex:"),
                    index,
                );
                let title = send0(album, c"localizedTitle");
                if !title.is_null()
                    && msg::<unsafe extern "C" fn(Id, Sel, Id) -> bool>()(
                        title,
                        sel(c"isEqualToString:"),
                        name,
                    )
                {
                    return send0(album, c"retain");
                }
            }
        }
        std::ptr::null_mut()
    }

    /// The framework's queue: the change itself, in `performChanges`' block: the picture becomes
    /// an asset and joins the album (made here when `find_album` found none).
    extern "C" fn changes(_: *const c_void) {
        let Ok(job) = JOB.lock() else {
            return;
        };
        let Some(path) = job.path.as_ref() else {
            return;
        };
        let url = send1(class(c"NSURL"), c"fileURLWithPath:", ns_string(path));
        let asset_request = send1(
            class(c"PHAssetChangeRequest"),
            c"creationRequestForAssetFromImageAtFileURL:",
            url,
        );
        let album_class = class(c"PHAssetCollectionChangeRequest");
        if url.is_null() || asset_request.is_null() || album_class.is_null() {
            return;
        }
        let album_request = if job.album == 0 {
            send1(
                album_class,
                c"creationRequestForAssetCollectionWithTitle:",
                ns_string(ALBUM_NAME),
            )
        } else {
            send1(
                album_class,
                c"changeRequestForAssetCollection:",
                job.album as Id,
            )
        };
        let placeholder = send0(asset_request, c"placeholderForCreatedAsset");
        if album_request.is_null() || placeholder.is_null() {
            return;
        }
        let list = send1(class(c"NSArray"), c"arrayWithObject:", placeholder);
        send1(album_request, c"addAssets:", list);
    }

    /// The framework's queue: the change is over.
    extern "C" fn completed(_: *const c_void, ok: bool, _error: Id) {
        LAST_OK.store(ok, Ordering::SeqCst);
        on_main(finish);
    }

    fn finish_failed() {
        LAST_OK.store(false, Ordering::SeqCst);
        finish(std::ptr::null_mut());
    }

    /// Main thread: the first picture is done: a failed one goes to the camera roll, and the next
    /// waiting one starts.
    extern "C" fn finish(_: *mut c_void) {
        let album = match JOB.lock() {
            Ok(mut job) => {
                job.path = None;
                std::mem::take(&mut job.album)
            }
            Err(_) => 0,
        };
        if album != 0 {
            send0(album as Id, c"release");
        }
        let (done, more) = match STATE.lock() {
            Ok(mut state) => {
                let done = state.paths.pop_front();
                if state.paths.is_empty() {
                    state.busy = false;
                }
                (done, state.busy)
            }
            Err(_) => return,
        };
        if let Some(path) = done {
            if LAST_OK.load(Ordering::SeqCst) {
                note(&format!(
                    "{} added to the album Rally Championship",
                    path.to_string_lossy()
                ));
            } else {
                camera_roll(path);
            }
        }
        if more {
            begin(std::ptr::null_mut());
        }
    }

    /// Main thread: `UIImageWriteToSavedPhotosAlbum`, which adds the picture to the camera roll
    /// with the add-only permission.
    fn camera_roll(path: CString) {
        // The classes are looked up by name and null checked; the image is checked before use and
        // lives in the main queue's autorelease pool, which the save call retains it past.
        let image = {
            let image_class = class(c"UIImage");
            if image_class.is_null() {
                std::ptr::null_mut()
            } else {
                send1(image_class, c"imageWithContentsOfFile:", ns_string(&path))
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
            );
        }
        note(&format!(
            "{} sent to the camera roll",
            path.to_string_lossy()
        ));
    }
}
