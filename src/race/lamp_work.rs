//! A view's own lamps and flares: the sixteen lamps the view gathers, the flares' six tables, the
//! counts and the lamps wanted. One per view slot (`race::view_scratch::ViewScratch::lamps`). A
//! view keeps its own lamps from frame to frame: the lamps it starts a drawing from are the ones
//! its own last drawing left (`ViewScratch::begin` leaves them alone), not the previous view's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct LampWork {
    /// 16 lamps of 0x10 bytes (+0 the point, +4 the strength, +8 the colour).
    pub lamps: [PlainDword; 64],
    /// The flares' places' x.
    pub flare_place_x: [PlainDword; 256],
    /// The flares' y, z, colour, size and object number (`race::flares_store::ViewFlares`).
    pub flares: crate::race::flares_store::ViewFlares,
    /// The lamps wanted.
    pub lamps_wanted: PlainDword,
    /// The lamps gathered.
    pub lamp_count: PlainDword,
    /// The flares gathered.
    pub flare_count: PlainDword,
}

impl LampWork {
    /// A view's lamps at the start: none gathered or wanted, no flares.
    pub const fn new() -> Self {
        Self {
            lamps: ZeroCells::ZERO,
            flare_place_x: ZeroCells::ZERO,
            flares: crate::race::flares_store::ViewFlares::new(),
            lamps_wanted: PlainDword::ZERO,
            lamp_count: PlainDword::ZERO,
            flare_count: PlainDword::ZERO,
        }
    }
}
