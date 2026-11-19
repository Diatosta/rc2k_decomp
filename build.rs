// The program links SDL3. On Windows its ready-made library comes from `tools/fetch_sdl3.sh`
// (or build.ps1) into `third_party/sdl3` (not in git), and SDL3.dll is copied next to what is
// built (the program and the test binaries in `deps`): Windows, and Wine running the tests,
// look for a program's DLLs in its own folder, not on the build's search path. On Linux and
// macOS SDL is built from its source and linked into the program (`sdl3-sys`'s
// `build-from-source-static`): one program runs with nothing else installed. The films are
// decoded by `rc2k_core::film`, in Rust: no media library is linked. The Windows program carries
// `assets/icon.ico` as its icon (Explorer's, the taskbar's and the window's).
fn main() {
    // The debug UI: 64-bit builds with a window to draw it over (not 32-bit Windows, whose
    // Dear ImGui crate has no target; not the handheld, whose frames go to the framebuffer).
    println!("cargo:rustc-check-cfg=cfg(debug_ui)");
    // A build instrumented for profile-guided optimisation (`tools/anbernic/pgo.sh` passes it
    // with `-C profile-generate`): the counts are written as the program ends.
    println!("cargo:rustc-check-cfg=cfg(rc2k_pgo)");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let bits = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap_or_default();
    let handheld = os == "linux" && arch == "aarch64";
    if bits == "64" && !handheld && os != "android" && os != "ios" {
        println!("cargo:rustc-cfg=debug_ui");
    }
    println!("cargo:rerun-if-changed=build.rs");
    // A touch screen as the player's pointer: the menus take taps and drags (`platform::touch_menu`).
    // The phones and tablets; `RC2K_TOUCH_BUILD=1` builds it for any other system (a touch
    // laptop, trying it out).
    println!("cargo:rustc-check-cfg=cfg(touch_screen)");
    println!("cargo:rerun-if-env-changed=RC2K_TOUCH_BUILD");
    if matches!(os.as_str(), "android" | "ios") || std::env::var_os("RC2K_TOUCH_BUILD").is_some() {
        println!("cargo:rustc-cfg=touch_screen");
    }
    // Network play through Steam: the desktop systems (not the handheld, which has no Steam, nor
    // the web). Valve's library is not linked; the program loads it while it runs when it is
    // there.
    println!("cargo:rustc-check-cfg=cfg(steam_support)");
    if !handheld && matches!(os.as_str(), "windows" | "linux" | "macos") {
        println!("cargo:rustc-cfg=steam_support");
    }
    if os == "android" {
        // SDL's shared library, built by `tools/android/build.sh` (its Java glue must match it);
        // the system log's library for `platform::ffi::android`.
        println!("cargo:rerun-if-env-changed=RC2K_ANDROID_SDL_LIB");
        if let Ok(dir) = std::env::var("RC2K_ANDROID_SDL_LIB") {
            println!("cargo:rustc-link-search=native={dir}");
        }
        println!("cargo:rustc-link-lib=dylib=SDL3");
        println!("cargo:rustc-link-lib=dylib=log");
        return;
    }
    if os == "macos" {
        // SDL's haptics for controllers: on every Mac since 10.15 (SDL looks for it before use),
        // missing from Darling, where the tests run; weak, so the program starts without it.
        println!("cargo:rustc-link-arg=-Wl,-weak_framework,CoreHaptics");
        return;
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // 32 or 64 bits: SDL's folder for it.
    let sdl = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "third_party/sdl3/lib/x64",
        _ => "third_party/sdl3/lib/x86",
    };
    println!("cargo:rustc-link-search=native={dir}/{sdl}");
    icon_resource(&dir);
    // OUT_DIR is <profile>/build/<crate>-<hash>/out (cargo 1.99) or
    // <profile>/build/<crate>/<hash>/out (cargo 1.100): the profile folder is the parent of
    // the `build` folder.
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let Some(profile) = out
        .ancestors()
        .find(|a| a.file_name().is_some_and(|n| n == "build"))
        .and_then(std::path::Path::parent)
    else {
        return;
    };
    for dll in [format!("{sdl}/SDL3.dll")] {
        let from = std::path::Path::new(&dir).join(dll);
        println!("cargo:rerun-if-changed={}", from.display());
        let Some(name) = from.file_name() else {
            continue;
        };
        for to in [profile.to_path_buf(), profile.join("deps")] {
            // A missing DLL (a fresh checkout) is no build error: the link says what is missing.
            let _ = std::fs::create_dir_all(&to);
            let _ = std::fs::copy(&from, to.join(name));
        }
    }
}

