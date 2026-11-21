//! A diagnostic for what the GPU spends its time on (`RC2K_GPU_CUT=fog,alpha,...`): each name
//! removes one kind of GPU work from the GPU backends (the handheld's GLES and the PC's SDL
//! GPU), so that a run shows the GPU's time without it. The pictures are wrong under a cut;
//! that is expected. Off (nothing removed, the shaders as without this module) unless the
//! variable names a cut. The set is live: the environment gives its start, and the PC's debug
//! window's GPU tab changes it while the game runs; each backend reads it once per flush.
//!
//! Cuts of the scene's fragment shader (compiled in as `#define`s, so an unset cut costs
//! nothing in the shader):
//! - `fog`: the fog mix skipped;
//! - `alpha`: the alpha test never discards (a discard defeats early depth rejection);
//! - `tex`: no texture lookup at all (a flat grey texel);
//! - `texhot`: the lookups kept, all at one point of their texture (the same texels, always
//!   in the cache: the lookup's work without its memory traffic);
//! - `second`: the second pass skipped (GLES only: the PC draws a second pass as a batch of
//!   its own, which `seethru` removes);
//! - `flat`: the vertex colour only, no texture, alpha test, fog or second pass.
//!
//! Cuts of the draw state and the work issued:
//! - `blend`: blending never enabled;
//! - `depth`: the depth test never enabled (nothing is written to the depth either);
//! - `nearest` / `linear`: every texture filtered so;
//! - `half`: the frame drawn into a quarter of the target (half the width and height), a
//!   quarter of the fragments;
//! - `2d`: the retained background, the canvas rows and the quads not drawn;
//! - `scene`: no 3D batch drawn;
//! - `seethru`: the batches that blend not drawn; `solid`: the batches that do not blend
//!   not drawn.

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The cuts that are on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cuts {
    pub fog: bool,
    pub alpha: bool,
    pub tex: bool,
    pub texhot: bool,
    pub second: bool,
    pub flat: bool,
    pub blend: bool,
    pub depth: bool,
    pub nearest: bool,
    pub linear: bool,
    pub half: bool,
    pub two_d: bool,
    pub scene: bool,
    pub seethru: bool,
    pub solid: bool,
}

/// One cut: its name, what it removes, and the flag it sets.
pub struct Cut {
    pub name: &'static str,
    /// Shown by the debug window's GPU tab.
    #[cfg_attr(not(debug_ui), allow(dead_code))]
    pub about: &'static str,
    field: fn(&mut Cuts) -> &mut bool,
}

/// Every cut. The first `SHADER_CUTS` are the scene fragment shader's; a cut's place in this
/// list is its bit in `Cuts::bits`.
pub const CUTS: [Cut; 15] = [
    Cut {
        name: "fog",
        about: "the fog mix skipped",
        field: |c| &mut c.fog,
    },
    Cut {
        name: "alpha",
        about: "the alpha test never discards",
        field: |c| &mut c.alpha,
    },
    Cut {
        name: "tex",
        about: "no texture lookup (a flat grey texel)",
        field: |c| &mut c.tex,
    },
    Cut {
        name: "texhot",
        about: "every lookup at one point of its texture (always in the cache)",
        field: |c| &mut c.texhot,
    },
    Cut {
        name: "second",
        about: "the second pass skipped (handheld only; the PC's is a batch: seethru)",
        field: |c| &mut c.second,
    },
    Cut {
        name: "flat",
        about: "the vertex colour only: no texture, alpha test or fog",
        field: |c| &mut c.flat,
    },
    Cut {
        name: "blend",
        about: "blending never enabled",
        field: |c| &mut c.blend,
    },
    Cut {
        name: "depth",
        about: "no depth test, no depth writes",
        field: |c| &mut c.depth,
    },
    Cut {
        name: "nearest",
        about: "every texture filtered nearest",
        field: |c| &mut c.nearest,
    },
    Cut {
        name: "linear",
        about: "every texture filtered linearly",
        field: |c| &mut c.linear,
    },
    Cut {
        name: "half",
        about: "drawn into a quarter of the target (a quarter of the fragments)",
        field: |c| &mut c.half,
    },
    Cut {
        name: "2d",
        about: "the background, the canvas rows and the quads not drawn",
        field: |c| &mut c.two_d,
    },
    Cut {
        name: "scene",
        about: "no 3D batch drawn",
        field: |c| &mut c.scene,
    },
    Cut {
        name: "seethru",
        about: "the batches that blend not drawn",
        field: |c| &mut c.seethru,
    },
    Cut {
        name: "solid",
        about: "the batches that do not blend not drawn",
        field: |c| &mut c.solid,
    },
];

/// How many of the first cuts in `CUTS` belong to the scene's fragment shader.
const SHADER_CUTS: usize = 6;

/// The live set's bits, read by the backends once per flush.
static LIVE: AtomicU32 = AtomicU32::new(0);

/// The cuts now asked for (by `RC2K_GPU_CUT` at the start, or the debug window's tab since).
pub fn live() -> Cuts {
    Cuts::from_bits(LIVE.load(Relaxed))
}

/// Replaces the live set.
#[cfg_attr(not(debug_ui), allow(dead_code))]
pub fn set_live(cuts: Cuts) {
    LIVE.store(cuts.bits(), Relaxed);
}

