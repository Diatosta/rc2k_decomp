//! The race owner's shared group (`race::owner::Race::shared`): the race's stores the menus, the network and the audio side read too.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::blocks_store::CarBlocks;
use crate::race::buffers_store::Buffers;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings};
use crate::race::replay::Replays;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::replay_path::Templates;
use crate::race::state_store::RaceState;
use crate::race::switches_store::FrameSwitches;
use crate::race::tables_store::RaceTables;
use crate::race::tyre_store::TyreChange;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::work_store::RaceWork;

/// The race's stores the menus, the network and the audio side read too.
pub struct RaceShared {
    /// `RACE_WORK` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub work: RaceWork,
    /// `RACE_STATE` (`race`).
    pub state: RaceState,
    /// `RACE_TABLES` (`race`).
    pub tables: RaceTables,
    /// `BUFFERS` (`race`).
    pub buffers: Buffers,
    /// `TYRE_CHANGE` (`race`).
    pub tyre_change: TyreChange,
    /// `RACE_VARS` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub vars: RaceVars,
    /// `RACE_VIEW` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub view: RaceView,
    /// `FRAME_SWITCHES` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub switches: FrameSwitches,
    /// `REPLAY_FILE` (`race`).
    pub replay_file: ReplayFileBlock,
    /// `TEMPLATES` (`race`).
    pub replay_templates: Templates,
    /// `REPLAYS` (`race`).
    pub replays: Replays,
    /// `PHYSICS` (`physics`).
    /// One copy: the drawing only reads it (frame plan F4).
    pub physics_tables: PhysicsTables,
    /// `CAR` (`physics`): the work car. One copy: the drawing no longer reads it (frame plan
    /// F3), so the drawing threads need none of their own.
    pub car: Car,
    /// The ten cars' blocks (`Race::replay`'s), for the code that holds only this group.
    pub car_blocks: &'static CarBlocks,
    /// The effects' settings with the cars' records (`Race::replay`'s), as `car_blocks`.
    pub effects_settings: &'static EffectsSettings,
}

impl RaceShared {
    /// The group, reaching the cars' blocks and the effects' settings at `car_blocks` and
    /// `effects_settings`.
    pub const fn new(
        car_blocks: &'static CarBlocks,
        effects_settings: &'static EffectsSettings,
    ) -> Self {
        RaceShared {
            work: RaceWork::new(),
            state: RaceState::new(),
            tables: RaceTables::new(),
            buffers: Buffers::new(),
            tyre_change: TyreChange::new(),
            vars: RaceVars::new(),
            view: RaceView::new(),
            switches: FrameSwitches::new(),
            replay_file: ReplayFileBlock::new(),
            replay_templates: Templates::new(),
            replays: Replays::new(),
            physics_tables: PhysicsTables::new(),
            car: Car::new(),
            car_blocks,
            effects_settings,
        }
    }

    /// Car `n`'s block (`g_carBlocks[n]`), refused for a value that names no block (see
    /// `RaceWork::car_block` for the read past the eight).
    pub fn race_car(&self, n: usize) -> Result<&'static Car, BadInput> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        if let Some(cell) = self.work.car_blocks.get(n) {
            let k = (cell.get() ^ BLOCK_TAG) as usize;
            if k < BLOCK_COUNT {
                return Ok(&self.car_blocks.cars[k]);
            }
        }
        self.car_blocks.named(self.work.car_block(n))
    }

    /// Car `n`'s block, or the refusal `what` for none (the original's read at a small offset
    /// from 0 fails). A handle that names no block ends the program, as `RaceWork::car_block`.
    pub fn race_car_or(&self, n: usize, what: &'static str) -> Result<&'static Car, BadInput> {
        match self.work.car_blocks.get(n) {
            Some(cell) if cell.get() == 0 => Err(BadInput(what)),
            Some(_) => self.race_car(n),
            None => self.car_blocks.at(self.work.car_block(n), what),
        }
    }

    /// Car `n`'s block; a null block or one that is none of the ten ends the program.
    pub fn race_car_or_end(&self, n: usize) -> &'static Car {
        let block = self.work.car_block(n);
        match self.work.car_blocks.get(n) {
            Some(_) if block != 0 => self.race_car(n),
            _ => self.car_blocks.named(block),
        }
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::Car::at", e))
    }

    /// Car record `n` (`g_carRecords[n]`, the name of one of the effects' car records).
    /// Refused past the eight (the original reads on into the next variables) and for a name
    /// that is not a record's start.
    pub fn car_record(&self, n: usize) -> Result<&'static CarEffects, BadInput> {
        self.work
            .car_record_number(n)
            .map(|k| &self.effects_settings.cars[k])
    }

    /// The eight car records.
    pub fn car_records(&self) -> Result<[&'static CarEffects; 8], BadInput> {
        let first = self.car_record(0)?;
        let mut out = [first; 8];
        for (n, record) in out.iter_mut().enumerate() {
            *record = self.car_record(n)?;
        }
        Ok(out)
    }

    /// View `n`'s car block (None for none).
    pub fn view_car(&self, n: usize) -> Option<&'static Car> {
        self.car_blocks.of(self.view.view_blocks[n].get())
    }

    /// View `n`'s car block, or the end of the program for none (as `Car::at` on its address).
    pub fn view_car_or_end(&self, n: usize) -> &'static Car {
        self.view_car(n).unwrap_or_else(|| {
            crate::fatal::raise::end(
                "physics::Car::at",
                BadInput("a car block at no car's place"),
            )
        })
    }

    /// The car this machine plays: the first, but in a network race (`session` 1) its own.
    /// The original has the first block's address wherever it means the player's car, so on
    /// a joined machine the cockpit's effects, the mud and the listener followed the host's
    /// car.
    pub fn own_car_number(&self, session: u32) -> usize {
        if session == 1 {
            let local = self.view.local_car.get() as usize;
            if local < 8 { local } else { 0 }
        } else {
            0
        }
    }

    /// `own_car_number`'s block.
    pub fn own_car(&self, session: u32) -> &'static Car {
        &self.car_blocks.cars[self.own_car_number(session)]
    }
}