/// `assets/icon.ico` as the program's icon: a resource file written here (each image of the
/// `.ico` as an icon resource, and the group that lists them) and handed to the linker, which
/// takes `.res` files as they are (MSVC's link and LLVM's lld-link alike): no resource compiler
/// is needed on any host.
fn icon_resource(dir: &str) {
    const RT_ICON: u16 = 3;
    const RT_GROUP_ICON: u16 = 14;
    let path = std::path::Path::new(dir).join("assets/icon.ico");
    println!("cargo:rerun-if-changed={}", path.display());
    let ico = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let word = |at: usize| u16::from_le_bytes([ico[at], ico[at + 1]]);
    let dword = |at: usize| u32::from_le_bytes([ico[at], ico[at + 1], ico[at + 2], ico[at + 3]]);
    assert!(
        ico.len() >= 6 && word(0) == 0 && word(2) == 1,
        "{} is not an icon file",
        path.display()
    );
    let count = usize::from(word(4));

    // One resource: its header (numeric type and name, so 32 bytes) and its data, padded to
    // four bytes.
    fn entry(res: &mut Vec<u8>, kind: u16, name: u16, flags: u16, data: &[u8]) {
        res.extend_from_slice(&(data.len() as u32).to_le_bytes());
        res.extend_from_slice(&32u32.to_le_bytes());
        for v in [0xFFFF, kind, 0xFFFF, name] {
            res.extend_from_slice(&u16::to_le_bytes(v));
        }
        res.extend_from_slice(&0u32.to_le_bytes()); // data version
        res.extend_from_slice(&flags.to_le_bytes());
        res.extend_from_slice(&0x0409u16.to_le_bytes()); // US English, as resource compilers give
        res.extend_from_slice(&0u32.to_le_bytes()); // version
        res.extend_from_slice(&0u32.to_le_bytes()); // characteristics
        res.extend_from_slice(data);
        res.resize(res.len().next_multiple_of(4), 0);
    }

    // The file starts with an empty resource that says it is a 32-bit resource file.
    let mut res = vec![0u8; 32];
    res[4] = 32;
    res[8..12].copy_from_slice(&[0xFF, 0xFF, 0, 0]);
    res[12..16].copy_from_slice(&[0xFF, 0xFF, 0, 0]);
    let mut group = Vec::new();
    for v in [0u16, 1, count as u16] {
        group.extend_from_slice(&v.to_le_bytes());
    }
    for i in 0..count {
        let at = 6 + 16 * i;
        let (size, offset) = (dword(at + 8) as usize, dword(at + 12) as usize);
        let id = (i + 1) as u16;
        entry(&mut res, RT_ICON, id, 0x1010, &ico[offset..offset + size]);
        // The directory entry of the file, with the resource's number in place of the offset.
        group.extend_from_slice(&ico[at..at + 12]);
        group.extend_from_slice(&id.to_le_bytes());
    }
    entry(&mut res, RT_GROUP_ICON, 1, 0x1030, &group);
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icon.res");
    std::fs::write(&out, res).unwrap();
    println!("cargo:rustc-link-arg-bin=rc2k={}", out.display());
}
