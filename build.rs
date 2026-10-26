// The standalone program links SDL3, whose ready-made library `tools/fetch_sdl3.sh` puts
// in `third_party/sdl3`, and FFmpeg's decoder, which `tools/build_ffmpeg.sh` builds into
// `third_party/ffmpeg` (neither in git). Their DLLs are copied next to what is built (the
// program and the test binaries in `deps`): Windows, and Wine running the tests, look for
// a program's DLLs in its own folder, not on the build's search path.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
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
