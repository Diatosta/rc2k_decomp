//! What a race view's scene leaves its overlays in the shared stores (the frame census of the
//! audit build: `viewN_scene` -> `viewN_rest`): the
//! scratch matrices' rows that `scene::multiply_matrices` never writes. Kept from the
//! scene's own copies at its end, and set again before its overlays when they run after
//! other views' parts (the players' views beside each other, `race::draw_race_scene`): the
//! merged stores hold what the last part left there, and a part that set a value back to
//! the frame's first is not merged at all.

use crate::race::draw_view_scene::ViewOutput;
use crate::race::owner::Race;
use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// The kept values, plain (the matrices' floats as their bits).
#[derive(Clone, Copy)]
struct HandOffValues {
    /// 1 when the mirror was drawn this frame (its full window follows: `race::draw_race_scene`).
    mirror_drawn: u32,
    /// `SceneMatrices.lit_matrix`.
    lit: [u32; 16],
    /// `SceneMatrices.made_object_matrix`.
    made: [u32; 16],
    /// The view's output (`ViewOutput`), applied by the game's thread: the block and value for
    /// `store_view_value`, with a word saying it is there.
    view_value: [u32; 3],
    /// Whether its camera set its view matrices.
    view_matrices: u32,
}

impl Merge for HandOffValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(
            &mut self.mirror_drawn,
            &now.mirror_drawn,
            &start.mirror_drawn,
        );
        merge_run(&mut self.lit, &now.lit, &start.lit);
        merge_run(&mut self.made, &now.made, &start.made);
        merge_run(&mut self.view_value, &now.view_value, &start.view_value);
        merge_value(
            &mut self.view_matrices,
            &now.view_matrices,
            &start.view_matrices,
        );
    }
}

pub struct SceneHandOff(PlainSlot<HandOffValues>);

impl SceneHandOff {
    pub const fn new() -> Self {
        Self(PlainSlot::new(HandOffValues {
            mirror_drawn: 0,
            lit: [0; 16],
            made: [0; 16],
            view_value: [0; 3],
            view_matrices: 0,
        }))
    }

    /// Every value set from `other`'s.
    pub fn copy_from(&self, other: &SceneHandOff) {
        self.0.copy_from(&other.0);
    }

    /// What `now` changed since it was `start` put into this one.
    pub fn merge_changes(&self, now: &SceneHandOff, start: &SceneHandOff) {
        self.0.merge_changes(&now.0, &start.0);
    }

    /// The view's output kept (on its part).
    pub(super) fn keep_output(&self, output: ViewOutput) {
        let (there, (block, value)) = output.view_value.map_or((0, (0, 0)), |v| (1, v));
        let mut kept = self.0.borrow_mut();
        kept.view_value = [there, block as u32, value];
        kept.view_matrices = u32::from(output.view_matrices);
    }

    /// The mirror's drawing kept (it was drawn, or not).
    pub(super) fn keep_mirror_drawn(&self, drawn: bool) {
        self.0.borrow_mut().mirror_drawn = u32::from(drawn);
    }

    /// Whether the mirror was drawn.
    pub(super) fn mirror_drawn(&self) -> bool {
        self.0.borrow().mirror_drawn != 0
    }

    /// The kept output.
    pub fn output(&self) -> ViewOutput {
        let kept = self.0.borrow();
        ViewOutput {
            view_value: (kept.view_value[0] != 0)
                .then(|| (kept.view_value[1] as usize, kept.view_value[2])),
            view_matrices: kept.view_matrices != 0,
        }
    }

    /// The scene's values kept (at its end; the matrices' fourth columns the part's own,
    /// `scene::view_matrices::current_matrix`).
    pub fn keep(
        &self,
        m: crate::race::matrices_store::SceneMatricesRef<'_>,
        part: &crate::scene::scratch_store::PartScratch,
    ) {
        let m = m.get();
        let lit = crate::scene::view_matrices::current_matrix(&m, part, 1);
        let made = crate::scene::view_matrices::current_matrix(&m, part, 2);
        let mut kept = self.0.borrow_mut();
        kept.lit = lit;
        kept.made = made;
    }

    /// The kept values set again (before the view's overlays).
    pub fn put(&self, race: &Race) {
        let kept = *self.0.borrow();
        let mut m = race.frame.matrices.borrow_mut();
        m.lit_matrix = kept.lit.map(f32::from_bits);
        m.made_object_matrix = kept.made.map(f32::from_bits);
    }
}
