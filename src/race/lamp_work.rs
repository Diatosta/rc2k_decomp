//! A view's own lamps and flares: the sixteen lamps the view gathers, the flares' six tables, the
//! counts and the lamps wanted. One per view slot (`race::view_scratch::ViewScratch::lamps`). A
//! view keeps its own lamps from frame to frame: the lamps it starts a drawing from are the ones
//! its own last drawing left (`ViewScratch::begin` leaves them alone), not the previous view's.

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// How many lamps and how many flares a view holds.
pub const LAMPS: usize = 16;
pub const FLARES: usize = 256;

/// One flare as the tables hold it.
#[derive(Clone, Copy)]
pub struct Flare {
    pub place: [u32; 3],
    pub size: u32,
    pub other_light: u32,
    pub object_number: u32,
}

/// The values of a view's lamps and flares, plain.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LampValues {
    /// 16 lamps of four words (the point, the strength, the colour; the fourth is never used).
    pub lamps: [[u32; 4]; LAMPS],
    /// The flares' places (x, y, z).
    pub flare_place_x: [u32; FLARES],
    pub flare_place_y: [u32; FLARES],
    pub flare_place_z: [u32; FLARES],
    /// The flares' other light (colour), size and object number.
    pub flare_other_light: [u32; FLARES],
    pub flare_size: [u32; FLARES],
    pub flare_object_number: [u32; FLARES],
    /// The lamps wanted.
    pub lamps_wanted: u32,
    /// The lamps gathered.
    pub lamp_count: u32,
    /// The flares gathered.
    pub flare_count: u32,
}

impl LampValues {
    /// A view's lamps at the start: none gathered or wanted, no flares.
    pub const fn new() -> Self {
        Self {
            lamps: [[0; 4]; LAMPS],
            flare_place_x: [0; FLARES],
            flare_place_y: [0; FLARES],
            flare_place_z: [0; FLARES],
            flare_other_light: [0; FLARES],
            flare_size: [0; FLARES],
            flare_object_number: [0; FLARES],
            lamps_wanted: 0,
            lamp_count: 0,
            flare_count: 0,
        }
    }

    /// Flare `n`; one past the 256 is `None`.
    pub fn flare(&self, n: u32) -> Option<Flare> {
        let n = n as usize;
        Some(Flare {
            place: [
                *self.flare_place_x.get(n)?,
                *self.flare_place_y.get(n)?,
                *self.flare_place_z.get(n)?,
            ],
            size: *self.flare_size.get(n)?,
            other_light: *self.flare_other_light.get(n)?,
            object_number: *self.flare_object_number.get(n)?,
        })
    }

    /// Flare `n` set to `flare`; the caller keeps `n` below 256.
    pub fn set_flare(&mut self, n: usize, flare: Flare) {
        self.flare_object_number[n] = flare.object_number;
        self.flare_place_x[n] = flare.place[0];
        self.flare_place_y[n] = flare.place[1];
        self.flare_place_z[n] = flare.place[2];
        self.flare_size[n] = flare.size;
        self.flare_other_light[n] = flare.other_light;
    }
}

impl Merge for LampValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.lamps, &now.lamps, &start.lamps);
        merge_run(
            &mut self.flare_place_x,
            &now.flare_place_x,
            &start.flare_place_x,
        );
        merge_run(
            &mut self.flare_place_y,
            &now.flare_place_y,
            &start.flare_place_y,
        );
        merge_run(
            &mut self.flare_place_z,
            &now.flare_place_z,
            &start.flare_place_z,
        );
        merge_run(
            &mut self.flare_other_light,
            &now.flare_other_light,
            &start.flare_other_light,
        );
        merge_run(&mut self.flare_size, &now.flare_size, &start.flare_size);
        merge_run(
            &mut self.flare_object_number,
            &now.flare_object_number,
            &start.flare_object_number,
        );
        merge_value(
            &mut self.lamps_wanted,
            &now.lamps_wanted,
            &start.lamps_wanted,
        );
        merge_value(&mut self.lamp_count, &now.lamp_count, &start.lamp_count);
        merge_value(&mut self.flare_count, &now.flare_count, &start.flare_count);
    }
}

/// A view's lamps and flares.
pub type LampWork = PlainSlot<LampValues>;
