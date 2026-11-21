use super::*;

use crate::render::{TestCommand, list};

fn fan(corners: usize, states: [u32; STATES]) -> TestCommand {
    TestCommand::Fan {
        vertices: vec![[0; 8]; corners],
        states: std::sync::Arc::new(states),
        texture: None,
        bounds: (0.0, 0.0, 0.0, 0.0),
    }
}

#[test]
fn fans_become_triangles_and_alike_runs_join() {
    let plain = [0; STATES];
    let mut blended = plain;
    blended[0x1B] = 1;
    let b = Batches::of(
        &list([fan(4, plain), fan(3, plain), fan(3, blended)]),
        640,
        480,
        None,
    );
    assert_eq!(b.vertices.len(), 12);
    assert_eq!(b.batches.len(), 2);
    assert_eq!(b.batches[0].vertices, 9);
    assert_eq!(b.batches[1].first_vertex, 9);
}

#[test]
fn a_switched_off_depth_test_still_writes() {
    let mut states = [0; STATES];
    states[0x0E] = 1;
    states[0x17] = 2;
    let p = pipeline_of(&states);
    assert_eq!((p.depth_function, p.depth_write), (ALWAYS, true));
    states[0x07] = 1;
    assert_eq!(pipeline_of(&states).depth_function, 2);
}

#[test]
fn a_clear_is_cut_to_the_target_and_only_touches_depth() {
    let b = Batches::of(
        &list([TestCommand::Clear([10, 20, 9999, 30])]),
        640,
        480,
        None,
    );
    assert_eq!(b.batches[0].scissor, Some([10, 20, 640, 30]));
    assert!(!b.batches[0].pipeline.colour);
    assert!(
        Batches::of(&list([TestCommand::Clear([5, 5, 5, 9])]), 640, 480, None)
            .batches
            .is_empty()
    );
}

/// A triangle over x `left ..= right`, y 0 ..= 10, marked by its first vertex's colour.
fn triangle(left: f32, right: f32, mark: u32, states: [u32; STATES]) -> TestCommand {
    let corner = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0, mark, 0, 0, 0];
    TestCommand::Fan {
        vertices: vec![corner(left, 0.0), corner(right, 0.0), corner(left, 10.0)],
        states: std::sync::Arc::new(states),
        texture: None,
        bounds: (0.0, 0.0, 0.0, 0.0),
    }
}

fn marks(b: &Batches) -> Vec<Vec<u32>> {
    let batch = |x: &Batch| {
        let first = x.first_vertex as usize;
        b.vertices[first..first + x.vertices as usize]
            .iter()
            .step_by(3)
            .map(|v| v[4])
            .collect()
    };
    b.batches.iter().map(batch).collect()
}

