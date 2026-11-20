use super::*;

/// What makes up a pipeline. The comparisons and factors are Direct3D's numbers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PipelineKey {
    /// Direct3D's culling: 1 none, 2 clockwise, 3 counter-clockwise on the screen.
    pub cull: u32,
    /// The depth test's comparison (always, when the test is off: writing goes on regardless,
    /// which a GPU's switched-off test would stop).
    pub depth_function: u32,
    pub depth_write: bool,
    /// The blend's source and target factors.
    pub blend: Option<(u32, u32)>,
    /// False for a depth clear: only the depth is written.
    pub colour: bool,
}

/// Which of a pipeline's states a key sets differently from the key before it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PipelineChange {
    pub cull: bool,
    pub depth_function: bool,
    pub depth_write: bool,
    pub blend: bool,
    pub colour: bool,
}

impl PipelineKey {
    /// The face a GL culling mode drops (`GL_FRONT` 0x0404 or `GL_BACK` 0x0405), `None` when
    /// culling is off.
    pub fn cull_face(&self) -> Option<u32> {
        match self.cull {
            3 => Some(0x0405),
            2 => Some(0x0404),
            _ => None,
        }
    }

    /// The GL depth comparison (`GL_NEVER` .. `GL_ALWAYS`) of the key's number.
    pub fn depth_func(&self) -> u32 {
        match self.depth_function {
            f @ 1..=7 => 0x0200 + f - 1,
            _ => 0x0207, // as the rasteriser's `compare`: anything else passes
        }
    }

    /// The states that must be set after `was` (all of them when nothing was set yet). A
    /// state whose GL value is the same stays out, even when the key's number differs.
    pub fn changes_from(&self, was: Option<&PipelineKey>) -> PipelineChange {
        match was {
            None => PipelineChange {
                cull: true,
                depth_function: true,
                depth_write: true,
                blend: true,
                colour: true,
            },
            Some(was) => PipelineChange {
                cull: self.cull_face() != was.cull_face(),
                depth_function: self.depth_func() != was.depth_func(),
                depth_write: self.depth_write != was.depth_write,
                blend: self.blend != was.blend,
                colour: self.colour != was.colour,
            },
        }
    }
}

/// The pixel shader's values (`scene.frag`'s `Pixel`).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PixelValues {
    pub fog_colour: [f32; 4],
    pub alpha_reference: f32,
    pub alpha_function: i32,
    pub mixing: i32,
    pub flags: i32,
}

impl PixelValues {
    /// The values' dwords as the shader's uniform block holds them.
    #[cfg(not(handheld))]
    pub fn to_words(self) -> [u32; 8] {
        let [red, green, blue, alpha] = self.fog_colour;
        [
            red.to_bits(),
            green.to_bits(),
            blue.to_bits(),
            alpha.to_bits(),
            self.alpha_reference.to_bits(),
            self.alpha_function as u32,
            self.mixing as u32,
            self.flags as u32,
        ]
    }
}

pub const TEXTURED: i32 = 1;
pub const TEXTURE_HAS_ALPHA: i32 = 2;
pub const FOG: i32 = 4;
/// The batch draws a second pass over its first in the same draw (`Batch::second`).
pub const SECOND: i32 = 8;

/// `source` with `defines` (whole `#define` lines) after its `#version` line, which must stay
/// the first.
pub fn with_defines(source: &str, defines: &str) -> String {
    match source.split_once('\n') {
        Some((version, rest)) => format!("{version}\n{defines}{rest}"),
        None => source.to_string(),
    }
}

/// What of one pass's values a pixel shader can be made for, in place of reading them for
/// every pixel: values that draw the same pixels have the same kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PassKind {
    /// The alpha test's comparison (1 ..= 7); 0 when the test discards nothing.
    pub alpha_function: u8,
    /// How the texel and the colour make the pixel: 1 the texel alone, 4 their product, 0
    /// their colours' product with the alpha of one of them. 0 without a texture.
    pub mixing: u8,
    pub textured: bool,
    /// The alpha is the texel's (only where `mixing` is 0).
    pub texel_alpha: bool,
    pub fog: bool,
}

impl PassKind {
    /// The kind of `values`; `discard_free`: the pass's alpha test is known to keep every pixel.
    pub fn of(values: &PixelValues, discard_free: bool) -> PassKind {
        let textured = values.flags & TEXTURED != 0;
        let mixing = match values.mixing {
            _ if !textured => 0,
            1 | 7 => 1,
            4 => 4,
            _ => 0,
        };
        PassKind {
            alpha_function: match values.alpha_function {
                function @ 1..=7 if !discard_free => function as u8,
                _ => 0,
            },
            mixing,
            textured,
            texel_alpha: textured && mixing == 0 && values.flags & TEXTURE_HAS_ALPHA != 0,
            fog: values.flags & FOG != 0,
        }
    }

    /// The kind as the shader's `flags` holds it.
    pub(super) fn flags(&self) -> i32 {
        let bit = |on: bool, flag: i32| if on { flag } else { 0 };
        bit(self.textured, TEXTURED) | bit(self.texel_alpha, TEXTURE_HAS_ALPHA) | bit(self.fog, FOG)
    }
}

/// What of a batch the scene's pixel shader can be made for: its pass's kind and, with a
/// second pass, that one's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ShaderKind {
    pub first: PassKind,
    pub second: Option<PassKind>,
}

impl ShaderKind {
    pub fn of(batch: &Batch) -> ShaderKind {
        ShaderKind {
            first: PassKind::of(&batch.values, batch.discard_free),
            second: batch
                .second
                .map(|second| PassKind::of(&second.values, false)),
        }
    }

    /// The `#define` lines that make the scene's shaders this kind's: the passes' numbers as
    /// constants, and which of the vertex's values a pass reads at all.
    pub fn defines(&self) -> String {
        let mut lines = format!(
            "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION {}\n#define MIXING {}\n#define FLAGS {}\n",
            self.first.alpha_function,
            self.first.mixing,
            self.first.flags() | if self.second.is_some() { SECOND } else { 0 },
        );
        let mut define = |on: bool, name: &str| {
            if on {
                lines += &format!("#define {name}\n");
            }
        };
        define(self.first.alpha_function == 0, "NO_DISCARD");
        define(self.first.textured, "FIRST_TEXTURE");
        define(self.first.fog, "FIRST_FOG");
        if let Some(second) = self.second {
            define(true, "SECOND");
            define(second.textured, "SECOND_TEXTURE");
            define(second.fog, "SECOND_FOG");
            lines += &format!(
                "#define ALPHA_FUNCTION2 {}\n#define MIXING2 {}\n#define FLAGS2 {}\n",
                second.alpha_function,
                second.mixing,
                second.flags(),
            );
        }
        lines
    }

    /// The `#define` lines of the shaders made for every kind: each pass's numbers read for
    /// every pixel, every value of the vertex passed on. `discard_free`: with no alpha test.
    pub fn defines_for_any(discard_free: bool) -> String {
        let mut lines = String::from(
            "#define FIRST_TEXTURE\n#define FIRST_FOG\n#define SECOND\n#define SECOND_TEXTURE\n#define SECOND_FOG\n",
        );
        if discard_free {
            lines += "#define NO_DISCARD\n";
        }
        lines
    }
}
