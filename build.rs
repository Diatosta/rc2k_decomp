// The standalone program links SDL3, whose ready-made library `tools/fetch_sdl3.sh` puts
// in `third_party/sdl3`, and FFmpeg's decoder, which `tools/build_ffmpeg.sh` builds into
// `third_party/ffmpeg` (neither in git). Their DLLs are copied next to what is built (the
// program and the test binaries in `deps`): Windows, and Wine running the tests, look for
// a program's DLLs in its own folder, not on the build's search path.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={dir}/third_party/sdl3/lib/x86");
    println!("cargo:rustc-link-search=native={dir}/third_party/ffmpeg/bin");
    // OUT_DIR is <profile>/build/<crate>-<hash>/out.
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let Some(profile) = out.ancestors().nth(3) else {
        return;
    };
    for dll in [
        "third_party/sdl3/lib/x86/SDL3.dll",
        "third_party/ffmpeg/bin/avcodec-61.dll",
        "third_party/ffmpeg/bin/avutil-59.dll",
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
