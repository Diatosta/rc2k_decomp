//! Module system's owner (step 11, owned state): the program-wide state in one struct, reached
//! through `system()` by code that has no reference to it.

use crate::files::scratch::Scratch as FileScratch;
use crate::logfile::numeric::SCRATCH_LEN;
use crate::records::stages_store::StageTables;
use crate::session::{Bytes, Clocks, Flags, Machine, Screen};
use crate::system::keep_start_time::KEPT_LEN;
use crate::system::load_game_data::RunCount;
use crate::system::read_command_line::CommandLine;
use crate::system::set_error_mode::ErrorMode;
use crate::system::start_texts::StartTexts;

/// The screen's figures, the window's flags, what the machine is and the clocks.
pub struct Session {
    pub screen: Screen,
    pub flags: Flags,
    pub machine: Machine,
    pub clocks: Clocks,
}

impl Session {
    pub const fn new() -> Self {
        Session {
            screen: Screen::new(),
            flags: Flags::new(),
            machine: Machine::new(),
            clocks: Clocks::new(),
        }
    }
}

/// What the start-up keeps: the start's moment, the run count, the error mode, the texts and
/// the command line.
pub struct Start {
    /// The nine bytes of the start's moment (`keep_start_time`).
    pub kept: Bytes<KEPT_LEN>,
    pub run_count: RunCount,
    pub error_mode: ErrorMode,
    pub texts: StartTexts,
    pub command_line: CommandLine,
}

impl Start {
    pub const fn new() -> Self {
        Start {
            kept: Bytes::new([0; KEPT_LEN]),
            run_count: RunCount::new(),
            error_mode: ErrorMode::new(),
            texts: StartTexts::new(),
            command_line: CommandLine::new(),
        }
    }
}

/// The scratch areas: the shared 64 KB one and the numbers' text.
pub struct Work {
    /// `files::scratch::SCRATCH` before the owner.
    pub files: FileScratch,
    /// `logfile::numeric::SCRATCH` before the owner: where a number's text is made.
    pub numbers: Bytes<SCRATCH_LEN>,
}

impl Work {
    pub const fn new() -> Self {
        Work {
            files: FileScratch::new(),
            numbers: Bytes::new([0; SCRATCH_LEN]),
        }
    }
}

/// The stage and rally record tables.
pub struct Records {
    pub stage_tables: StageTables,
}

impl Records {
    pub const fn new() -> Self {
        Records {
            stage_tables: StageTables::new(),
        }
    }
}

/// The program-wide state.
pub struct System {
    pub session: Session,
    pub start: Start,
    pub work: Work,
    pub records: Records,
}

/// The one instance. Reached through `system()`; `moved` registers its stores' places.
pub static SYSTEM: System = System {
    session: Session::new(),
    start: Start::new(),
    work: Work::new(),
    records: Records::new(),
};

/// The program-wide state; pass it down where a function reaches it more than once.
pub fn system() -> &'static System {
    &SYSTEM
}
