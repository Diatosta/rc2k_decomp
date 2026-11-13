//! What a race view's scene leaves its overlays in the shared stores (the frame census of the
//! audit build: `viewN_scene` -> `viewN_rest`): the
//! scratch matrices' rows that `scene::multiply_matrices` never writes. Kept from the
//! scene's own copies at its end, and set again before its overlays when they run after
//! other views' parts (the players' views beside each other, `race::draw_race_scene`): the
//! merged stores hold what the last part left there, and a part that set a value back to
//! the frame's first is not merged at all.

use crate::race::draw_view_scene::ViewOutput;
use crate::race::owner::Race;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SceneHandOff {
    /// 1 when the mirror was drawn this frame (its full window follows: `race::draw_race_scene`).
    mirror_drawn: PlainDword,
    /// `SceneMatrices.lit_matrix`.
    lit: [PlainFloat; 16],
    /// `SceneMatrices.made_object_matrix`.
    made: [PlainFloat; 16],
    /// The view's output (`ViewOutput`), applied by the game's thread: the EAX weight and the
    /// block and value for `store_view_value`, each with a word saying it is there.
    environment: [PlainDword; 2],
    view_value: [PlainDword; 3],
    /// Whether its camera set its view matrices.
    view_matrices: PlainDword,
}

impl SceneHandOff {
    pub const fn new() -> Self {
        Self {
            mirror_drawn: PlainDword::new(0),
            lit: [const { PlainFloat::new(0.0) }; 16],
            made: [const { PlainFloat::new(0.0) }; 16],
            environment: [const { PlainDword::new(0) }; 2],
            view_value: [const { PlainDword::new(0) }; 3],
            view_matrices: PlainDword::new(0),
        }
    }

    /// The view's output kept (on its part).
    pub fn keep_output(&self, output: ViewOutput) {
        let (there, weight) = output.environment.map_or((0, 0), |w| (1, w));
        self.environment[0].set(there);
        self.environment[1].set(weight);
        let (there, (block, value)) = output.view_value.map_or((0, (0, 0)), |v| (1, v));
        self.view_value[0].set(there);
        self.view_value[1].set(block as u32);
        self.view_value[2].set(value);
        self.view_matrices.set(u32::from(output.view_matrices));
    }

    /// The mirror's drawing kept (it was drawn, or not).
    pub fn keep_mirror_drawn(&self, drawn: bool) {
        self.mirror_drawn.set(u32::from(drawn));
    }

    /// Whether the mirror was drawn.
    pub fn mirror_drawn(&self) -> bool {
        self.mirror_drawn.get() != 0
    }

    /// The kept output.
    pub fn output(&self) -> ViewOutput {
        ViewOutput {
            environment: (self.environment[0].get() != 0).then(|| self.environment[1].get()),
            view_value: (self.view_value[0].get() != 0)
                .then(|| (self.view_value[1].get() as usize, self.view_value[2].get())),
            view_matrices: self.view_matrices.get() != 0,
        }
    }

    /// The scene's values kept (at its end; the matrices' fourth columns the part's own,
    /// `scene::view_matrices::current_matrix`).
    pub fn keep(
        &self,
        m: &crate::race::matrices_store::SceneMatrices,
        part: &crate::workers::PartScratch,
    ) {
        for (kept, n) in [(&self.lit, 1), (&self.made, 2)] {
            let now = crate::scene::view_matrices::current_matrix(m, part, n);
            for (cell, v) in kept.iter().zip(now) {
                cell.set(f32::from_bits(v));
            }
        }
    }

    /// The kept values set again (before the view's overlays).
    pub fn put(&self, race: &Race) {
        let m = &race.frame.matrices.get();
        for (kept, now) in [&self.lit, &self.made]
            .into_iter()
            .zip([&m.lit_matrix, &m.made_object_matrix])
        {
            for (cell, v) in kept.iter().zip(now) {
                v.set(cell.get());
            }
        }
    }
}