impl Cuts {
    /// The cuts named by a comma-separated list, and the names it holds that are no cut.
    pub fn parse(list: &str) -> (Cuts, Vec<String>) {
        let mut cuts = Cuts::default();
        let mut unknown = Vec::new();
        for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            match CUTS.iter().find(|cut| cut.name == name) {
                Some(cut) => *(cut.field)(&mut cuts) = true,
                None => unknown.push(name.to_string()),
            }
        }
        (cuts, unknown)
    }

    /// The set `RC2K_GPU_CUT` asks for (none when unset), made the live set, and the log line
    /// that says which are on, if any is. Called once, when a GPU backend starts.
    pub fn start_from_env() -> Option<String> {
        let list = super::switches().gpu_cut.as_deref()?;
        let (cuts, unknown) = Cuts::parse(list);
        set_live(cuts);
        let mut line = format!("render: RC2K_GPU_CUT: cuts on: {}", cuts.names());
        if !unknown.is_empty() {
            line += &format!(" (not cuts: {})", unknown.join(", "));
        }
        Some(line)
    }

    /// The set as bits, one a cut, by its place in `CUTS`.
    pub fn bits(&self) -> u32 {
        let mut copy = *self;
        CUTS.iter()
            .enumerate()
            .filter(|(_, cut)| *(cut.field)(&mut copy))
            .fold(0, |bits, (n, _)| bits | 1 << n)
    }

    /// The set of `bits` (bits past the last cut are ignored).
    pub fn from_bits(bits: u32) -> Cuts {
        let mut cuts = Cuts::default();
        for (n, cut) in CUTS.iter().enumerate() {
            *(cut.field)(&mut cuts) = bits & 1 << n != 0;
        }
        cuts
    }

    /// Only the scene fragment shader's cuts, the rest left out.
    pub fn shader_part(&self) -> Cuts {
        Cuts::from_bits(self.bits() & ((1 << SHADER_CUTS) - 1))
    }

    /// The shader cuts as the PC shader reads them: its `flags` word's bits from the ninth.
    #[cfg_attr(target_arch = "aarch64", allow(dead_code))]
    pub fn flag_bits(&self) -> u32 {
        self.shader_part().bits() << 8
    }

    /// Whether a batch is not drawn: the `seethru` cut drops those that blend and the `solid`
    /// cut those that do not.
    pub fn drops(&self, blends: bool) -> bool {
        if blends { self.seethru } else { self.solid }
    }

    /// The names of the cuts that are on, comma-separated ("none" for no cut).
    pub fn names(&self) -> String {
        let mut copy = *self;
        let on: Vec<&str> = CUTS
            .iter()
            .filter(|cut| *(cut.field)(&mut copy))
            .map(|cut| cut.name)
            .collect();
        if on.is_empty() {
            "none".to_string()
        } else {
            on.join(",")
        }
    }

    /// The GLES scene's fragment shader with the shader cuts defined: the `#define` lines go
    /// after the `#version` line, which must stay the first.
    #[cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]
    pub fn shader(&self, source: &str) -> String {
        let mut copy = *self;
        let defines: String = CUTS[..SHADER_CUTS]
            .iter()
            .filter(|cut| *(cut.field)(&mut copy))
            .map(|cut| format!("#define CUT_{}\n", cut.name.to_uppercase()))
            .collect();
        match source.split_once('\n') {
            Some((version, rest)) => format!("{version}\n{defines}{rest}"),
            None => source.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_parsed_with_spaces_and_unknown_ones_reported() {
        let (cuts, unknown) = Cuts::parse(" fog, 2d ,half,,bogus");
        assert_eq!(
            cuts,
            Cuts {
                fog: true,
                two_d: true,
                half: true,
                ..Cuts::default()
            }
        );
        assert_eq!(unknown, ["bogus"]);
        assert_eq!(cuts.names(), "fog,half,2d");
    }

    #[test]
    fn an_empty_list_cuts_nothing_and_leaves_the_shader_alone() {
        let (cuts, unknown) = Cuts::parse("");
        assert_eq!(cuts, Cuts::default());
        assert!(unknown.is_empty());
        assert_eq!(cuts.names(), "none");
        assert_eq!(cuts.bits(), 0);
        assert_eq!(cuts.flag_bits(), 0);
        let source = "#version 300 es\nvoid main() {}\n";
        assert_eq!(cuts.shader(source), source);
    }

    #[test]
    fn shader_cuts_are_defined_after_the_version_line() {
        let (cuts, _) = Cuts::parse("tex,alpha,blend");
        assert_eq!(
            cuts.shader("#version 300 es\nvoid main() {}\n"),
            "#version 300 es\n#define CUT_ALPHA\n#define CUT_TEX\nvoid main() {}\n"
        );
    }

    #[test]
    fn bits_round_trip_and_each_cut_has_its_own() {
        let every = Cuts::from_bits((1 << CUTS.len()) - 1);
        assert_eq!(every.names().split(',').count(), CUTS.len());
        for n in 0..CUTS.len() {
            assert_eq!(Cuts::from_bits(1 << n).bits(), 1 << n);
        }
        let (cuts, _) = Cuts::parse("fog,flat,depth,solid");
        assert_eq!(Cuts::from_bits(cuts.bits()), cuts);
        assert_eq!(cuts.shader_part(), Cuts::parse("fog,flat").0);
        assert_eq!(cuts.flag_bits(), 0b100001 << 8);
    }

    #[test]
    fn the_shader_cuts_are_the_first_in_the_list() {
        let names: Vec<_> = CUTS[..SHADER_CUTS].iter().map(|c| c.name).collect();
        assert_eq!(names, ["fog", "alpha", "tex", "texhot", "second", "flat"]);
    }
}
