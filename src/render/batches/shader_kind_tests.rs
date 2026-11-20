use super::*;

/// The scene's pixel shader for one pass, with `function`, `mixing` and `flags` as it reads
/// them (uniforms, or a kind's constants): the pixel's colour before the fog, whether the
/// fog is mixed in, or `None` for a pixel the alpha test discards.
fn shaded(
    (function, mixing, flags): (i32, i32, i32),
    reference: f32,
    colour: [f32; 4],
    texel: [f32; 4],
) -> Option<([f32; 4], bool)> {
    let mut c = colour;
    if flags & 1 != 0 {
        c = if mixing == 1 || mixing == 7 {
            texel
        } else if mixing == 4 {
            [0, 1, 2, 3].map(|n| texel[n] * colour[n])
        } else {
            let alpha = if flags & 2 != 0 { texel[3] } else { colour[3] };
            [
                texel[0] * colour[0],
                texel[1] * colour[1],
                texel[2] * colour[2],
                alpha,
            ]
        };
    }
    let passes = match function {
        1 => false,
        2 => c[3] < reference,
        3 => c[3] == reference,
        4 => c[3] <= reference,
        5 => c[3] > reference,
        6 => c[3] != reference,
        7 => c[3] >= reference,
        _ => true,
    };
    (function == 0 || passes).then_some((c, flags & 4 != 0))
}

fn values(alpha_function: i32, mixing: i32, flags: i32) -> PixelValues {
    PixelValues {
        fog_colour: [0.0; 4],
        alpha_reference: 0.5,
        alpha_function,
        mixing,
        flags,
    }
}

#[test]
fn a_pass_made_for_its_kind_shades_as_one_that_reads_its_values() {
    let samples = [
        ([1.0, 0.5, 0.25, 1.0], [0.5, 0.5, 0.5, 0.0]),
        ([0.2, 0.4, 0.6, 0.5], [1.0, 0.75, 0.5, 0.5]),
        ([0.9, 0.1, 0.3, 0.25], [0.1, 0.2, 0.3, 1.0]),
        ([0.0, 1.0, 0.0, 0.75], [0.3, 0.3, 0.3, 0.25]),
    ];
    let mut kinds = std::collections::HashSet::new();
    for alpha_function in 0..10 {
        for mixing in 0..9 {
            for flags in 0..16 {
                let v = values(alpha_function, mixing, flags);
                let kind = PassKind::of(&v, false);
                kinds.insert(kind);
                let read = (alpha_function, mixing, flags);
                let made = (
                    i32::from(kind.alpha_function),
                    i32::from(kind.mixing),
                    kind.flags(),
                );
                for (colour, texel) in samples {
                    assert_eq!(
                        shaded(read, v.alpha_reference, colour, texel),
                        shaded(made, v.alpha_reference, colour, texel),
                        "{read:?} as {kind:?}"
                    );
                }
                // What the kind says a pass has is what the shader would use of it.
                assert_eq!(kind.textured, flags & TEXTURED != 0);
                assert_eq!(kind.fog, flags & FOG != 0);
            }
        }
    }
    // Far fewer programs than values: 8 tests, by fog or not, by 4 ways with a texture and one without.
    assert_eq!(kinds.len(), 8 * 2 * 5);
}

#[test]
fn a_test_that_keeps_every_pixel_is_no_test_in_the_kind() {
    let kind = PassKind::of(&values(5, 4, TEXTURED), true);
    assert_eq!(kind.alpha_function, 0);
    assert_eq!(
        PassKind::of(&values(5, 4, TEXTURED), false).alpha_function,
        5
    );
    // Comparisons the shader does not know keep every pixel too.
    assert_eq!(
        PassKind::of(&values(8, 4, TEXTURED), false).alpha_function,
        0
    );
    assert_eq!(
        PassKind::of(&values(0, 4, TEXTURED), false).alpha_function,
        0
    );
}

#[test]
fn a_batchs_kind_is_its_passes_kinds() {
    let mut batch = Batch {
        first_vertex: 0,
        vertices: 3,
        pipeline: PipelineKey {
            cull: 1,
            depth_function: 4,
            depth_write: true,
            blend: None,
            colour: true,
        },
        values: values(5, 4, TEXTURED | FOG),
        texture: Some(0),
        linear: true,
        clamp: false,
        mips: false,
        scissor: None,
        second: None,
        discard_free: true,
    };
    let first = PassKind {
        alpha_function: 0,
        mixing: 4,
        textured: true,
        texel_alpha: false,
        fog: true,
    };
    assert_eq!(
        ShaderKind::of(&batch),
        ShaderKind {
            first,
            second: None
        }
    );
    // A second pass's test is kept whatever the first's is: only the first's is known to
    // keep every pixel.
    batch.second = Some(Second {
        values: values(5, 2, TEXTURED | TEXTURE_HAS_ALPHA),
        texture: Some(1),
        linear: true,
        clamp: false,
        mips: false,
    });
    let second = PassKind {
        alpha_function: 5,
        mixing: 0,
        textured: true,
        texel_alpha: true,
        fog: false,
    };
    assert_eq!(
        ShaderKind::of(&batch),
        ShaderKind {
            first,
            second: Some(second)
        }
    );
}

#[test]
fn a_kinds_defines_name_its_numbers_and_only_the_values_it_reads() {
    let first = PassKind::of(&values(5, 4, TEXTURED | TEXTURE_HAS_ALPHA | FOG), false);
    let alone = ShaderKind {
        first,
        second: None,
    };
    assert_eq!(
        alone.defines(),
        "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION 5\n#define MIXING 4\n#define FLAGS 5\n\
         #define FIRST_TEXTURE\n#define FIRST_FOG\n"
    );
    let with_second = ShaderKind {
        first: PassKind::of(&values(0, 2, 0), false),
        second: Some(PassKind::of(
            &values(7, 2, TEXTURED | TEXTURE_HAS_ALPHA),
            false,
        )),
    };
    assert_eq!(
        with_second.defines(),
        "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION 0\n#define MIXING 0\n#define FLAGS 8\n\
         #define NO_DISCARD\n#define SECOND\n#define SECOND_TEXTURE\n\
         #define ALPHA_FUNCTION2 7\n#define MIXING2 0\n#define FLAGS2 3\n"
    );
    assert_eq!(
        ShaderKind::defines_for_any(true),
        "#define FIRST_TEXTURE\n#define FIRST_FOG\n#define SECOND\n#define SECOND_TEXTURE\n\
         #define SECOND_FOG\n#define NO_DISCARD\n"
    );
}

#[test]
fn defines_go_after_the_version_line() {
    assert_eq!(
        with_defines(
            "#version 300 es\nvoid main() {}\n",
            "#define A\n#define B 1\n"
        ),
        "#version 300 es\n#define A\n#define B 1\nvoid main() {}\n"
    );
    assert_eq!(with_defines("no line end", "#define A\n"), "no line end");
}
