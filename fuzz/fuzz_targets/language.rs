#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(file) = <&[u8; 0x30]>::try_from(data.get(..0x30).unwrap_or(&[])) {
        let _ = rc2k_core::strings::language::Locale::parse(file);
    }
    let _ = rc2k_core::system::command_line::read(data);
});
