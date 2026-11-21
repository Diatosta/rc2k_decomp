//! The renderer's developer harnesses: tools for checking and measuring the drawing that the
//! player never needs. Each is off unless its `RC2K_*` environment switch is set; the switches
//! are read once, here (`switches`), and the draw paths call one entry point per hook point
//! (`dump::dump_if_asked`, `seethru::note_list`, the `check` functions, ...).
//!
//! | switch | what |
//! | --- | --- |
//! | `RC2K_CANVAS_CHECK` | log canvas rows holding a written pixel that were not noted |
//! | `RC2K_DUMP=n` | write the n-th flush of 1000 fans or more to `rc2k_frame_dump.bin` |
//! | `RC2K_FAN_HASH` | log a hash of every race flush's commands |
//! | `RC2K_SEETHRU_AREA`, `RC2K_SEETHRU_DUMP` | the see-through areas' pixel measurement and its dumps |
//! | `RC2K_GL_BENCH=n` | the GL thread's bench of one part in n |
//! | `RC2K_GPU_CUT` | the GPU draw's cut list (also the debug window's Switches) |
//! | `RC2K_GPU_TIME=n` | the GPU's wait measured every n-th frame |
//! | `RC2K_BACKGROUND_CHECK`, `RC2K_CHECK_SAME` | the GPU's second drawing and its comparison |

#[cfg(not(handheld))]
pub(crate) mod check;
pub(crate) mod dump;
pub(crate) mod gpu_cut;
pub(crate) mod gpu_time;
pub(crate) mod seethru;

#[cfg(not(handheld))]
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

/// What the developer switches ask for, read from the environment once.
#[derive(Debug, Default)]
pub struct DevSwitches {
    /// `RC2K_CANVAS_CHECK`.
    pub canvas_check: bool,
    /// `RC2K_DUMP`: the flush to write.
    pub dump_flush: Option<u32>,
    /// `RC2K_FAN_HASH`.
    pub fan_hash: bool,
    /// `RC2K_SEETHRU_AREA`: set and neither empty nor 0.
    pub seethru: bool,
    /// `RC2K_SEETHRU_DUMP`: the folder for the dumps.
    pub seethru_dump: Option<PathBuf>,
    /// `RC2K_GL_BENCH`: the bench measures every this many parts.
    pub gl_bench_every: Option<u32>,
    /// `RC2K_GPU_CUT`: the list of cuts.
    pub gpu_cut: Option<String>,
    /// `RC2K_GPU_TIME`: the frames between waits.
    pub gpu_time_every: Option<u32>,
    #[cfg(not(handheld))]
    /// `RC2K_BACKGROUND_CHECK`: set; its value (longer than a character) is the folder for the
    /// pictures of the frames that differ.
    pub background_check: Option<OsString>,
    #[cfg(not(handheld))]
    /// `RC2K_CHECK_SAME`.
    pub check_same: bool,
}

impl DevSwitches {
    fn from_environment() -> Self {
        let text = |name: &str| std::env::var(name).ok();
        let set = |name: &str| std::env::var_os(name).is_some();
        Self {
            canvas_check: set("RC2K_CANVAS_CHECK"),
            dump_flush: text("RC2K_DUMP").and_then(|n| n.parse().ok()),
            fan_hash: set("RC2K_FAN_HASH"),
            seethru: text("RC2K_SEETHRU_AREA").is_some_and(|v| v != "0" && !v.is_empty()),
            seethru_dump: std::env::var_os("RC2K_SEETHRU_DUMP").map(Into::into),
            gl_bench_every: text("RC2K_GL_BENCH")
                .and_then(|v| v.trim().parse().ok())
                .filter(|&n| n > 0),
            gpu_cut: text("RC2K_GPU_CUT"),
            gpu_time_every: text("RC2K_GPU_TIME").and_then(|v| v.trim().parse().ok()),
            #[cfg(not(handheld))]
            background_check: std::env::var_os("RC2K_BACKGROUND_CHECK"),
            #[cfg(not(handheld))]
            check_same: set("RC2K_CHECK_SAME"),
        }
    }
}

/// The developer switches, read at the first call.
pub fn switches() -> &'static DevSwitches {
    static SWITCHES: OnceLock<DevSwitches> = OnceLock::new();
    SWITCHES.get_or_init(DevSwitches::from_environment)
}
