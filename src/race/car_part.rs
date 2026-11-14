//! The view's own car drawn as a piece of its own beside the rest of its view (frame plan,
//! after F7: the player's car was over a third of the handheld's game thread while two cores
//! stood idle). The car's drawing reads what the view's set-up has made by the time its lamps
//! are gathered (its camera, matrices, fog, light, lamps and the cars' state bits), so from
//! there it is drawn on a drawing thread (`workers::draw_aside`) on a copy of the view's state
//! while the view's thread draws the course's faces, the race's slots, the shadows, the skid
//! marks, the cars' lights and the depth entries before the car. At the car's depth entry the
//! view's thread takes the car's drawing (`CarAside::draw`): its noted drawing put there, its
//! render states and texture into the part's, the point records it wrote into the part's,
//! what it changed in its copy of the view put into the view; then the entries after it as
//! before. So the noted drawing is in the same order as one thread's, and the view's state
//! after the car as one thread leaves it.
//!
//! The one thing the car reads that the drawing before its entry changes is the texture
//! filter's three switches (`polygon.smooth`, `plain_first`, `plain_second`): each course face
//! drawn of kind 0xFF or below 0x10 sets them to 0 after it (`race::draw_list_entry`), and every
//! polygon drawn after sets `smooth` from a plain one. So the car is drawn aside, with the
//! three at 0, only when the view's sort list (drawn before every depth entry) holds such a
//! face (`filter_cleared`); otherwise it is drawn in the list as before.
//!
//! `RC2K_CAR_PART=0` (`workers::car_part`) draws the car in the list as before.

use crate::errors::Refused;
use crate::race::view_scratch::{DrawCtx, ViewScratch};
use crate::session::GameThread;
use std::sync::Mutex;

/// The copy of the view the car is drawn on: the thread drawing the car holds the lock for its
/// run (`workers::draw_aside`), the view's thread to copy into it and merge from it.
static CAR_VIEW: Mutex<ViewScratch> = Mutex::new(ViewScratch::new(0));

/// The view as it was when the car's copy was made: what the copy changed since is merged.
static TAKEN: GameThread<ViewScratch> = GameThread::new(ViewScratch::new(0));

/// The number of the view's own car (the car `race::draw_car_entry` takes) when its depth
/// entry will be in the view's list: added by `race::car_lights` with the scene set up and the
/// car's lights' bit, which the view's cars' state already holds (`race::car_light_modes`).
pub fn own_car(ctx: &DrawCtx<'_>) -> Option<u32> {
    use rc2k_core::race::lights::lights_on;
    if ctx.reads.switches.scene_set_up.get() == 0 {
        return None;
    }
    let n = ctx
        .reads
        .view
        .drawn_view_car(ctx.view.drawn_view(ctx.race))
        .ok()?;
    if n >= ctx.reads.view.race_cars.get() || !ctx.reads.work.has_car_block(n as usize) {
        return None;
    }
    let block = ctx.reads.work.block_number(n as usize).ok()?;
    lights_on(ctx.view.cars.state_of(block).ok()?.get()).then_some(n)
}

/// Whether the view's sort list (`lists.sort_entries`, its first `count`) holds a course face
/// `race::draw_list_entry` draws with the texture filter's switches set to 0 after it (kind
/// 0xFF or below 0x10): the car's entry, after the whole sort list, then finds them at 0. A
/// face whose kind cannot be read answers no (the list's drawing refuses it, as before).
pub fn filter_cleared(
    ctx: &DrawCtx<'_>,
    entries: &[crate::session::PlainDword],
    count: u32,
) -> bool {
    for entry in entries.as_chunks::<2>().0.iter().take(count as usize) {
        let word = entry[1].get() & 0x7FFF_FFFF;
        if word >> 24 != 0 {
            continue;
        }
        match crate::race::course_places::face_field::<u8>(ctx.reads.course(), word, 0) {
            Ok(kind) if kind == 0xFF || kind < 0x10 => return true,
            Ok(_) => {}
            Err(_) => return false,
        }
    }
    false
}

/// The car being drawn aside, for the view's depth list (`draw_entries_around`).
pub struct CarAside<'a, 'b> {
    aside: &'b crate::workers::Aside<'a>,
    number: u32,
}

/// Car `number` (`own_car`) drawn aside of the view `ctx` draws, from its state now with the
/// texture filter's switches at 0 (`filter_cleared`), while `rest` draws on; `rest` puts the
/// car's drawing at its entry (`CarAside`).
pub fn draw_with_car_aside<'a, R>(
    helper: crate::workers::Helper<'a>,
    ctx: &DrawCtx<'a>,
    number: u32,
    rest: impl FnOnce(&CarAside<'a, '_>) -> Result<R, Refused>,
) -> Result<R, Refused> {
    {
        let copy = CAR_VIEW
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        copy.copy_drawing_from(ctx.view);
        for cell in filter_switches(&copy) {
            cell.set(0);
        }
    }
    TAKEN.get().copy_drawing_from(ctx.view);
    let DrawCtx {
        race,
        scene,
        input,
        clock,
        far_clip,
        ..
    } = *ctx;
    let piece: crate::workers::Piece<'a> = Box::new(move |part, reads| {
        let view = CAR_VIEW
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ctx = DrawCtx {
            race,
            scene,
            input,
            view: &view,
            clock,
            far_clip,
            marks: &crate::scene::car_marks::NONE,
            part,
            reads,
        };
        draw_car(&ctx, number)
    });
    let point_count = ctx.reads.course_work.scene_point_count.get();
    crate::workers::draw_aside(helper, ctx.part, point_count, piece, |aside| {
        rest(&CarAside { aside, number })
    })
}

/// The texture filter's switches (`scene::choose_texture`).
fn filter_switches(view: &ViewScratch) -> [&crate::session::PlainDword; 3] {
    [
        &view.polygon.smooth,
        &view.polygon.plain_first,
        &view.polygon.plain_second,
    ]
}

/// The counts the car adds to (cars drawn, polygons drawn whole and split): its own added to
/// what the view counted meanwhile, where the other cells take the car's value.
fn counts(view: &ViewScratch) -> [&crate::session::PlainDword; 3] {
    [
        &view.cars.cars_drawn,
        &view.object.whole_polygon_count,
        &view.object.split_polygon_count,
    ]
}

/// The car's depth entry drawn (`race::draw_list_entry`'s car).
fn draw_car(ctx: &DrawCtx<'_>, number: u32) -> Result<(), Refused> {
    crate::race::car_draw::draw_car_entry(ctx, number)?;
    ctx.view.light.view_kind.set(0u32);
    Ok(())
}

impl CarAside<'_, '_> {
    /// The car's depth entry's second word (`race::add_car_depth_entry`), the top bit clear.
    pub fn entry_word(&self) -> u32 {
        self.number + 0x200_0000
    }

    /// The car's drawing taken at its entry in the view `ctx` draws (see the header).
    pub fn draw(&self, ctx: &DrawCtx<'_>) -> Result<(), Refused> {
        let run = self.aside.join()?;
        let copy = CAR_VIEW
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let taken = TAKEN.get();
        let counted: [u32; 3] = std::array::from_fn(|k| {
            counts(ctx.view)[k]
                .get()
                .wrapping_add(counts(&copy)[k].get().wrapping_sub(counts(taken)[k].get()))
        });
        ctx.view.merge_drawing(&copy, taken);
        for (cell, n) in counts(ctx.view).into_iter().zip(counted) {
            cell.set(n);
        }
        drop(copy);
        self.aside.put(run)
    }
}
