//! The race owner's shared group (`race::owner::Race::shared`): the race's stores the menus, the
//! network and the audio side read too.

use crate::errors::BadInput;
use crate::memory::session::{DrawArc, DrawPlain, GameThread};
use crate::physics::car_store::Car;
use crate::physics::run_store::PhysicsRun;
use crate::physics::tables_store::PhysicsTables;
use crate::race::blocks_store::CarBlocks;
use crate::race::buffers_store::Buffers;
use crate::race::checkpoints_store::{Checkpoints, CheckpointsRef};
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
use rc2k_core::net::net_race::NetRace;
use std::cell::{Ref, RefCell, RefMut};

/// The race's stores the menus, the network and the audio side read too.
pub struct RaceShared {
    /// The race's work variables, lent to the drawing parts as they stood when their scope opened.
    pub work: DrawArc<RaceWork>,
    pub state: RaceState,
    pub buffers: Buffers,
    /// The checkpoint tables: shared with the drawing parts copy-on-write.
    pub checkpoints: DrawArc<Checkpoints>,
    /// One copy: no drawing part writes it.
    pub vars: crate::memory::session::GameThread<RaceVars>,
    /// One copy: no drawing part writes it.
    pub view: GameThread<RaceView>,
    /// One copy: no drawing part writes it.
    pub switches: DrawPlain<FrameSwitches>,
    /// One copy: the drawing only reads it.
    pub physics_tables: crate::memory::session::DrawArc<PhysicsTables>,
    /// The work car. One copy: the drawing does not read it, so the drawing threads need none of
    /// their own.
    car: GameThread<RefCell<Car>>,
    /// The cars' blocks. One copy: no drawing part writes them (each view draws its own copy of
    /// its car, `ViewScratch::car`).
    pub car_blocks: DrawArc<CarBlocks>,
    /// What the drawing keeps of each car (`race::car_parts_store`), written by the game's thread
    /// before a frame's views are drawn and only read by the drawing parts (lent as an `Arc`).
    pub car_parts: crate::memory::session::DrawArc<crate::race::car_parts_store::CarParts>,
    /// The replay files' names.
    pub replay_templates: Templates,
    /// The turn the race's sounds are placed by.
    pub heard_turn: GameThread<crate::race::heard_turn::HeardTurn>,
    /// Whether the cars of the replay now viewed are all heard (`race::heard_cars`).
    pub every_car_heard: std::sync::atomic::AtomicBool,
    main: GameThread<SharedMain>,
    /// The random numbers: the main generator and the sounds' and drawing's own streams. Drawing
    /// threads draw from it in the list's order (`Randoms::next_drawing`).
    pub random: crate::math::next_random::Randoms,
}

/// What only the game's thread reaches among the shared stores (no drawing part and no other
/// thread touches them).
struct SharedMain {
    tables: RaceTables,
    tyre_change: RefCell<TyreChange>,
    replay_file: ReplayFileBlock,
    replays: Replays,
    /// `race::viewer_buffers_store`: the replay viewer's buffers.
    viewer: ViewerBuffers,
    /// The race's clocks and input words. Borrowed for one statement at a time; the drawing
    /// parts read the scene clock from the copy their scope was opened with.
    run: RefCell<PhysicsRun>,
}

impl RaceShared {
    pub const fn new() -> Self {
        RaceShared {
            work: DrawArc::new(RaceWork::new),
            state: RaceState::new(),
            buffers: Buffers::new(),
            checkpoints: DrawArc::new(Checkpoints::new),
            vars: crate::memory::session::GameThread::new(RaceVars::new()),
            view: GameThread::new(RaceView::new()),
            switches: DrawPlain::new(FrameSwitches::new()),
            physics_tables: DrawArc::new(PhysicsTables::new),
            car: GameThread::new(RefCell::new(Car::new())),
            car_blocks: DrawArc::new(CarBlocks::new),
            car_parts: crate::memory::session::DrawArc::new(
                crate::race::car_parts_store::CarParts::new,
            ),
            replay_templates: Templates::new(),
            heard_turn: GameThread::new(crate::race::heard_turn::HeardTurn::new()),
            every_car_heard: std::sync::atomic::AtomicBool::new(false),
            random: crate::math::next_random::Randoms::new(),
            main: GameThread::new(SharedMain {
                tables: RaceTables::new(),
                tyre_change: RefCell::new(TyreChange::new()),
                replay_file: ReplayFileBlock::new(),
                replays: Replays::new(),
                viewer: ViewerBuffers::new(),
                run: RefCell::new(PhysicsRun::new()),
            }),
        }
    }

