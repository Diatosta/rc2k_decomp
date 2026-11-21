//! The view's own car drawn as a piece of its own beside the rest of its view (frame plan,
//! after F7: the player's car was over a third of the handheld's game thread while two cores
//! stood idle). The car's drawing reads what the view's set-up has made by the time its lamps
//! are gathered (its camera, matrices, fog, light, lamps and the cars' state bits), so from
//! there it is drawn on a drawing thread (`scene::workers::draw_aside`) on a copy of the view's state
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
//! `RC2K_CAR_PART=0` (`scene::workers::car_part`) draws the car in the list as before.

use crate::errors::Refused;
use crate::race::view_scratch::{DrawCtx, ViewScratch};

/// The number of the view's own car (the car `race::draw_car_entry` takes) when its depth
/// entry will be in the view's list: added by `race::car_lights` with the scene set up and the
/// car's lights' bit, which the view's cars' state already holds (`race::car_light_modes`).
pub fn own_car(ctx: &DrawCtx<'_>) -> Option<u32> {
    use rc2k_core::race::lights::lights_on;
    if ctx.reads.switches.scene_set_up() == 0 {
        return None;
    }
    let n = ctx
        .reads
        .view
        .drawn_view_car(ctx.view.drawn_view(ctx.race))
        .ok()?;
    if n >= ctx.reads.view.race_cars() || !ctx.reads.work.has_car_block(n as usize) {
        return None;
    }
    let block = ctx.reads.work.block_number(n as usize).ok()?;
    lights_on(ctx.view.cars.borrow().state_of(block).ok()?).then_some(n)
}

/// Whether the view's sort list (`lists.sort_entries`, its first `count`) holds a course face
/// `race::draw_list_entry` draws with the texture filter's switches set to 0 after it (kind
/// 0xFF or below 0x10): the car's entry, after the whole sort list, then finds them at 0. A
/// face whose kind cannot be read answers no (the list's drawing refuses it, as before).
pub(super) fn filter_cleared(
    ctx: &DrawCtx<'_>,
    entries: &[crate::memory::session::PlainDword],
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
    aside: &'b crate::scene::workers::Aside<'a>,
    number: u32,
}

/// Car `number` (`own_car`) drawn aside of the view `ctx` draws, from its state now with the
/// texture filter's switches at 0 (`filter_cleared`), while `rest` draws on; `rest` puts the
/// car's drawing at its entry (`CarAside`).
pub(super) fn draw_with_car_aside<'a, R>(
    helper: crate::scene::workers::Helper<'a>,
    ctx: &DrawCtx<'a>,
    number: u32,
    rest: impl FnOnce(&CarAside<'a, '_>) -> Result<R, Refused>,
) -> Result<R, Refused> {
    {
        let copy = ctx.scene.scratch.car_view();
        copy.copy_drawing_from(ctx.view);
        let mut scalars = copy.polygon.scalars.borrow_mut();
        scalars.smooth = 0;
        scalars.plain_first = 0;
        scalars.plain_second = 0;
    }
    ctx.scene.scratch.taken().copy_drawing_from(ctx.view);
    let DrawCtx {
        race,
        scene,
        input,
        clock,
        far_clip,
        ..
    } = *ctx;
    let piece: crate::scene::workers::Piece<'a> = Box::new(move |part, reads| {
        let view = scene.scratch.car_view();
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
    let point_count = ctx.reads.course_work.get().scene_point_count;
    crate::scene::workers::draw_aside(helper, ctx.part, point_count, piece, |aside| {
        rest(&CarAside { aside, number })
    })
}

/// The count a piece of the view's drawing adds to (cars drawn): its own added to what the view
/// counted meanwhile, where the other cells take the piece's value.
pub(crate) fn counted(view: &ViewScratch) -> u32 {
    view.cars.borrow().cars_drawn
}

/// The count set (see `counted`).
pub(crate) fn set_counted(view: &ViewScratch, cars: u32) {
    view.cars.borrow_mut().cars_drawn = cars;
}

/// The car's depth entry drawn (`race::draw_list_entry`'s car).
fn draw_car(ctx: &DrawCtx<'_>, number: u32) -> Result<(), Refused> {
    crate::race::car_draw::draw_car_entry(ctx, number)?;
    ctx.view.light.borrow_mut().view_kind = 0u32;
    Ok(())
}

impl CarAside<'_, '_> {
    /// The car's depth entry's second word (`race::add_car_depth_entry`), the top bit clear.
    pub(super) fn entry_word(&self) -> u32 {
        self.number + 0x200_0000
    }

    /// The car's drawing taken at its entry in the view `ctx` draws (see the header).
    pub fn draw(&self, ctx: &DrawCtx<'_>) -> Result<(), Refused> {
        let run = self.aside.join()?;
        let copy = ctx.scene.scratch.car_view();
        let taken = ctx.scene.scratch.taken();
        let cars = counted(ctx.view).wrapping_add(counted(&copy).wrapping_sub(counted(taken)));
        ctx.view.merge_drawing(&copy, taken);
        set_counted(ctx.view, cars);
        drop(copy);
        self.aside.put(run)
    }
}
