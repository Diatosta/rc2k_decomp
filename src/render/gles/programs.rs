//! Building and choosing the programs: shader compilation, the scene program per pass kind,
//! and the texture filters in force.

use super::*;

impl Gpu {
    pub(super) fn program(&self, vertex: &CStr, fragment: &CStr) -> Result<u32, String> {
        let (v, f) = (
            self.gl.compile_shader(0x8B31, vertex)?,
            self.gl.compile_shader(0x8B30, fragment)?,
        );
        Ok(self.gl.link_program(v, f))
    }

    pub(super) fn uniform(&self, program: u32, name: &str) -> i32 {
        self.gl.uniform_location(program, name)
    }

    pub(super) fn quad(&self, vertex: &CStr, fragment: &CStr) -> Result<Quad, String> {
        let program = self.program(vertex, fragment)?;
        Ok(Quad {
            program,
            flip: self.uniform(program, "flip"),
            fit: self.uniform(program, "fit"),
            bars: self.uniform(program, "bars"),
            wash: self.uniform(program, "wash"),
            has_wash: self.uniform(program, "has_wash"),
        })
    }

    /// The scene's program for `kind` with the shader cuts of `cuts` defined, and where its
    /// values go; its picture units are set.
    pub(super) fn scene_program(
        &self,
        cuts: Cuts,
        kind: ProgramKind,
    ) -> Result<SceneProgram, String> {
        let defines = kind.defines();
        let text = |source: &CStr| {
            let source = crate::render::batches::with_defines(&source.to_string_lossy(), &defines);
            std::ffi::CString::new(cuts.shader(&source))
                .map_err(|_| "the scene's shader holds a NUL".to_string())
        };
        let program = self.program(&text(SCENE_VS)?, &text(SCENE_FS)?)?;
        // A value the kind's program does not read has no place (-1), and setting it does nothing.
        let values = |suffix: &str| ValueLocations {
            fog_colour: self.uniform(program, &format!("fog_colour{suffix}")),
            alpha_reference: self.uniform(program, &format!("alpha_reference{suffix}")),
            alpha_function: self.uniform(program, &format!("alpha_function{suffix}")),
            mixing: self.uniform(program, &format!("mixing{suffix}")),
            flags: self.uniform(program, &format!("flags{suffix}")),
        };
        let locations = Locations {
            size: self.uniform(program, "size"),
            first: values(""),
            second: values("2"),
        };
        self.gl.use_program(program);
        for (unit, name) in ["picture_a", "picture_b", "picture2_a", "picture2_b"]
            .into_iter()
            .enumerate()
        {
            self.gl.uniform_1i(self.uniform(program, name), unit as i32);
        }
        Ok(SceneProgram {
            name: program,
            locations,
            size: None,
            first: None,
            second: None,
        })
    }

    /// Which of `programs` draws `kind` under the shader cuts of `cuts`: made on first use. One
    /// that does not build is logged once and another drawn with from then on: the program for
    /// every kind (with the alpha test, if the one without is what failed), then the first.
    /// Making one leaves it the program in use.
    pub(super) fn program_number(&mut self, cuts: Cuts, kind: ProgramKind) -> usize {
        let shader_cuts = cuts.shader_part();
        let key = (shader_cuts.bits(), kind);
        if let Some(&number) = self.program_numbers.get(&key) {
            return number;
        }
        let started = std::time::Instant::now();
        let number = match self.scene_program(shader_cuts, kind) {
            Ok(program) => {
                self.programs.push(program);
                // A kind missing from `KINDS_AT_START` holds a frame up once: the log names it,
                // so that it can be added there.
                if self.set_up_done && cuts == Cuts::default() {
                    log(&format!(
                        "render: a scene program made while drawing, in {:.1} ms, for {kind:?}",
                        started.elapsed().as_secs_f64() * 1000.0
                    ));
                }
                self.programs.len() - 1
            }
            Err(why) => {
                log(&format!(
                    "render: the scene's shader for {kind:?} with cuts {} failed: {why}",
                    cuts.names()
                ));
                match kind {
                    ProgramKind::Made(_) => self.program_number(
                        cuts,
                        ProgramKind::Any {
                            discard_free: kind.discard_free(),
                        },
                    ),
                    ProgramKind::Any { discard_free: true } => self.program_number(
                        cuts,
                        ProgramKind::Any {
                            discard_free: false,
                        },
                    ),
                    ProgramKind::Any {
                        discard_free: false,
                    } => 0,
                }
            }
        };
        self.program_numbers.insert(key, number);
        number
    }

    /// Takes the live cuts for the flush about to be drawn: the samplers' filters (the scene's
    /// programs are chosen by the cuts batch by batch).
    pub(super) fn take_cuts(&mut self) {
        let cuts = crate::render::dev::gpu_cut::live();
        if cuts == self.cuts {
            return;
        }
        self.set_filters(cuts.nearest, cuts.linear);
        log(&format!("render: GPU cuts now on: {}", cuts.names()));
        self.cuts = cuts;
    }

    /// Every sampler's filter: nearest for the even ones and linear for the odd, unless a cut
    /// makes all one or the other; the last four (mipmapping) are linear across the nearest smaller
    /// level.
    pub(super) fn set_filters(&self, nearest: bool, linear_cut: bool) {
        for (n, &sampler) in self.samplers.iter().enumerate() {
            let linear = (n & 1 != 0 || linear_cut) && !nearest;
            let filter = if linear { 0x2601 } else { 0x2600 };
            let far = if linear && n & 4 != 0 { 0x2701 } else { filter };
            let wrap = if n & 2 != 0 { 0x812F } else { 0x2901 };
            self.gl.sampler_parameter(sampler, 0x2801, far);
            self.gl.sampler_parameter(sampler, 0x2800, filter);
            self.gl.sampler_parameter(sampler, 0x2802, wrap);
            self.gl.sampler_parameter(sampler, 0x2803, wrap);
        }
    }
}
