//! Module system's owner: the program-wide state in one struct, handed by reference to the code
//! that uses it.

use crate::files::scratch::Scratch as FileScratch;
use crate::logfile::numeric::SCRATCH_LEN;
use crate::memory::session::{
    Clocks, DrawShared, GameThread, HostShared, KeyClock, Machine, PlainBytes, Screen,
    WindowLoopState, WindowShared,
};
use crate::records::stages_store::StageTables;
use crate::system::keep_start_time::KEPT_LEN;
use crate::system::load_game_data::RunCount;
use crate::system::read_command_line::CommandLine;
use crate::system::start_texts::StartTexts;

/// The screen's figures and what other threads read of the window, the machine and the clocks.
pub struct Session {
    /// Lent to the drawing parts (`scene::workers::Lent`): the game's thread sets the mode between
    /// frames.
    pub screen: DrawShared<Screen>,
    /// The window's flags the audio, key and network threads read (`quitting`, focus, active).
    pub flags: WindowShared,
    /// The window handle and the FPU flag the audio thread reads.
    pub host: HostShared,
    /// The key thread's count of turns.
    pub key_clock: KeyClock,
    /// The clock override of the replay viewer's picture saving (`clock::tick_count`): the
    /// input and audio threads read the clock too.
    pub picture_clock: crate::clock::tick_count::PictureClock,
}

impl Session {
    pub const fn new() -> Self {
        Session {
            screen: DrawShared::new(Screen::new()),
            flags: WindowShared::new(),
            host: HostShared::new(),
            key_clock: KeyClock::new(),
            picture_clock: crate::clock::tick_count::PictureClock::new(),
        }
    }
}

/// What the start-up keeps: the start's moment, the run count, the texts and
/// the command line.
pub struct Start {
    /// The nine bytes of the start's moment (`keep_start_time`).
    pub kept: PlainBytes<KEPT_LEN>,
    pub run_count: RunCount,
    pub texts: StartTexts,
    pub command_line: CommandLine,
}

impl Start {
    pub const fn new() -> Self {
        Start {
            kept: PlainBytes::new([0; KEPT_LEN]),
            run_count: RunCount::new(),
            texts: StartTexts::new(),
            command_line: CommandLine::new(),
        }
    }
}

/// The scratch areas: the shared 64 KB one and the numbers' text.
pub struct Work {
    pub files: FileScratch,
    /// Where a number's text is made.
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

/// What only the game's thread reaches: the start-up's keeps, the scratch areas, the
/// record tables, the main loop's state, the machine's facts and the menus' clocks (the
/// audio, key and network threads and the drawing parts touch none).
struct SystemMain {
    pub start: Start,
    pub work: Work,
    pub records: Records,
    pub window_loop: WindowLoopState,
    pub machine: Machine,
    pub clocks: Clocks,
}

impl SystemMain {
    pub const fn new() -> Self {
        SystemMain {
            start: Start::new(),
            work: Work::new(),
            records: Records::new(),
            window_loop: WindowLoopState::new(),
            machine: Machine::new(),
            clocks: Clocks::new(),
        }
    }
}

/// The program-wide state.
pub struct System {
    /// Read by other threads (the window's flags, the key ticks, the window handle and FPU
    /// flag) or by the drawing parts (the screen): atomic cells.
    pub session: Session,
    main: GameThread<SystemMain>,
}

impl System {
    pub fn start(&self) -> &Start {
        &self.main.get().start
    }

    pub fn work(&self) -> &Work {
        &self.main.get().work
    }

    pub fn records(&self) -> &Records {
        &self.main.get().records
    }

    /// The main loop's state (paused, surfaces lost, the two settings).
    pub fn window_loop(&self) -> &WindowLoopState {
        &self.main.get().window_loop
    }

    pub fn machine(&self) -> &Machine {
        &self.main.get().machine
    }

    pub fn clocks(&self) -> &Clocks {
        &self.main.get().clocks
    }
}

impl System {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            session: Session::new(),
            main: GameThread::new(SystemMain::new()),
        }
    }
}
