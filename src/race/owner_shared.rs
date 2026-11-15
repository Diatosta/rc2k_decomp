//! The race owner's shared group (`race::owner::Race::shared`): the race's stores the menus, the
//! network and the audio side read too.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::blocks_store::CarBlocks;
use crate::race::buffers_store::Buffers;
use crate::race::replay::Replays;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::replay_path::Templates;
use crate::race::state_store::RaceState;
use crate::race::switches_store::FrameSwitches;
use crate::race::tables_store::RaceTables;
use crate::race::tyre_store::TyreChange;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::race::work_store::RaceWork;
use crate::session::{DrawShared, GameThread};
use rc2k_core::net::net_race::NetRace;

/// The race's stores the menus, the network and the audio side read too.
pub struct RaceShared {
    /// One copy: no drawing part writes it.
    pub work: DrawShared<RaceWork>,
    pub state: RaceState,
    pub buffers: Buffers,
    /// One copy: no drawing part writes it.
    pub vars: DrawShared<RaceVars>,
    /// One copy: no drawing part writes it.
    pub view: DrawShared<RaceView>,
    /// One copy: no drawing part writes it.
    pub switches: DrawShared<FrameSwitches>,
    /// One copy: the drawing only reads it.
    pub physics_tables: DrawShared<PhysicsTables>,
    /// The work car. One copy: the drawing does not read it, so the drawing threads need none of
    /// their own.
    car: GameThread<Car>,
    /// The cars' blocks. One copy: no drawing part writes them (each view draws its own copy of
    /// its car, `ViewScratch::car`).
    pub car_blocks: DrawShared<CarBlocks>,
    /// The replay files' names.
    pub replay_templates: Templates,
    /// The turn the race's sounds are placed by.
    pub heard_turn: GameThread<crate::race::heard_turn::HeardTurn>,
    main: GameThread<SharedMain>,
}

/// What only the game's thread reaches among the shared stores (no drawing part and no other
/// thread touches them).
struct SharedMain {
    tables: RaceTables,
    tyre_change: TyreChange,
    replay_file: ReplayFileBlock,
    replays: Replays,
    /// `race::viewer_buffers_store`: the replay viewer's buffers.
    viewer: ViewerBuffers,
}

impl RaceShared {
    pub const fn new() -> Self {
        RaceShared {
            work: DrawShared::new(RaceWork::new()),
            state: RaceState::new(),
            buffers: Buffers::new(),
            vars: DrawShared::new(RaceVars::new()),
            view: DrawShared::new(RaceView::new()),
            switches: DrawShared::new(FrameSwitches::new()),
            physics_tables: DrawShared::new(PhysicsTables::new()),
            car: GameThread::new(Car::new()),
            car_blocks: DrawShared::new(CarBlocks::new()),
            replay_templates: Templates::new(),
            heard_turn: GameThread::new(crate::race::heard_turn::HeardTurn::new()),
            main: GameThread::new(SharedMain {
                tables: RaceTables::new(),
                tyre_change: TyreChange::new(),
                replay_file: ReplayFileBlock::new(),
                replays: Replays::new(),
                viewer: ViewerBuffers::new(),
            }),
        }
    }

    /// The work car (`CAR`): the game thread's.
    pub fn car(&self) -> &Car {
        self.car.get()
    }

    /// The cars' blocks, on the game's thread outside a drawing scope (a drawing part
    /// reads them through its `workers::DrawReads`).
    pub fn car_blocks(&self) -> &CarBlocks {
        self.car_blocks.get()
    }

    pub fn tables(&self) -> &RaceTables {
        &self.main.get().tables
    }

    pub fn tyre_change(&self) -> &TyreChange {
        &self.main.get().tyre_change
    }

    pub fn replay_file(&self) -> &ReplayFileBlock {
        &self.main.get().replay_file
    }

    pub fn replays(&self) -> &Replays {
        &self.main.get().replays
    }

    pub fn viewer_buffers(&self) -> &ViewerBuffers {
        &self.main.get().viewer
    }

    /// Car `n`'s block (`car_blocks[n]`), refused for a value that names no block (see
    /// `RaceWork::car_block` for the read past the eight).
    pub fn race_car(&self, n: usize) -> Result<&Car, BadInput> {
        Self::race_car_in(self.work.get(), self.car_blocks(), n)
    }

    /// `race_car` in the blocks `blocks` (a drawing part's, `workers::DrawReads`).
    pub fn race_car_in<'b>(
        work: &RaceWork,
        blocks: &'b CarBlocks,
        n: usize,
    ) -> Result<&'b Car, BadInput> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        if let Some(cell) = work.car_blocks.get(n) {
            let k = (cell.get() ^ BLOCK_TAG) as usize;
            if k < BLOCK_COUNT {
                return Ok(&blocks.cars[k]);
            }
        }
        blocks.named(work.car_block(n))
    }

    /// Car `n`'s block, or the refusal `what` for none. A handle that names no block ends the
    /// program, as `RaceWork::car_block`.
    pub fn race_car_or(&self, n: usize, what: &'static str) -> Result<&Car, BadInput> {
        Self::race_car_or_in(self.work.get(), self.car_blocks(), n, what)
    }

    /// `race_car_or` in the blocks `blocks` (a drawing part's, `workers::DrawReads`).
    pub fn race_car_or_in<'b>(
        work: &RaceWork,
        blocks: &'b CarBlocks,
        n: usize,
        what: &'static str,
    ) -> Result<&'b Car, BadInput> {
        match work.car_blocks.get(n) {
            Some(cell) if cell.get() == 0 => Err(BadInput(what)),
            Some(_) => Self::race_car_in(work, blocks, n),
            None => blocks.at(work.car_block(n), what),
        }
    }

    /// Car `n`'s block; a null block or one that is none of the eight ends the program.
    pub fn race_car_or_end(&self, n: usize) -> &Car {
        Self::race_car_or_end_in(self.work.get(), self.car_blocks(), n)
    }

    /// `race_car_or_end` in the blocks `blocks` (a drawing part's, `workers::DrawReads`).
    pub fn race_car_or_end_in<'b>(work: &RaceWork, blocks: &'b CarBlocks, n: usize) -> &'b Car {
        let block = work.car_block(n);
        match work.car_blocks.get(n) {
            Some(_) if block != 0 => Self::race_car_in(work, blocks, n),
            _ => blocks.named(block),
        }
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::Car::at", e))
    }

    /// View `n`'s car block (None for none).
    pub fn view_car(&self, n: usize) -> Option<&Car> {
        self.car_blocks().of(self.view.get().view_blocks[n].get())
    }

    /// View `n`'s car block, or the end of the program for none (as `Car::at` on its address).
    pub fn view_car_or_end(&self, n: usize) -> &Car {
        self.view_car(n).unwrap_or_else(|| {
            crate::fatal::raise::end(
                "physics::Car::at",
                BadInput("a car block at no car's place"),
            )
        })
    }

    /// The car this machine plays: the first, but in a network race its own (so on a joined
    /// machine the cockpit's effects, the mud and the listener follow its car, not the host's).
    pub fn own_car_number(&self, net_race: Option<NetRace>) -> usize {
        if net_race.is_some() {
            let local = self.view.get().local_car.get() as usize;
            if local < 8 { local } else { 0 }
        } else {
            0
        }
    }

    /// `own_car_number`'s block.
    pub fn own_car(&self, net_race: Option<NetRace>) -> &Car {
        &self.car_blocks().cars[self.own_car_number(net_race)]
    }
}
