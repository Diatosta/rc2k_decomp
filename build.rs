// The standalone program links SDL3, whose ready-made library `tools/fetch_sdl3.sh` puts
// in `third_party/sdl3`, and FFmpeg's decoder, which `tools/build_ffmpeg.sh` builds into
// `third_party/ffmpeg` (neither in git).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={dir}/third_party/sdl3/lib/x86");
    println!("cargo:rustc-link-search=native={dir}/third_party/ffmpeg/bin");
}
