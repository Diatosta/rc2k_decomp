// The standalone program links SDL3, whose ready-made library `tools/fetch_sdl3.sh` puts
// in `third_party/sdl3` (not in git). The hook DLL does not use it.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_STANDALONE").is_some() {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-search=native={dir}/third_party/sdl3/lib/x86");
    }
}
