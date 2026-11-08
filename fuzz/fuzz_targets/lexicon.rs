#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = rc2k_core::strings::lexicon::compile(data, 0x40_0000);
});
