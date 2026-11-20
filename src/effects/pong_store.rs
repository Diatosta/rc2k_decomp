//! The pong game's state and constants.

/// The pong game over the race: the box scratch, the ball, the bat, the score.
pub struct PongState {
    /// The box drawn last (`effects::draw_pong_box`): the offsets of its corners from the place,
    /// twice (a reading: six, then the same plus the second shift).
    pub box_sides: [f32; 12],
    /// The ball's place (x, y).
    pub ball: [f32; 2],
    /// The ball's way, added to its place at each step (starts 1, 1).
    pub ball_way: [f32; 2],
    /// The ball's size (width, height).
    pub ball_size: [f32; 2],
    /// The bat's place before the move (put back when the bat overlaps the ball).
    pub saved_bat: [f32; 2],
    /// The bat's place (x, y).
    pub bat: [f32; 2],
    /// The bat's size.
    pub bat_size: [f32; 2],
    /// The ball's centre, worked out by the court's steps (`effects::pong_ball_hits`).
    pub ball_centre: [f32; 2],
    /// The bat's centre (`effects::pong_ball_hits`).
    pub bat_centre: [f32; 2],
    /// The ball's centre less the bat's (`effects::pong_ball_hits`); it becomes the ball's way
    /// after a hit.
    pub difference: [f32; 2],
    /// The ball's steps a frame (5 at the start, 4 more with each hit).
    pub steps: u32,
    /// The digits' 4 x 5 patterns, 10 of 0x14 bytes (a byte a cell, 0 for empty).
    pub digit_patterns: [[u8; 20]; 10],
    /// The best score (hits).
    pub best: u32,
    /// The hits of this game.
    pub hits: u32,
    /// Where the next score digit is drawn (x, y).
    pub digit_at: [u32; 2],
}

impl PongState {
    pub const fn new() -> Self {
        PongState {
            box_sides: [0.0; 12],
            ball: [0.0, 0.0],
            ball_way: [1.0, 1.0],
            ball_size: [0.0, 0.0],
            saved_bat: [0.0, 0.0],
            bat: [0.0, 0.0],
            bat_size: [0.0, 0.0],
            ball_centre: [0.0, 0.0],
            bat_centre: [0.0, 0.0],
            difference: [0.0, 0.0],
            steps: 0x0,
            digit_patterns: [
                [0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0],
                [0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0],
                [1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0],
                [1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0],
                [1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0],
                [1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0],
                [0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0],
                [1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0],
                [0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0],
                [0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0],
            ],
            best: 0x0,
            hits: 0x0,
            digit_at: [0x0, 0x0],
        }
    }
}

/// The pong game's constants.
pub struct PongRules {
    /// The court at the start of a game (`Court::reset`), as bits: four (x, y) pairs, the ball, its
    /// size, the bat, its size.
    pub start: [f32; 8],
    /// The court's walls the ball turns at and the places it is put back at (a reading,
    /// `effects::move_pong_ball`).
    pub walls: [f32; 8],
    /// A half, four times: a place plus its size times it is the centre (`Court` centres).
    pub centre: [f32; 4],
    /// 1, what a hit changes the way by.
    pub nudge: f32,
    /// 4, 4: the way after a hit is the difference over them (`effects::pong_ball_hits`).
    pub way_scale: [f32; 2],
    /// 16 and -16: a score cell's step across and down (`effects::pong_cell`).
    pub cell_step: [f32; 2],
    /// 16, 16: a score cell's size.
    pub cell_size: [f32; 2],
    /// The texture places of a pong box (`effects::draw_pong_box`; 0.99 four times, as bits).
    pub box_places: [f32; 4],
    /// 16384: `effects::draw_pong_box` draws nothing when the first corner's third value (its
    /// depth) compares below it.
    pub near: f32,
    /// The texture places the pong frame's set-up uses (`effects::pong_setup`).
    pub setup_places: [f32; 4],
}

impl PongRules {
    pub const fn new() -> Self {
        PongRules {
            start: [1024.0, 512.0, 64.0, 64.0, 1792.0, 256.0, 64.0, 256.0],
            walls: [128.0, 128.0, 2048.0, 128.0, 128.0, 128.0, 1024.0, 1024.0],
            centre: [0.5, 0.5, 0.5, 0.5],
            nudge: 1.0,
            way_scale: [4.0, 4.0],
            cell_step: [16.0, -16.0],
            cell_size: [16.0, 16.0],
            box_places: [0.99, 0.99, 0.99, 0.99],
            near: 16384.0,
            setup_places: [0.11328125, 0.03125, 0.24609375, 0.1640625],
        }
    }
}

/// The pong game's state and constants, one part of the effects' owner (`effects::owner::Effects`).
pub struct Pong {
    pub state: std::cell::RefCell<PongState>,
    pub rules: PongRules,
}

impl Pong {
    pub const fn new() -> Self {
        Pong {
            state: std::cell::RefCell::new(PongState::new()),
            rules: PongRules::new(),
        }
    }
}
