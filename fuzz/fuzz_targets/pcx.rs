#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 8 { return; }
    let (head, file) = data.split_at(8);
    // Both sides at most 0xFFF: a picture of 16 million pixels is the longest a run takes.
    let layout = rc2k_core::gfx::pcx::Layout {
        width: u32::from(u16::from_le_bytes([head[0], head[1]])) & 0xFFF,
        height: u32::from(u16::from_le_bytes([head[2], head[3]])) & 0xFFF,
        planes: u32::from(head[4] % 5),
        ..Default::default()
    };
    let palette = [0u32; 256];
    let mut at = 0usize;
    let next = || -> Result<u8, ()> {
        let Some(&b) = file.get(at) else { return Err(()) };
        at += 1;
        Ok(b)
    };
    let mut sum = 0u64;
    let _ = rc2k_core::gfx::pcx::decode_8bit(&layout, &palette, next, |_, p| sum += u64::from(p));
    std::hint::black_box(sum);
});
