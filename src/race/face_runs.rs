//! The course's sorted faces (a view's sort list) drawn in runs over the idle drawing threads
//! (`scene::workers::draw_list_in_runs`): on the handheld consoles the faces are the largest
//! piece of a view's drawing after its car, and cores stand idle while one thread draws them.
//!
//! Why a run can start from the view as the list began instead of from what the face before
//! it left: a face's drawing reads the view's set-up (camera, matrices, fog, light, lamps,
//! face tables), which no face changes, and work values it writes itself before reading them
//! (the face work, the object's placing and light, the polygon work, the texture filter's
//! switches set to 1 before each face). The point records are a cache of projections that
//! depend on the point and the view alone, so a record made again in another run holds the
//! same values. The render states and the texture are put together in the runs' order
//! (`d3d::run_states`). A described face whose shape moves draws a random number and keeps
//! its own state in its own face: the numbers wait for the earlier runs
//! (`memory::draw_order`), and no two runs draw the same face.
//!
//! What the runs leave in the view is put back in their order (`ViewScratch::merge_drawing`:
//! every cell a run changed from its value as the list began; the polygon counts added up).
//! A cell a later run set back to the value it had as the list began keeps an earlier run's
//! value: these are the work values above, which the drawing after the list sets before it
//! reads them.
//!
//! The check of all this is `RC2K_FACE_RUNS=4096` with `RC2K_DRAW_SERIAL` (every face a run
//! of its own, from the view as the list began, on a scratch another face used): the held
//! races' fan hashes are those of `RC2K_FACE_RUNS=0`.

use crate::errors::{BadInput, Refused};
use crate::race::view_scratch::{DrawCtx, ViewScratch};

/// The view's sort list drawn (`race::draw_entries` on it): in runs when `helper` has drawing
/// threads to spare (`car_aside`: one of them is drawing the view's car) and every entry is a
/// course face, else on the view's thread alone. Refused (before anything): an entry of
/// 0x4000000 or more, as `draw_entries` refuses it.
pub(super) fn draw_sort_list<'a>(
    ctx: &DrawCtx<'a>,
    helper: Option<crate::scene::workers::Helper<'a>>,
    car_aside: bool,
) -> Result<(), Refused> {
    let lists = &ctx.view.lists;
    let alone = || {
        crate::race::frame_draw::draw_entries(ctx, &lists.sort_entry_count, |n| lists.sort_from(n))
    };
    let Some(helper) = helper else {
        return alone();
    };
    let count = lists.sort_entry_count.get() as usize;
    let runs = crate::scene::workers::list_runs_wanted(Some(helper), usize::from(car_aside), count);
    if runs < 2 {
        return alone();
    }
    let entries = lists
        .sort_entries()
        .get(..2 * count)
        .ok_or(BadInput("a draw list entry past the list"))?;
    // The faces, read once: the runs' threads do not reach the view's list.
    let faces: Vec<u32> = entries
        .as_chunks::<2>()
        .0
        .iter()
        .map(|entry| entry[1].get())
        .collect();
    if faces.iter().any(|word| word & 0x7FFF_FFFF >= 0x100_0000) {
        // Not only faces (no such list is made today): entry by entry, as before.
        return alone();
    }
    let DrawCtx {
        race,
        scene,
        input,
        clock,
        far_clip,
        ..
    } = *ctx;
    let faces = &faces;
    let draw = move |part: &crate::scene::scratch_store::PartScratch,
                     view: &ViewScratch,
                     reads: crate::scene::workers::DrawReads<'_>,
                     entries: std::ops::Range<usize>| {
        let ctx = DrawCtx {
            race,
            scene,
            input,
            view,
            clock,
            far_clip,
            marks: &crate::scene::car_marks::NONE,
            part,
            reads,
        };
        crate::race::frame_draw::draw_face_entries(&ctx, &faces[entries])
    };
    let view = ctx.view;
    let mut put_view = |copy: &ViewScratch, start: &ViewScratch| {
        use crate::race::car_part::counts;
        let counted: [u32; 3] = std::array::from_fn(|k| {
            counts(view)[k]
                .get()
                .wrapping_add(counts(copy)[k].get().wrapping_sub(counts(start)[k].get()))
        });
        view.merge_drawing(copy, start);
        for (cell, n) in counts(view).into_iter().zip(counted) {
            cell.set(n);
        }
    };
    crate::scene::workers::draw_list_in_runs(
        helper,
        ctx.part,
        view,
        ctx.reads.course_work.get().scene_point_count,
        count,
        runs,
        &draw,
        &mut put_view,
    )
}
