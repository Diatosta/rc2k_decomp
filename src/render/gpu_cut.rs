//! A diagnostic for what the GPU spends its time on (`RC2K_GPU_CUT=fog,alpha,...`): each name
//! removes one kind of GPU work from the handheld's GLES backend, so that a run with
//! `RC2K_GPU_TIME` shows the GPU's time without it. The pictures are wrong under a cut;
//! that is expected. Off (nothing removed, the shader the same text as without this module)
//! unless the variable names a cut.
//!
//! Cuts of the scene's fragment shader (compiled in as `#define`s, so an unset cut costs
//! nothing in the shader):
//! - `fog`: the fog mix skipped;
//! - `alpha`: the alpha test never discards (a discard defeats early depth rejection);
//! - `tex`: no texture lookup at all (a flat grey texel);
//! - `texhot`: the lookups kept, all at one point of their texture (the same texels, always
//!   in the cache: the lookup's work without its memory traffic);
//! - `second`: the second pass skipped;
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

impl Cuts {
    /// The cuts named by a comma-separated list, and the names it holds that are no cut.
    pub fn parse(list: &str) -> (Cuts, Vec<String>) {
        let mut cuts = Cuts::default();
        let mut unknown = Vec::new();
        for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            let flag = match name {
                "fog" => &mut cuts.fog,
                "alpha" => &mut cuts.alpha,
                "tex" => &mut cuts.tex,
                "texhot" => &mut cuts.texhot,
                "second" => &mut cuts.second,
                "flat" => &mut cuts.flat,
                "blend" => &mut cuts.blend,
                "depth" => &mut cuts.depth,
                "nearest" => &mut cuts.nearest,
                "linear" => &mut cuts.linear,
                "half" => &mut cuts.half,
                "2d" => &mut cuts.two_d,
                "scene" => &mut cuts.scene,
                "seethru" => &mut cuts.seethru,
                "solid" => &mut cuts.solid,
                other => {
                    unknown.push(other.to_string());
                    continue;
                }
            };
            *flag = true;
        }
        (cuts, unknown)
    }

    /// The cuts `RC2K_GPU_CUT` asks for (none when unset), and the log line that says which
    /// are on, if any is.
    pub fn from_env() -> (Cuts, Option<String>) {
        let Ok(list) = std::env::var("RC2K_GPU_CUT") else {
            return (Cuts::default(), None);
        };
        let (cuts, unknown) = Cuts::parse(&list);
        let mut line = format!("render: RC2K_GPU_CUT: cuts on: {}", cuts.names());
        if !unknown.is_empty() {
            line += &format!(" (not cuts: {})", unknown.join(", "));
        }
        (cuts, Some(line))
    }

    /// The names of the cuts that are on, comma-separated ("none" for no cut).
    pub fn names(&self) -> String {
        let all = [
            (self.fog, "fog"),
            (self.alpha, "alpha"),
            (self.tex, "tex"),
            (self.texhot, "texhot"),
            (self.second, "second"),
            (self.flat, "flat"),
            (self.blend, "blend"),
            (self.depth, "depth"),
            (self.nearest, "nearest"),
            (self.linear, "linear"),
            (self.half, "half"),
            (self.two_d, "2d"),
            (self.scene, "scene"),
            (self.seethru, "seethru"),
            (self.solid, "solid"),
        ];
        let on: Vec<&str> = all.iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect();
        if on.is_empty() {
            "none".to_string()
        } else {
            on.join(",")
        }
    }

    /// The scene's fragment shader with the shader cuts defined: the `#define` lines go
    /// after the `#version` line, which must stay the first.
    pub fn shader(&self, source: &str) -> String {
        let defines: String = [
            (self.fog, "CUT_FOG"),
            (self.alpha, "CUT_ALPHA"),
            (self.tex, "CUT_TEX"),
            (self.texhot, "CUT_TEXHOT"),
            (self.second, "CUT_SECOND"),
            (self.flat, "CUT_FLAT"),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, name)| format!("#define {name}\n"))
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
}
