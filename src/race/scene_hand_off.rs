//! What a race view's scene leaves its overlays in the shared stores (the frame census,
//! `RC2K_FRAME_CENSUS`: `viewN_scene` -> `viewN_rest`): the view window, the sun, the clock
//! and the scratch matrices' rows that `scene::multiply_matrices` never writes. Kept from the
//! scene's own copies at its end, and set again before its overlays when they run after
//! other views' parts (the players' views beside each other, `race::draw_race_scene`): the
//! merged stores hold what the last part left there, and a part that set a value back to
//! the frame's first is not merged at all.

use crate::race::draw_view_scene::ViewOutput;
use crate::race::owner::Race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SceneHandOff {
    /// `SceneDraw.view_centre`, `view_scale`, `farthest` (the view window).
    window: [Dword; 4],
    /// `FrameWork.sun_on_screen`, `view_out`.
    sun: [Dword; 2],
    /// `PhysicsTables.scene_clock` (restarted at the view's fog).
    clock: Dword,
    /// `SceneMatrices.lit_matrix`.
    lit: [Dword; 16],
    /// `SceneMatrices.made_object_matrix`.
    made: [Dword; 16],
    /// The view's output (`ViewOutput`), applied by the game's thread: the EAX weight and the
    /// block and value for `store_view_value`, each with a word saying it is there.
    environment: [Dword; 2],
    view_value: [Dword; 3],
}

impl SceneHandOff {
    pub const fn new() -> Self {
        Self {
            window: [const { Dword::new(0) }; 4],
            sun: [const { Dword::new(0) }; 2],
            clock: Dword::new(0),
            lit: [const { Dword::new(0) }; 16],
            made: [const { Dword::new(0) }; 16],
            environment: [const { Dword::new(0) }; 2],
            view_value: [const { Dword::new(0) }; 3],
        }
    }

    /// The view's output kept (on its part).
    pub fn keep_output(&self, output: ViewOutput) {
        let (there, weight) = output.environment.map_or((0, 0), |w| (1, w));
        self.environment[0].set(there);
        self.environment[1].set(weight);
        let (there, (block, value)) = output.view_value.map_or((0, (0, 0)), |v| (1, v));
        self.view_value[0].set(there);
        self.view_value[1].set(block);
        self.view_value[2].set(value);
    }

    /// The kept output.
    pub fn output(&self) -> ViewOutput {
        ViewOutput {
            environment: (self.environment[0].get() != 0).then(|| self.environment[1].get()),
            view_value: (self.view_value[0].get() != 0)
                .then(|| (self.view_value[1].get(), self.view_value[2].get())),
        }
    }

    /// The scene's values kept (at its end, on its part's copies).
    pub fn keep(&self, race: &Race, scene: &crate::scene::owner::Scene) {
        let s = &scene.drawing.scene;
        let window = [
            s.view_centre[0].bits(),
            s.view_centre[1].bits(),
            s.view_scale.bits(),
            s.farthest.bits(),
        ];
        for (cell, v) in self.window.iter().zip(window) {
            cell.set(v);
        }
        let work = &race.frame.frame_work;
        self.sun[0].set(work.sun_on_screen.get());
        self.sun[1].set(work.view_out.get());
        self.clock.set(race.shared.physics_tables.scene_clock.get());
        let m = &race.frame.matrices;
        for (kept, now) in [&self.lit, &self.made]
            .into_iter()
            .zip([&m.lit_matrix, &m.made_object_matrix])
        {
            for (cell, v) in kept.iter().zip(now) {
                cell.set(v.bits());
            }
        }
    }

    /// The kept values set again (before the view's overlays).
    pub fn put(&self, race: &Race, scene: &crate::scene::owner::Scene) {
        let s = &scene.drawing.scene;
        s.view_centre[0].set_bits(self.window[0].get());
        s.view_centre[1].set_bits(self.window[1].get());
        s.view_scale.set_bits(self.window[2].get());
        s.farthest.set_bits(self.window[3].get());
        let work = &race.frame.frame_work;
        work.sun_on_screen.set(self.sun[0].get());
        work.view_out.set(self.sun[1].get());
        race.shared.physics_tables.scene_clock.set(self.clock.get());
        let m = &race.frame.matrices;
        for (kept, now) in [&self.lit, &self.made]
            .into_iter()
            .zip([&m.lit_matrix, &m.made_object_matrix])
        {
            for (cell, v) in kept.iter().zip(now) {
                v.set_bits(cell.get());
            }
        }
    }
}
