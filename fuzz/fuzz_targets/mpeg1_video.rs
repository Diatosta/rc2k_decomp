#![no_main]

// An MPEG-1 video stream cut into packets and decoded; pictures above 512 x 512 are left out
// (they only make the runs slow).
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    use rc2k_core::film::mpeg1_video::{VideoDecoder, cut};
    let mut decoder = VideoDecoder::new();
    let mut sum = 0u64;
    for packet in cut(data)
        .into_iter()
        .map(|p| &data[p])
        .chain(std::iter::once(&[][..]))
    {
        if packet
            .windows(7)
            .any(|w| w[..4] == [0, 0, 1, 0xB3] && (w[4] > 0x20 || w[5] & 0xF > 2))
        {
            return;
        }
        if let Ok(Some(picture)) = decoder.decode(packet) {
            sum += picture
                .packed_plane(0)
                .iter()
                .map(|&b| u64::from(b))
                .sum::<u64>();
        }
    }
    std::hint::black_box(sum);
});
