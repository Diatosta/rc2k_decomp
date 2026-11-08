#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut buffer = data.to_vec();
    let size = data.len();
    buffer.resize(size * 2 + 64, 0);
    let _ = rc2k_core::files::packed::unpack(&mut buffer, size);
});
