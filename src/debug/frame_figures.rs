//! Figures of the last frame for the display and watch pages' rows whose counters nothing in the
//! shipped game wrote: worked out here from what the game already counts or keeps (taken
//! once a UI frame, `take`, and when the first view's scene is drawn, `note_first_view`), so
//! play pays nothing more for them.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The figures, as the rows read them (`row_cells::Raw`).
pub struct Figures {
    /// Triangle fans drawn in the last frame (`render::FANS`; every polygon drawn is one).
    pub polygons: AtomicU32,
    /// Ground searches run in the last frame (`timing::note_ground_search`).
    pub ground_searches: AtomicU32,
    /// Of those, the ones that found a face under the point.
    pub ground_found: AtomicU32,
    /// The first view's first flare as `race::add_flare` turns it by the view matrix: x and y
    /// as float bits, z rounded (the depth key's source); kept while the view has no flare.
    pub light_turned: [AtomicU32; 3],
}

pub(super) static FIGURES: Figures = Figures {
    polygons: AtomicU32::new(0),
    ground_searches: AtomicU32::new(0),
    ground_found: AtomicU32::new(0),
    light_turned: [const { AtomicU32::new(0) }; 3],
};

/// The running counts at the last `take`: fans, ground searches, those found.
static LAST: Mutex<Option<(u64, u32, u32)>> = Mutex::new(None);

/// The last frame's counts (the running counters' differences since the last UI frame).
pub fn take() {
    let fans = crate::shell::frame::frame_counters()[3];
    let (searches, found) = crate::debug::timing::ground_searches();
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((then_fans, then_searches, then_found)) = *last {
        FIGURES
            .polygons
            .store(fans.wrapping_sub(then_fans) as u32, Relaxed);
        FIGURES
            .ground_searches
            .store(searches.wrapping_sub(then_searches), Relaxed);
        FIGURES
            .ground_found
            .store(found.wrapping_sub(then_found), Relaxed);
    }
    *last = Some((fans, searches, found));
}

/// The first view's scene has been drawn (on the game's thread): its first flare turned by
/// its view matrix, as `race::add_flare` turned it (a pure reckoning: nothing is written).
pub(super) fn note_first_view(scene: &crate::scene::owner::Scene) {
    let view = scene.scratch.main_view(0);
    if view.lamps.flare_count.get() == 0 {
        return;
    }
    let place = [
        view.lamps.flare_place_x[0].get(),
        view.lamps.flares.place_y[0].get(),
        view.lamps.flares.place_z[0].get(),
    ];
    let turned = crate::math::x87::rotate_rounding_z(
        place.map(|v| v as i32),
        &view.matrices.borrow().view_matrix.map(f32::to_bits),
    );
    for (figure, value) in FIGURES.light_turned.iter().zip(turned) {
        figure.store(value, Relaxed);
    }
}
