use super::*;

#[test]
fn mip_levels_average_four_texels_and_are_chosen_by_density() {
    // 4x2, 32-bit: the left half 0xFF000000-black with alpha, the right half white.
    let texels: [u32; 8] = [
        0xFF00_0000,
        0xFF00_0000,
        0xFFFF_FFFF,
        0xFFFF_FFFF,
        0xFF00_0000,
        0xFF20_4060,
        0xFFFF_FFFF,
        0xFFFF_FFFF,
    ];
    let bytes: Vec<u8> = texels.iter().flat_map(|p| p.to_le_bytes()).collect();
    let chain = mip_chain(&Texture {
        pixels: &bytes,
        width: 4,
        height: 2,
        pitch: 16,
        bytes: 4,
        masks: [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000],
        lower: &[],
    });
    assert_eq!(chain.len(), 2);
    assert_eq!((chain[0].width, chain[0].height), (2, 1));
    let first: Vec<u32> = chain[0]
        .pixels
        .chunks(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    // (0 + 0 + 0 + 0x60 + 2) / 4 = 0x18 in blue; the right half stays white.
    assert_eq!(first, [0xFF08_1018, 0xFFFF_FFFF]);
    assert_eq!((chain[1].width, chain[1].height), (1, 1));
    // A pixel as big as a texel reads the first level, two texels across the next.
    let scale = |s: f32, t: f32| mip_level([[s, 0.0], [0.0, t]], 2);
    assert_eq!(scale(1.0, 1.0), 0);
    assert_eq!(scale(2.0, 2.0), 1);
    assert_eq!(scale(4.0, 4.0), 2);
    assert_eq!(scale(40.0, 40.0), 2);
    // Stretched: the narrow side decides.
    assert_eq!(scale(8.0, 1.0), 0);
    assert_eq!(scale(1.0, 2.0), 0);
}

/// A frame of `w` x `h` 32-bit pixels drawn with one fan (blending and the depth test
/// on), in tiles of `size` (0: one), scalar or not: its pixels and depths.
fn draw(size: usize, scalar: bool) -> (Vec<u32>, Vec<f32>) {
    let (w, h) = (100, 70);
    let mut pixels = vec![0x0010_2030u32; w * h];
    let mut depth = vec![1.0f32; w * h];
    let v = |x: f32, y: f32, z: f32, colour: u32| -> Vertex {
        [
            x.to_bits(),
            y.to_bits(),
            z.to_bits(),
            1f32.to_bits(),
            colour,
            0xFF00_0000,
            0,
            0,
        ]
    };
    let fan = [
        v(3.5, 2.0, 0.5, 0x80FF_0000),
        v(95.0, 10.0, 0.2, 0x8000_FF00),
        v(70.0, 66.0, 0.7, 0x8000_00FF),
        v(10.0, 50.0, 0.4, 0xFFFF_FFFF),
    ];
    let mut states = [0u32; 0x40];
    states[0x07] = 1;
    states[0x17] = 4;
    states[0x0E] = 1;
    states[0x1B] = 1;
    states[0x13] = 5;
    states[0x14] = 6;
    states[0x16] = 1;
    SCALAR_ONLY.store(scalar, std::sync::atomic::Ordering::SeqCst);
    {
        let mut frame = Frame::new(&mut pixels, w, w, h, &mut depth);
        for tile in frame.tiles(size).iter_mut() {
            tile.draw_fan(&fan, &States(&states), None);
            tile.clear_depth([0, 0, 5, 5]);
        }
    }
    SCALAR_ONLY.store(false, std::sync::atomic::Ordering::SeqCst);
    (pixels, depth)
}

#[test]
fn tiles_and_lanes_draw_the_same() {
    let whole = draw(0, true);
    assert!(whole.0.iter().any(|&p| p != 0x0010_2030), "something drawn");
    // The four-lane path and the scalar one: the same numbers.
    assert!(whole == draw(0, false));
    assert!(draw(16, false) == draw(16, true));
    // Tiles: the same colours; a row's values start from its first place in the tile,
    // so a depth can differ in its last bit.
    for tiled in [draw(16, false), draw(7, true)] {
        assert!(whole.0 == tiled.0);
        assert!(
            whole
                .1
                .iter()
                .zip(&tiled.1)
                .all(|(a, b)| (a - b).abs() < 1e-6)
        );
    }
}
