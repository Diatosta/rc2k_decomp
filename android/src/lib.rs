//! The entry point the Android activity calls (SDL's `SDL_main`), into the game's own start
//! (`rc2k_decomp::android`).

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn SDL_main(
    _argc: std::ffi::c_int,
    _argv: *mut *mut std::ffi::c_char,
) -> std::ffi::c_int {
    rc2k_decomp::android::main()
}
