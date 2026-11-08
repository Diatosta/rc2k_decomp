#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 4 { return; }
    let (head, rest) = data.split_at(4);
    let mut out = vec![0u8; 1 << 16];
    let _ = rc2k_core::gfx::png::inflate(rest, &mut out);
    let row_bytes = usize::from(head[0]) + 1;
    let rows = u32::from(head[1]);
    let step = usize::from(head[2] % 8) + 1;
    let mut area = rest.to_vec();
    let _ = rc2k_core::gfx::png::unfilter(&mut area, row_bytes, rows, step, &|i| head[3].wrapping_add(i as u8));
});
