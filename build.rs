// The program links SDL3. On Windows its ready-made library comes from `tools/fetch_sdl3.sh`
// (or build.ps1) into `third_party/sdl3` (not in git), and SDL3.dll is copied next to what is
// built (the program and the test binaries in `deps`): Windows, and Wine running the tests,
// look for a program's DLLs in its own folder, not on the build's search path. On Linux and
// macOS SDL is built from its source and linked into the program (`sdl3-sys`'s
// `build-from-source-static`): one program runs with nothing else installed. The films are
// decoded by `rc2k_core::film`, in Rust: no media library is linked.
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
    if bits == "64" && !handheld {
        println!("cargo:rustc-cfg=debug_ui");
    }
    println!("cargo:rerun-if-changed=build.rs");
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