    /// The checkpoint tables, on the game's thread.
    pub fn checkpoints(&self) -> CheckpointsRef<'_> {
        CheckpointsRef::Store(&self.checkpoints)
    }

    /// The work car (`CAR`): the game thread's.
    pub fn car(&self) -> Ref<'_, Car> {
        self.car.get().borrow()
    }

    /// The work car, to write: no other borrow may be held.
    pub fn car_mut(&self) -> RefMut<'_, Car> {
        self.car.get().borrow_mut()
    }

    /// The cars' blocks, on the game's thread outside a drawing scope (a drawing part
    /// reads them through its `scene::workers::DrawReads`).
    pub fn car_blocks(&self) -> Ref<'_, CarBlocks> {
        self.car_blocks.borrow()
    }

    /// The cars' blocks, to write, on the game's thread outside a drawing scope: no other borrow
    /// may be held.
    pub fn car_blocks_mut(&self) -> RefMut<'_, CarBlocks> {
        self.car_blocks.borrow_mut()
    }

    /// The physics' tables and work variables, read, on the game's thread outside a drawing scope
    /// (a drawing part reads them through its `scene::workers::DrawReads`): no write borrow may be
    /// held.
    pub fn physics_tables(&self) -> std::cell::Ref<'_, PhysicsTables> {
        self.physics_tables.borrow()
    }

    /// The physics' tables, to write, on the game's thread outside a drawing scope: no other
    /// borrow of them may be held.
    pub fn physics_tables_mut(&self) -> std::cell::RefMut<'_, PhysicsTables> {
        self.physics_tables.borrow_mut()
    }

    /// The cars' drawing state, on the game's thread outside a drawing scope (a drawing part reads
    /// it through its `scene::workers::DrawReads`).
    pub fn car_parts(&self) -> std::cell::Ref<'_, crate::race::car_parts_store::CarParts> {
        self.car_parts.borrow()
    }

    /// The cars' drawing state, to write, on the game's thread outside a drawing scope.
    pub fn car_parts_mut(&self) -> std::cell::RefMut<'_, crate::race::car_parts_store::CarParts> {
        self.car_parts.borrow_mut()
    }

    /// The race's work variables, on the game's thread outside a drawing scope (a drawing part reads
    /// them through its `scene::workers::DrawReads`).
    pub fn work(&self) -> Ref<'_, RaceWork> {
        self.work.borrow()
    }

    /// The race's work variables, to write, on the game's thread outside a drawing scope: no other
    /// borrow of them may be held.
    pub fn work_mut(&self) -> RefMut<'_, RaceWork> {
        self.work.borrow_mut()
    }

    /// The race's clocks and input words, read: no write borrow may be held.
    pub fn run(&self) -> Ref<'_, PhysicsRun> {
        self.main.get().run.borrow()
    }

    /// The race's clocks and input words, to write: no other borrow may be held.
    pub fn run_mut(&self) -> RefMut<'_, PhysicsRun> {
        self.main.get().run.borrow_mut()
    }

    pub fn tables(&self) -> &RaceTables {
        &self.main.get().tables
    }

    /// The tyre change's variables, read: no write borrow may be held.
    pub fn tyre_change(&self) -> Ref<'_, TyreChange> {
        self.main.get().tyre_change.borrow()
    }

    /// The tyre change's variables, to write: no other borrow may be held.
    pub fn tyre_change_mut(&self) -> RefMut<'_, TyreChange> {
        self.main.get().tyre_change.borrow_mut()
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

    /// The number of car `n`'s block, refused for a value that names no block (see
    /// `RaceWork::car_block` for the read past the eight).
    fn race_index_in(work: &RaceWork, n: usize) -> Result<usize, BadInput> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        if let Some(cell) = work.car_blocks.get(n) {
            let k = (*cell ^ BLOCK_TAG) as usize;
            if k < BLOCK_COUNT {
                return Ok(k);
            }
        }
        CarBlocks::named_index(work.car_block(n))
    }

    /// The number of car `n`'s block, or the refusal `what` for none; a handle that names no block
    /// ends the program, as `RaceWork::car_block`.
    fn race_index_or_in(work: &RaceWork, n: usize, what: &'static str) -> Result<usize, BadInput> {
        match work.car_blocks.get(n) {
            Some(cell) if *cell == 0 => Err(BadInput(what)),
            Some(_) => Self::race_index_in(work, n),
            None => CarBlocks::index_at(work.car_block(n), what),
        }
    }

    /// The number of car `n`'s block; a null block or one that is none of the eight ends the
    /// program.
    fn race_index_or_end_in(work: &RaceWork, n: usize) -> usize {
        let block = work.car_block(n);
        match work.car_blocks.get(n) {
            Some(_) if block != 0 => Self::race_index_in(work, n),
            _ => CarBlocks::named_index(block),
        }
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::Car::at", e))
    }

    /// The number of car `n`'s block, or the refusal `what` for none (as `race_car_or`).
    pub fn race_index_or_in_pub(&self, n: usize, what: &'static str) -> Result<usize, BadInput> {
        Self::race_index_or_in(&self.work(), n, what)
    }

    /// Car `n`'s block (`car_blocks[n]`), refused for a value that names no block (see
    /// `RaceWork::car_block` for the read past the eight).
    pub fn race_car(&self, n: usize) -> Result<Ref<'_, Car>, BadInput> {
        let k = Self::race_index_in(&self.work(), n)?;
        Ok(Ref::map(self.car_blocks(), |b| &b.cars[k]))
    }

    /// `race_car` to write: no other borrow of the blocks may be held.
    pub fn race_car_mut(&self, n: usize) -> Result<RefMut<'_, Car>, BadInput> {
        let k = Self::race_index_in(&self.work(), n)?;
        Ok(RefMut::map(self.car_blocks_mut(), |b| &mut b.cars[k]))
    }

    /// `race_car` in the blocks `blocks` (a drawing part's, `scene::workers::DrawReads`).
    pub(super) fn race_car_in<'b>(
        work: &RaceWork,
        blocks: &'b CarBlocks,
        n: usize,
    ) -> Result<&'b Car, BadInput> {
        Ok(&blocks.cars[Self::race_index_in(work, n)?])
    }

    /// Car `n`'s block, or the refusal `what` for none. A handle that names no block ends the
    /// program, as `RaceWork::car_block`.
    pub fn race_car_or(&self, n: usize, what: &'static str) -> Result<Ref<'_, Car>, BadInput> {
        let k = Self::race_index_or_in(&self.work(), n, what)?;
        Ok(Ref::map(self.car_blocks(), |b| &b.cars[k]))
    }

    /// `race_car_or` to write.
    pub fn race_car_or_mut(
        &self,
        n: usize,
        what: &'static str,
    ) -> Result<RefMut<'_, Car>, BadInput> {
        let k = Self::race_index_or_in(&self.work(), n, what)?;
        Ok(RefMut::map(self.car_blocks_mut(), |b| &mut b.cars[k]))
    }

    /// `race_car_or` in the blocks `blocks` (a drawing part's, `scene::workers::DrawReads`).
    pub fn race_car_or_in<'b>(
        work: &RaceWork,
        blocks: &'b CarBlocks,
        n: usize,
        what: &'static str,
    ) -> Result<&'b Car, BadInput> {
        Ok(&blocks.cars[Self::race_index_or_in(work, n, what)?])
    }

    /// Car `n`'s block; a null block or one that is none of the eight ends the program.
    pub fn race_car_or_end(&self, n: usize) -> Ref<'_, Car> {
        let k = Self::race_index_or_end_in(&self.work(), n);
        Ref::map(self.car_blocks(), |b| &b.cars[k])
    }

    /// The number of car `n`'s block; a null block or one that is none of the eight ends the
    /// program.
    pub fn race_index_or_end(&self, n: usize) -> usize {
        Self::race_index_or_end_in(&self.work(), n)
    }

    /// `race_car_or_end` to write.
    pub fn race_car_or_end_mut(&self, n: usize) -> RefMut<'_, Car> {
        let k = Self::race_index_or_end_in(&self.work(), n);
        RefMut::map(self.car_blocks_mut(), |b| &mut b.cars[k])
    }

    /// `race_car_or_end` in the blocks `blocks` (a drawing part's, `scene::workers::DrawReads`).
    pub fn race_car_or_end_in<'b>(work: &RaceWork, blocks: &'b CarBlocks, n: usize) -> &'b Car {
        &blocks.cars[Self::race_index_or_end_in(work, n)]
    }

    /// The number of view `n`'s block (None for none).
    pub fn view_index(&self, n: usize) -> Option<usize> {
        match self.view.get().view_blocks.get(n) {
            Some(block) => CarBlocks::index_of(block.get()),
            // The further views of a split screen show their own player's car.
            None if n < 4 => CarBlocks::index_of(crate::race::blocks_store::block_handle(n)),
            None => None,
        }
    }

    /// View `n`'s car block (None for none).
    pub fn view_car(&self, n: usize) -> Option<Ref<'_, Car>> {
        let k = self.view_index(n)?;
        Some(Ref::map(self.car_blocks(), |b| &b.cars[k]))
    }

    /// View `n`'s car block to write (None for none).
    pub fn view_car_mut(&self, n: usize) -> Option<RefMut<'_, Car>> {
        let k = self.view_index(n)?;
        Some(RefMut::map(self.car_blocks_mut(), |b| &mut b.cars[k]))
    }

    /// View `n`'s car block to write, or the end of the program for none.
    pub(super) fn view_car_or_end_mut(&self, n: usize) -> RefMut<'_, Car> {
        self.view_car_mut(n).unwrap_or_else(|| {
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
    pub fn own_car(&self, net_race: Option<NetRace>) -> Ref<'_, Car> {
        let n = self.own_car_number(net_race);
        Ref::map(self.car_blocks(), |b| &b.cars[n])
    }

    /// `own_car` to write.
    pub fn own_car_mut(&self, net_race: Option<NetRace>) -> RefMut<'_, Car> {
        let n = self.own_car_number(net_race);
        RefMut::map(self.car_blocks_mut(), |b| &mut b.cars[n])
    }
}
