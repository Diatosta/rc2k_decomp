//! A view's cars' drawing state (frame plan F4): what the drawing of the cars and of the
//! objects works out and reads again within the view, one in each view's slot
//! (`race::view_scratch`) and one for the front end.
//!
//! The original kept it in shared pieces for every view at once: `FrameWork` (the cars drawn,
//! the car's kept place and depth), `CameraWork` (the car's kept word), `RaceWork` (the car's
//! drawing word) and `FrameSwitches` (the object's material way, the fade words and the
//! number of the object being drawn). The old fields stay in the stores, unused.

use crate::session::Dword;

pub struct CarDrawState {
    /// The cars drawn this frame (`g_carsDrawn`).
    pub cars_drawn: Dword,
    /// The car's kept place (`dword_64159C` ..).
    pub car_kept: [Dword; 2],
    /// The car's depth (`dword_6415A4`, 0x50000 at first).
    pub car_depth: Dword,
    /// The car's kept word (0x2100000 while a car is drawn).
    pub car_draw_kept: Dword,
    /// The car's drawing word (0x60 plus its nearness size).
    pub car_draw_word: Dword,
    /// The material way of the object being drawn (`dword_6214DC`).
    pub other_way_kind: Dword,
    /// The other kind and the distance of the object's fade (`qword_6214E0`).
    pub fade_words: [Dword; 2],
    /// The object being drawn or set up (`g_objectNumber`).
    pub object_number: Dword,
}

impl CarDrawState {
    pub const fn new() -> Self {
        Self {
            cars_drawn: Dword::new(0),
            car_kept: [const { Dword::new(0) }; 2],
            car_depth: Dword::new(0),
            car_draw_kept: Dword::new(0),
            car_draw_word: Dword::new(0),
            other_way_kind: Dword::new(0),
            fade_words: [const { Dword::new(0) }; 2],
            object_number: Dword::new(0),
        }
    }
}
