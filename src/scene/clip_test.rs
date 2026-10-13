//! Polygons for the clipping hooks' self-tests: records and a pool of the test's, the
//! entries in the game's table.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::framework::{game_addr, game_read, game_write};
use crate::scene::clip::{NEXT_ENTRY_RVA, NEXT_RECORD_RVA, regions};
use crate::scene::polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, LIST_RVA, put};
use crate::scene::project_point::{
    CENTRE_X_RVA, CENTRE_Y_RVA, FLAT_DEPTH_RVA, FLAT_RVA, SCALE_RVA,
};
use crate::service::harness::{Region, at, capture, restore};

/// A vertex: screen x and y, depth.
pub type Corner = (f32, f32, f32);

pub struct Polygons {
    records: Vec<[u32; 8]>,
    pool: Vec<[u32; 8]>,
    saved: (Vec<Region>, Vec<u8>),
}

impl Polygons {
    /// `more`: what the test sets besides, to be put back at the end.
    pub fn begin(more: &[Region]) -> Self {
        let mut all = regions();
        // The pool of the moment is not the test's to save.
        if unsafe { game_read::<u32>(NEXT_RECORD_RVA) } != 0 {
            all.pop();
        }
        all.extend_from_slice(more);
        all.extend(
            [
                FLAT_RVA,
                FLAT_DEPTH_RVA,
                SCALE_RVA,
                CENTRE_X_RVA,
                CENTRE_Y_RVA,
            ]
            .map(at),
        );
        let saved = capture(&all);
        Self {
            records: vec![[0; 8]; 8],
            pool: vec![[0; 8]; ENTRIES as usize + 1],
            saved: (all, saved),
        }
    }

    /// Puts a polygon in place, everything it may write blotted out first. `variant`
    /// turns the entries' numbers and values about.
    pub unsafe fn set(&mut self, corners: &[Corner], variant: usize, flat: bool) {
        let f = |value: f32| value.to_bits();
        unsafe {
            game_write(NEXT_RECORD_RVA, 0u32);
            let written = regions();
            restore(&written, &vec![0xAA; written.iter().map(|r| r.1).sum()]);
            self.pool.fill([0xAAAA_AAAA; 8]);
            for (n, &(x, y, depth)) in corners.iter().enumerate() {
                let k = (n + variant) as f32;
                self.records[n] = [
                    n as u32 * 1000 + 7,
                    (variant as u32 + 1).wrapping_mul(0x00FF_F123),
                    0u32.wrapping_sub(n as u32 * 77_777),
                    f(x),
                    f((x - 400.0) * depth / 640.0),
                    f((300.0 - y) * depth / 640.0),
                    f(depth),
                    f(y),
                ];
                // The list picks the entries out of order.
                let number = [5u32, 2, 9, 0, 0x30, 3, 7, 1][(n + variant) % 8];
                game_write(LIST_RVA + n * 4, number);
                let entry = game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN;
                put(entry, n as u32);
                put(entry + 4, self.records[n].as_ptr() as u32);
                for (j, value) in [0.125f32, 0.875, 0.3, 0.7].into_iter().enumerate() {
                    put(entry + 8 + j * 4, f(value * (k + 1.0)));
                }
                put(
                    entry + 0x18,
                    0x1234_5678u32.rotate_left(n as u32 * 8 + variant as u32),
                );
                ((entry + 0x20) as *mut [u8; 2]).write([(n * 60 + variant) as u8, 255 - n as u8]);
            }
            game_write(COUNT_RVA, corners.len() as u32);
            game_write(NEXT_ENTRY_RVA, 0x40u32 + variant as u32);
            game_write(NEXT_RECORD_RVA, self.pool.as_mut_ptr() as u32);
            game_write(FLAT_RVA, flat as u32);
            game_write(FLAT_DEPTH_RVA, 6999u32);
            game_write(SCALE_RVA, f(640.0));
            game_write(CENTRE_X_RVA, f(400.0));
            game_write(CENTRE_Y_RVA, f(300.0));
        }
    }

    /// Everything a clipping left, the pool with it. The pool's address in the
    /// variable is the same on both sides.
    pub fn state(&self) -> (Vec<u8>, Vec<[u32; 8]>) {
        let mut all = regions();
        all.pop();
        (capture(&all), self.pool.clone())
    }

    pub fn end(self) {
        restore(&self.saved.0, &self.saved.1);
    }
}

/// Polygons that lie about a screen of 800 x 600 and depths of 4096 .. 60000 in every
/// way: inside, outside, across one edge, across corners, a vertex on an edge.
pub const POLYGONS: [&[Corner]; 12] = [
    &[
        (100.0, 100.0, 8000.0),
        (700.0, 120.0, 9000.0),
        (400.0, 500.0, 10000.0),
    ],
    &[
        (-50.0, 100.0, 8000.0),
        (700.0, 120.0, 9000.0),
        (400.0, 500.0, 10000.0),
    ],
    &[
        (100.0, -80.5, 8000.0),
        (900.25, 120.0, 9000.0),
        (400.0, 700.0, 5000.0),
    ],
    &[
        (-100.0, -100.0, 5000.0),
        (900.0, -100.0, 6000.0),
        (900.0, 700.0, 7000.0),
        (-100.0, 700.0, 8000.0),
    ],
    &[
        (-300.0, 100.0, 8000.0),
        (-200.0, 120.0, 9000.0),
        (-100.0, 500.0, 10000.0),
    ],
    &[
        (0.0, 0.0, 4096.0),
        (800.0, 0.0, 4096.0),
        (800.0, 600.0, 60000.0),
        (0.0, 600.0, 60000.0),
    ],
    &[
        (100.0, 100.0, 100.0),
        (700.0, 120.0, 9000.0),
        (400.0, 500.0, 70000.0),
    ],
    &[
        (100.0, 100.0, 100.0),
        (700.0, 120.0, 200.0),
        (400.0, 500.0, 300.0),
    ],
    &[
        (10.5, 20.25, 3000.0),
        (790.0, 30.0, 5000.0),
        (780.0, 590.0, 99999.0),
        (400.0, 650.0, 4000.0),
        (-20.0, 300.0, 4096.5),
    ],
    &[
        (400.0, 300.0, 8000.0),
        (400.0, 300.0, 8000.0),
        (1200.0, 300.0, 2000.0),
    ],
    &[
        (f32::NAN, 100.0, 8000.0),
        (700.0, 120.0, 9000.0),
        (400.0, 900.0, 10000.0),
    ],
    &[(850.0, 650.0, 90000.0), (700.0, 120.0, 9000.0)],
];