#[test]
fn alike_batches_gather_past_others_that_touch_none_of_their_pixels() {
    let plain = [0; STATES];
    let mut blended = plain;
    blended[0x1B] = 1;
    let b = Batches::of(
        &list([
            triangle(0.0, 10.0, 1, plain),
            triangle(20.0, 30.0, 2, blended),
            triangle(40.0, 50.0, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
    assert_eq!(b.vertices.len(), 9);
}

#[test]
fn triangles_sharing_only_places_between_whole_ones_keep_their_order() {
    let plain = [0; STATES];
    let mut blended = plain;
    blended[0x1B] = 1;
    // The blended triangle and the last one both cover x from 10.3 to 10.6, where no whole
    // place lies: a target finer than the game's grid has pixel centres there.
    let b = Batches::of(
        &list([
            triangle(0.0, 10.0, 1, plain),
            triangle(10.3, 30.0, 2, blended),
            triangle(9.0, 10.6, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
}

#[test]
fn a_batch_between_that_touches_the_same_pixels_keeps_the_order() {
    let plain = [0; STATES];
    let mut blended = plain;
    blended[0x1B] = 1;
    // Pixel column 20 is in all three: the order there is the picture.
    let b = Batches::of(
        &list([
            triangle(0.0, 20.0, 1, plain),
            triangle(20.0, 30.0, 2, blended),
            triangle(20.0, 50.0, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
    // A column apart, with a margin under a pixel, is apart.
    let b = Batches::of(
        &list([
            triangle(0.0, 19.0, 1, plain),
            triangle(20.0, 30.0, 2, blended),
            triangle(31.0, 50.0, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
}

#[test]
fn nothing_gathers_past_a_clear() {
    let plain = [0; STATES];
    let b = Batches::of(
        &list([
            triangle(0.0, 10.0, 1, plain),
            TestCommand::Clear([100, 100, 200, 200]),
            triangle(40.0, 50.0, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(b.batches.len(), 3);
    assert_eq!(b.batches[1].scissor, Some([100, 100, 200, 200]));
}

#[test]
fn a_place_that_is_no_number_touches_everything() {
    let plain = [0; STATES];
    let mut blended = plain;
    blended[0x1B] = 1;
    let b = Batches::of(
        &list([
            triangle(0.0, 10.0, 1, plain),
            triangle(f32::NAN, 30.0, 2, blended),
            triangle(40.0, 50.0, 3, plain),
        ]),
        640,
        480,
        None,
    );
    assert_eq!(b.batches.len(), 3);
}

/// A textured fan of `corners` places (the same each time), its colour `colour`, its
/// texture at `texture` (an identity only).
fn textured(corners: usize, colour: u32, texture: u32, states: [u32; STATES]) -> TestCommand {
    let corner = |n: usize| {
        let (x, y) = (n as f32 * 10.0, (n % 2) as f32 * 10.0);
        [x.to_bits(), y.to_bits(), 0, 0, colour, 0x7F00_0000, 3, 4]
    };
    TestCommand::Fan {
        vertices: (0..corners).map(corner).collect(),
        states: std::sync::Arc::new(states),
        texture: Some(TextureNote {
            pixels: crate::render::TextureId {
                surface: texture,
                serial: 0,
            },
            len: 4,
            width: 1,
            height: 1,
            pitch: 4,
            bytes: 4,
            masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
        }),
        bounds: (0.0, 0.0, 0.0, 0.0),
    }
}

/// The ground's passes: the first writes depth, the second does not and blends.
fn passes() -> ([u32; STATES], [u32; STATES]) {
    let mut first = [0; STATES];
    first[0x07] = 1;
    first[0x17] = 4;
    first[0x0E] = 1;
    first[0x1B] = 1;
    first[0x13] = 5;
    first[0x14] = 6;
    let mut second = first;
    second[0x0E] = 0;
    (first, second)
}

const OPAQUE: u32 = 0x1000;
const SEE_THROUGH: u32 = 0x2000;

fn joined(commands: impl IntoIterator<Item = TestCommand>) -> Batches {
    Batches::of(
        &list(commands),
        640,
        480,
        Some(&|t: &TextureNote| t.pixels.surface == OPAQUE),
    )
}

#[test]
fn a_second_pass_over_the_same_places_joins_the_first() {
    let (first, second) = passes();
    let b = joined([
        textured(4, 0xFF00_0000, OPAQUE, first),
        textured(4, 0x8000_0000, SEE_THROUGH, second),
    ]);
    assert_eq!(b.batches.len(), 1);
    let batch = &b.batches[0];
    assert_eq!(batch.vertices, 6);
    assert_eq!(batch.pipeline.blend, None);
    assert!(batch.pipeline.depth_write);
    assert_ne!(batch.values.flags & SECOND, 0);
    let s = batch.second.expect("a second pass");
    assert_eq!(s.texture, Some(1));
    assert_eq!(b.seconds.len(), 6);
    assert!(
        b.seconds
            .iter()
            .all(|v| *v == [0x8000_0000, 0x7F00_0000, 3, 4])
    );
}

#[test]
fn a_second_pass_joins_only_over_an_opaque_full_alpha_first() {
    let (first, second) = passes();
    // A texture with see-through pixels, corners not fully opaque, other places: apart.
    for commands in [
        [
            textured(4, 0xFF00_0000, SEE_THROUGH, first),
            textured(4, 0x8000_0000, OPAQUE, second),
        ],
        [
            textured(4, 0xFE00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, OPAQUE, second),
        ],
        [
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(5, 0x8000_0000, OPAQUE, second),
        ],
        [
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, OPAQUE, first),
        ],
    ] {
        let b = joined(commands);
        assert!(b.batches.iter().all(|b| b.second.is_none()));
        assert_eq!(b.seconds.len(), b.vertices.len());
    }
}

/// `textured`'s, with a texture of `size` x `size`.
fn sized(texture: u32, size: usize, x: f32, states: [u32; STATES]) -> TestCommand {
    let mut command = textured(3, 0xFF00_0000, texture, states);
    if let TestCommand::Fan {
        vertices,
        texture: Some(note),
        ..
    } = &mut command
    {
        (note.width, note.height) = (size, size);
        for v in vertices.iter_mut() {
            v[0] = (f32::from_bits(v[0]) + x).to_bits();
        }
    }
    command
}

#[test]
fn layered_batches_join_across_textures_of_the_same_arrays() {
    let plain = [0; STATES];
    let same = |a: &TextureNote, b: &TextureNote| {
        let main = |t: &TextureNote| matches!(t.width, 128 | 256);
        a.width == b.width || (main(a) && main(b))
    };
    let b = Batches::of_layered(
        &list([
            sized(0x10, 128, 0.0, plain),
            sized(0x20, 256, 0.0, plain),
            sized(0x30, 64, 0.0, plain),
            sized(0x40, 128, 100.0, plain),
        ]),
        640,
        480,
        &|_| false,
        same,
        false,
    );
    // The 64 x 64 one apart (it overlaps the first two: the last one, apart from it,
    // gathers back into theirs).
    assert_eq!(b.batches.len(), 2);
    assert_eq!(b.batches[0].vertices, 9);
    let numbers: Vec<u16> = b.vertex_textures.iter().map(|v| v[0]).collect();
    assert_eq!(numbers, [0, 0, 0, 1, 1, 1, 3, 3, 3, 2, 2, 2]);
    assert!(b.vertex_textures.iter().all(|v| v[1] == NO_TEXTURE));
    // Unlayered, one texture a draw.
    let commands = list([sized(0x10, 128, 0.0, plain), sized(0x20, 256, 0.0, plain)]);
    assert_eq!(Batches::of(&commands, 640, 480, None).batches.len(), 2);
    assert!(
        Batches::of(&commands, 640, 480, None)
            .vertex_textures
            .is_empty()
    );
}

#[test]
fn the_first_pass_leaves_the_batch_it_had_joined() {
    let (first, second) = passes();
    let b = joined([
        textured(3, 0xFF00_0000, OPAQUE, first),
        textured(4, 0xFF00_0000, OPAQUE, first),
        textured(4, 0x8000_0000, SEE_THROUGH, second),
    ]);
    // The first fan alone, then the second fan with its pass: their places meet, so in
    // that order.
    assert_eq!(b.batches.len(), 2);
    assert_eq!((b.batches[0].vertices, b.batches[0].second), (3, None));
    assert_eq!(b.batches[1].vertices, 6);
    assert!(b.batches[1].second.is_some());
    assert_eq!(b.seconds[..3], [[0; 4]; 3]);
}

/// A vertex at (`x`, `y`) with 1/w `w`.
fn at(x: f32, y: f32, w: f32) -> Vertex {
    [
        x.to_bits(),
        y.to_bits(),
        0,
        w.to_bits(),
        0xFF00_0000,
        0,
        0,
        0,
    ]
}

const TARGET: (f64, f64) = (640.0, 480.0);

#[test]
fn triangles_on_either_side_of_a_shared_edge_are_apart() {
    let (p, q) = (at(10.0, 10.0, 1.0), at(50.0, 30.0, 1.0));
    let left = [p, q, at(12.0, 40.0, 1.0)];
    let right = [q, p, at(45.0, 5.0, 1.0)];
    assert!(apart(&left, &right, TARGET) && apart(&right, &left, TARGET));
    // The other's third corner on the same side: they overlap.
    assert!(!apart(&left, &[q, p, at(20.0, 35.0, 1.0)], TARGET));
    // The edge's corners the same in x and y but not in w: the GPU may place them apart.
    let right_w = [at(50.0, 30.0, 0.5), at(10.0, 10.0, 1.0), at(45.0, 5.0, 1.0)];
    assert!(!apart(&left, &right_w, TARGET));
    // The shared edge past the target: clipping may make new corners on it.
    assert!(!apart(&left, &right, (40.0, 480.0)));
    // A third corner within the margin of the shared edge's line.
    assert!(!apart(&left, &[q, p, at(30.0, 19.95, 1.0)], TARGET));
}

#[test]
fn triangles_apart_by_more_than_the_margin_are_apart() {
    let a = [at(0.0, 0.0, 1.0), at(10.0, 0.0, 1.0), at(0.0, 10.0, 1.0)];
    let gap = |d: f32| {
        [
            at(10.0 + d, 0.0, 1.0),
            at(10.0 + d, 10.0, 1.0),
            at(d, 10.0, 1.0),
        ]
    };
    assert!(apart(&a, &gap(1.0), TARGET));
    // A corner on the diagonal (no corners the same), or nearly: they may meet.
    let touching = [at(5.0, 5.0, 1.0), at(15.0, 5.0, 1.0), at(5.0, 15.0, 1.0)];
    assert!(!apart(&a, &touching, TARGET));
    assert!(!apart(&a, &gap(0.1), TARGET));
    // The diagonal's corners the same: an edge shared.
    assert!(apart(&a, &gap(0.0), TARGET));
    // No plain place, or no area: they may meet.
    let far = [
        at(f32::NAN, 0.0, 1.0),
        at(100.0, 0.0, 1.0),
        at(100.0, 10.0, 1.0),
    ];
    assert!(!apart(&a, &far, TARGET));
    let tiny_w = [
        at(100.0, 0.0, 1e-31),
        at(110.0, 0.0, 1.0),
        at(100.0, 10.0, 1.0),
    ];
    assert!(!apart(&a, &tiny_w, TARGET));
    let flat = [
        at(100.0, 0.0, 1.0),
        at(110.0, 0.0, 1.0),
        at(120.0, 0.0, 1.0),
    ];
    assert!(!apart(&flat, &flat, TARGET));
}

/// A face's two passes, as a car's: its texture with depth written, then its shine added
/// without (blended source alpha, one), each pass a batch of its own kind.
#[test]
fn layered_faces_drawn_twice_gather_past_neighbours_sharing_an_edge() {
    let mut first = [0; STATES];
    (first[0x07], first[0x17], first[0x0E]) = (1, 4, 1);
    let mut shine = first;
    (shine[0x0E], shine[0x1B], shine[0x13], shine[0x14]) = (0, 1, 5, 2);
    let face = |corners: [Vertex; 3], states: [u32; STATES]| TestCommand::Fan {
        vertices: corners.to_vec(),
        states: std::sync::Arc::new(states),
        texture: None,
        bounds: (0.0, 0.0, 0.0, 0.0),
    };
    let (p, q) = (at(10.0, 10.0, 1.0), at(50.0, 30.0, 1.0));
    let one = [p, q, at(12.0, 40.0, 1.0)];
    let two = [q, p, at(45.0, 5.0, 1.0)];
    let commands = list([
        face(one, first),
        face(one, shine),
        face(two, first),
        face(two, shine),
    ]);
    let same = |_: &TextureNote, _: &TextureNote| true;
    let layered = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
    assert_eq!(layered.batches.len(), 2);
    assert_eq!(layered.batches[0].pipeline.blend, None);
    assert_eq!(layered.batches[1].pipeline.blend, Some((5, 2)));
    // In each pass's batch, the faces in their order.
    assert_eq!(layered.vertices[..3], one);
    assert_eq!(layered.vertices[3..6], two);
    // Unlayered (the PC's GPU), the areas alone are looked at: they meet.
    assert_eq!(Batches::of(&commands, 640, 480, None).batches.len(), 4);
    // A second face over the first's places is not apart from it: the order stays.
    let over = [p, q, at(20.0, 35.0, 1.0)];
    let commands = list([
        face(one, first),
        face(one, shine),
        face(over, first),
        face(over, shine),
    ]);
    let layered = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
    assert_eq!(layered.batches.len(), 4);
}

/// States with blending of `source` / `target` on, an alpha test `function` against `reference`
/// (a byte; the test on when `function` is not 0).
fn fast_states(blend: Option<(u32, u32)>, function: u32, reference: u32) -> [u32; STATES] {
    let mut states = [0; STATES];
    if let Some((source, target)) = blend {
        states[0x1B] = 1;
        states[0x13] = source;
        states[0x14] = target;
    }
    if function != 0 {
        states[0x0F] = 1;
        states[0x19] = function;
        states[0x18] = reference;
    }
    states
}

fn corners(alpha: u32, count: usize) -> Vec<Vertex> {
    vec![[0, 0, 0, 0, alpha << 24 | 0x00FF_FFFF, 0, 0, 0]; count]
}

#[test]
fn the_blend_is_off_where_it_keeps_the_source() {
    let full = corners(0xFF, 3);
    let half = corners(0x80, 3);
    let fast = |blend, vertices: &[Vertex], clear| {
        fan_fast_path(&fast_states(blend, 0, 0), vertices, clear).blend_off
    };
    // Source alpha / inverse source alpha: only where the alpha is 1 everywhere.
    assert!(fast(Some((5, 6)), &full, true));
    assert!(!fast(Some((5, 6)), &half, true));
    assert!(!fast(Some((5, 6)), &full, false));
    // One / zero: whatever the alpha.
    assert!(fast(Some((2, 1)), &half, false));
    // Other factors and no blend: not touched.
    assert!(!fast(Some((5, 2)), &full, true));
    assert!(!fast(Some((2, 2)), &full, true));
    assert!(!fast(None, &full, true));
    // One corner under a full alpha spoils the fan.
    let mut mixed = corners(0xFF, 4);
    mixed[2][4] = 0xFE00_0000;
    assert!(!fast(Some((5, 6)), &mixed, true));
}

#[test]
fn the_alpha_test_is_left_out_where_it_cannot_fail() {
    let full = corners(0xFF, 3);
    let half = corners(0x80, 3);
    let free = |function, reference, vertices: &[Vertex], clear| {
        fan_fast_path(&fast_states(None, function, reference), vertices, clear).discard_free
    };
    // Off: for any alpha.
    assert!(free(0, 0, &half, false));
    // Greater, not equal, greater or equal against a reference under 255: where alpha is 1.
    for function in [5, 6, 7] {
        assert!(free(function, 0, &full, true));
        assert!(free(function, 0x80, &full, true));
        assert!(free(function, 254, &full, true));
        assert!(!free(function, 255, &full, true));
        assert!(!free(function, 0x80, &half, true));
        assert!(!free(function, 0x80, &full, false));
    }
    // Never, less, equal, less or equal: can fail at alpha 1.
    for function in [1, 2, 3, 4] {
        assert!(!free(function, 0, &full, true));
    }
    // Always, and a number the shader does not know: pass.
    assert!(free(8, 0, &half, false));
    assert!(free(9, 255, &half, false));
}

#[test]
fn fans_of_the_fast_path_say_so_in_their_batches_and_merge_by_it() {
    let states = fast_states(Some((5, 6)), 7, 0x10);
    let corners = |colour: u32, x: f32| TestCommand::Fan {
        vertices: (0..3)
            .map(|n| {
                let (px, py) = (x + n as f32 * 4.0, (n % 2) as f32 * 4.0);
                [px.to_bits(), py.to_bits(), 0, 0, colour, 0, 0, 0]
            })
            .collect(),
        states: std::sync::Arc::new(states),
        texture: None,
        bounds: (0.0, 0.0, 0.0, 0.0),
    };
    let same = |_: &TextureNote, _: &TextureNote| true;
    let commands = list([
        corners(0xFF00_0000, 0.0),
        corners(0xFF00_0000, 100.0),
        corners(0x8000_0000, 200.0),
        corners(0xFF00_0000, 300.0),
    ]);
    let with = Batches::of_layered(&commands, 640, 480, &|_| false, same, true);
    // The full alpha fans leave out the alpha test and the blend and are one batch past the
    // fan between (apart from them); the half alpha one keeps both.
    let kinds: Vec<_> = with
        .batches
        .iter()
        .map(|b| (b.discard_free, b.pipeline.blend, b.vertices))
        .collect();
    assert_eq!(kinds, [(true, None, 9), (false, Some((5, 6)), 3)]);
    // Off, nothing differs from the game's states.
    let without = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
    assert!(without.batches.iter().all(|b| !b.discard_free));
    assert!(
        without
            .batches
            .iter()
            .all(|b| b.pipeline.blend == Some((5, 6)))
    );
    // The same triangles in the same places either way.
    let places = |b: &Batches| {
        let mut v: Vec<_> = b.vertices.iter().map(|v| v[0]).collect();
        v.sort_unstable();
        v
    };
    assert_eq!(places(&with), places(&without));
}
