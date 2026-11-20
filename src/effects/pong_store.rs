//! The pong game's state and constants.

use crate::memory::session::{PlainByte, PlainDword, PlainFloat};

/// The pong game over the race: the box scratch, the ball, the bat, the score.
#[derive(rc2k_derive::Cells)]
pub struct PongState {
    /// The box drawn last (`effects::draw_pong_box`): the offsets of its corners from the place,
    /// twice (a reading: six, then the same plus the second shift).
    pub box_sides: [PlainFloat; 12],
    /// The ball's place (x, y).
    pub ball: [PlainFloat; 2],
    /// The ball's way, added to its place at each step (starts 1, 1).
    pub ball_way: [PlainFloat; 2],
    /// The ball's size (width, height).
    pub ball_size: [PlainFloat; 2],
    /// The bat's place before the move (put back when the bat overlaps the ball).
    pub saved_bat: [PlainFloat; 2],
    /// The bat's place (x, y).
    pub bat: [PlainFloat; 2],
    /// The bat's size.
    pub bat_size: [PlainFloat; 2],
    /// The ball's centre, worked out by the court's steps (`effects::pong_ball_hits`).
    pub ball_centre: [PlainFloat; 2],
    /// The bat's centre (`effects::pong_ball_hits`).
    pub bat_centre: [PlainFloat; 2],
    /// The ball's centre less the bat's (`effects::pong_ball_hits`); it becomes the ball's way
    /// after a hit.
    pub difference: [PlainFloat; 2],
    /// The ball's steps a frame (5 at the start, 4 more with each hit).
    pub steps: PlainDword,
    /// The digits' 4 x 5 patterns, 10 of 0x14 bytes (a byte a cell, 0 for empty).
    pub digit_patterns: [[PlainByte; 20]; 10],
    /// The best score (hits).
    pub best: PlainDword,
    /// The hits of this game.
    pub hits: PlainDword,
    /// Where the next score digit is drawn (x, y).
    pub digit_at: [PlainDword; 2],
}

impl PongState {
    pub const fn new() -> Self {
        PongState {
            box_sides: [const { PlainFloat::new(0.0) }; 12],
            ball: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            ball_way: [PlainFloat::new(1.0), PlainFloat::new(1.0)],
            ball_size: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            saved_bat: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            bat: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            bat_size: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            ball_centre: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            bat_centre: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            difference: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            steps: PlainDword::new(0x0),
            digit_patterns: [
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
            ],
            best: PlainDword::new(0x0),
            hits: PlainDword::new(0x0),
            digit_at: [PlainDword::new(0x0), PlainDword::new(0x0)],
        }
    }
}

/// The pong game's constants.
#[derive(rc2k_derive::Cells)]
pub struct PongRules {
    /// The court at the start of a game (`Court::reset`), as bits: four (x, y) pairs, the ball, its
    /// size, the bat, its size.
    pub start: [PlainFloat; 8],
    /// The court's walls the ball turns at and the places it is put back at (a reading,
    /// `effects::move_pong_ball`).
    pub walls: [PlainFloat; 8],
    /// A half, four times: a place plus its size times it is the centre (`Court` centres).
    pub centre: [PlainFloat; 4],
    /// 1, what a hit changes the way by.
    pub nudge: PlainFloat,
    /// 4, 4: the way after a hit is the difference over them (`effects::pong_ball_hits`).
    pub way_scale: [PlainFloat; 2],
    /// 16 and -16: a score cell's step across and down (`effects::pong_cell`).
    pub cell_step: [PlainFloat; 2],
    /// 16, 16: a score cell's size.
    pub cell_size: [PlainFloat; 2],
    /// The texture places of a pong box (`effects::draw_pong_box`; 0.99 four times, as bits).
    pub box_places: [PlainFloat; 4],
    /// 16384: `effects::draw_pong_box` draws nothing when the first corner's third value (its
    /// depth) compares below it.
    pub near: PlainFloat,
    /// The texture places the pong frame's set-up uses (`effects::pong_setup`).
    pub setup_places: [PlainFloat; 4],
}

impl PongRules {
    pub const fn new() -> Self {
        PongRules {
            start: [
                PlainFloat::new(1024.0),
                PlainFloat::new(512.0),
                PlainFloat::new(64.0),
                PlainFloat::new(64.0),
                PlainFloat::new(1792.0),
                PlainFloat::new(256.0),
                PlainFloat::new(64.0),
                PlainFloat::new(256.0),
            ],
            walls: [
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(2048.0),
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(1024.0),
                PlainFloat::new(1024.0),
            ],
            centre: [
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
            ],
            nudge: PlainFloat::new(1.0),
            way_scale: [PlainFloat::new(4.0), PlainFloat::new(4.0)],
            cell_step: [PlainFloat::new(16.0), PlainFloat::new(-16.0)],
            cell_size: [PlainFloat::new(16.0), PlainFloat::new(16.0)],
            box_places: [
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
            ],
            near: PlainFloat::new(16384.0),
            setup_places: [
                PlainFloat::new(0.11328125),
                PlainFloat::new(0.03125),
                PlainFloat::new(0.24609375),
                PlainFloat::new(0.1640625),
            ],
        }
    }
}

/// The pong game's state and constants, one part of the effects' owner (`effects::owner::Effects`).
pub struct Pong {
    pub state: PongState,
    pub rules: PongRules,
}

impl Pong {
    pub const fn new() -> Self {
        Pong {
            state: PongState::new(),
            rules: PongRules::new(),
        }
    }
}
