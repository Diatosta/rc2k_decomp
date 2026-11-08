#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut seen = 0u64;
    let _ = rc2k_core::drive::rle::unpack_rle(data, |_, b| seen += u64::from(b));
    std::hint::black_box(seen);
});
