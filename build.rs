// The standalone program links SDL3, whose ready-made library `tools/fetch_sdl3.sh` puts
// in `third_party/sdl3`, and FFmpeg's decoder, which `tools/build_ffmpeg.sh` builds into
// `third_party/ffmpeg` (neither in git). Their DLLs are copied next to what is built (the
// program and the test binaries in `deps`): Windows, and Wine running the tests, look for
// a program's DLLs in its own folder, not on the build's search path.
// On Linux both are linked into the program: SDL built from its source (`sdl3-sys`'s
// `build-from-source-static`), FFmpeg's decoders from `tools/build_ffmpeg.sh linux`
// (`third_party/ffmpeg-linux/lib`, static): one program runs on any distribution with
// nothing else installed. macOS the same way: SDL from its source, FFmpeg's decoders from
// `tools/build_ffmpeg.sh mac` (`third_party/ffmpeg-mac-x64` or `ffmpeg-mac-a64`).
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
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let folder = match arch.as_str() {
            "aarch64" => "ffmpeg-mac-a64",
            _ => "ffmpeg-mac-x64",
        };
        let lib = format!("{dir}/third_party/{folder}/lib");
        if !std::path::Path::new(&format!("{lib}/libavcodec.a")).exists() {
            panic!("no static FFmpeg in {lib}: run tools/build_ffmpeg.sh mac");
        }
        println!("cargo:rerun-if-changed={lib}");
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-lib=static=avcodec");
        println!("cargo:rustc-link-lib=static=avutil");
        // SDL's haptics for controllers: on every Mac since 10.15 (SDL looks for it before use),
        // missing from Darling, where the tests run; weak, so the program starts without it.
        println!("cargo:rustc-link-arg=-Wl,-weak_framework,CoreHaptics");
        return;
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let folder = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
            Ok("aarch64") => "ffmpeg-a64",
            _ => "ffmpeg-linux",
        };
        let lib = format!("{dir}/third_party/{folder}/lib");
        if !std::path::Path::new(&format!("{lib}/libavcodec.a")).exists() {
            panic!("no static FFmpeg in {lib}: run tools/build_ffmpeg.sh (linux or a64)");
        }
        println!("cargo:rerun-if-changed={lib}");
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-lib=static=avcodec");
        println!("cargo:rustc-link-lib=static=avutil");
        println!("cargo:rustc-link-lib=dylib=m");
        return;
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // 32 or 64 bits: SDL's folder for it, and FFmpeg's (`tools/build_ffmpeg.sh x64`).
    let (sdl, ffmpeg) = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => ("third_party/sdl3/lib/x64", "third_party/ffmpeg-x64/bin"),
        _ => ("third_party/sdl3/lib/x86", "third_party/ffmpeg/bin"),
    };
    println!("cargo:rustc-link-search=native={dir}/{sdl}");
    println!("cargo:rustc-link-search=native={dir}/{ffmpeg}");
    // OUT_DIR is <profile>/build/<crate>-<hash>/out.
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let Some(profile) = out.ancestors().nth(3) else {
        return;
    };
    for dll in [
        format!("{sdl}/SDL3.dll"),
        format!("{ffmpeg}/avcodec-61.dll"),
        format!("{ffmpeg}/avutil-59.dll"),
    ] {
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
