//! Module system's owner (step 11, owned state): the program-wide state in one struct, reached
//! through `system()` by code that has no reference to it.

use crate::files::scratch::Scratch as FileScratch;
use crate::logfile::numeric::SCRATCH_LEN;
use crate::records::stages_store::StageTables;
use crate::session::{Clocks, GameThread, Machine, PlainBytes, Screen, WindowLoopState};
use crate::system::keep_start_time::KEPT_LEN;
use crate::system::load_game_data::RunCount;
use crate::system::read_command_line::CommandLine;
use crate::system::set_error_mode::ErrorMode;
use crate::system::start_texts::StartTexts;

/// The screen's figures, the window's flags, what the machine is and the clocks.
pub struct Session {
    pub screen: Screen,
    pub flags: WindowLoopState,
    pub machine: Machine,
    pub clocks: Clocks,
}

impl Session {
    pub const fn new() -> Self {
        Session {
            screen: Screen::new(),
            flags: WindowLoopState::new(),
            machine: Machine::new(),
            clocks: Clocks::new(),
        }
    }
}

/// What the start-up keeps: the start's moment, the run count, the error mode, the texts and
/// the command line.
pub struct Start {
    /// The nine bytes of the start's moment (`keep_start_time`).
    pub kept: PlainBytes<KEPT_LEN>,
    pub run_count: RunCount,
    pub error_mode: ErrorMode,
    pub texts: StartTexts,
    pub command_line: CommandLine,
}

impl Start {
    pub const fn new() -> Self {
        Start {
            kept: PlainBytes::new([0; KEPT_LEN]),
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
    pub numbers: PlainBytes<SCRATCH_LEN>,
}

impl Work {
    pub const fn new() -> Self {
        Work {
            files: FileScratch::new(),
            numbers: PlainBytes::new([0; SCRATCH_LEN]),
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

/// What only the game's thread reaches: the start-up's keeps, the scratch areas and the
/// record tables (the audio, key and network threads and the drawing parts touch none).
pub struct SystemMain {
    pub start: Start,
    pub work: Work,
    pub records: Records,
}

impl SystemMain {
    pub const fn new() -> Self {
        SystemMain {
            start: Start::new(),
            work: Work::new(),
            records: Records::new(),
        }
    }
}

/// The program-wide state.
pub struct System {
    /// Read by other threads (the flags, the clocks' key ticks, the machine's window) or by
    /// the drawing parts (the screen, the MMX flag): atomic cells.
    pub session: Session,
    main: GameThread<SystemMain>,
}

impl System {
    /// What the start-up keeps.
    pub fn start(&self) -> &Start {
        &self.main.get().start
    }

    /// The scratch areas.
    pub fn work(&self) -> &Work {
        &self.main.get().work
    }

    /// The stage and rally record tables.
    pub fn records(&self) -> &Records {
        &self.main.get().records
    }
}

/// The one instance. Reached through `system()`.
pub static SYSTEM: System = System {
    session: Session::new(),
    main: GameThread::new(SystemMain::new()),
};

/// The program-wide state; pass it down where a function reaches it more than once.
pub fn system() -> &'static System {
    &SYSTEM
}
