//! The car's condition in the service area (converted 2026-10-21 from
//! `service::repair_body` .. `repair_suspension`, `reset_tyre_wear` 0x486EDD and
//! `repair_car` 0x486F63): each part has a run of the player's wear dwords and a damage
//! dword; a repair zeroes both. The service area's picture for a car converted the same
//! day from `service::load_background` 0x470F55 (`background`).

use crate::Invalid;
use crate::menu::player::Player;

/// A part the service area repairs whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Body,
    Gearbox,
    Brakes,
    Lights,
    Suspension,
}

impl Part {
    /// Its run of `parts_wear` (first, dwords) and its `damage` dword.
    fn places(self) -> (usize, usize, usize) {
        match self {
            Part::Body => (10, 6, 1),       // +0x1A4, +0xD8
            Part::Gearbox => (0, 10, 12),   // +0x17C, +0x104
            Part::Brakes => (28, 3, 11),    // +0x1EC, +0x100
            Part::Lights => (16, 7, 5),     // +0x1BC, +0xE8
            Part::Suspension => (23, 5, 9), // +0x1D8, +0xF8
        }
    }
}

/// The tyre damage dword (+0x108).
const TYRE_DAMAGE: usize = 13;
/// The damage dwords `repair_car` zeroes itself (+0xD4 .. +0xFC).
const CAR_DAMAGE: [usize; 8] = [0, 2, 3, 4, 6, 7, 8, 10];

impl Player {
    /// The part's run of wear dwords zeroed (`service::apply_repairs` before the work is
    /// shown).
    pub fn clear_wear(&mut self, part: Part) {
        let (first, dwords, _) = part.places();
        self.parts_wear[first..first + dwords].fill(0);
    }

    /// `service::repair_body` and the four others.
    pub fn repair(&mut self, part: Part) {
        let (first, dwords, damage) = part.places();
        self.parts_wear[first..first + dwords].fill(0);
        self.damage[damage] = 0;
    }

    /// `service::reset_tyre_wear`: the five wear dwords (+0x1F8) and the tyre damage.
    pub fn reset_tyre_wear(&mut self) {
        self.tyre_wear = [0; 5];
        self.damage[TYRE_DAMAGE] = 0;
    }

    /// `service::repair_car`: the car as new: eight damage dwords, every part, the tyres,
    /// `state_flags` -1 and `field_24c` 0.
    pub fn repair_car(&mut self) {
        for d in CAR_DAMAGE {
            self.damage[d] = 0;
        }
        self.repair(Part::Body);
        self.reset_tyre_wear();
        for part in [Part::Gearbox, Part::Brakes, Part::Lights, Part::Suspension] {
            self.repair(part);
        }
        self.state_flags = u32::MAX;
        self.field_24c = [0; 8];
    }
}

/// The service area's pictures (`files\g\graphics\screens\servNN.png`).
pub const PICTURES: u32 = 60;

/// The picture of the service area for a car: car 0x21 has car 0x17's. Refuses a car
/// past the pictures' table.
pub fn background(car: u32) -> Result<u32, Invalid> {
    let car = if car == 0x21 { 0x17 } else { car };
    if car >= PICTURES {
        return Err(Invalid("a car past the service pictures' table"));
    }
    Ok(car)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dword(p: &Player, at: usize) -> u32 {
        let bytes: &[u8; std::mem::size_of::<Player>()] = unsafe { std::mem::transmute(p) };
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    fn filled() -> Player {
        unsafe { std::mem::transmute([0x11u8; std::mem::size_of::<Player>()]) }
    }

    #[test]
    fn the_original_places() {
        let mut p = filled();
        p.repair(Part::Body);
        assert_eq!(dword(&p, 0x1A0), 0x1111_1111);
        assert_eq!(dword(&p, 0x1A4), 0);
        assert_eq!(dword(&p, 0x1B8), 0);
        assert_eq!(dword(&p, 0x1BC), 0x1111_1111);
        assert_eq!(dword(&p, 0xD8), 0);
        let mut p = filled();
        p.repair(Part::Brakes);
        assert_eq!(
            (dword(&p, 0x1EC), dword(&p, 0x1F4), dword(&p, 0x1F8)),
            (0, 0, 0x1111_1111)
        );
        assert_eq!(dword(&p, 0x100), 0);
        let mut p = filled();
        p.reset_tyre_wear();
        assert_eq!(
            (dword(&p, 0x1F8), dword(&p, 0x208), dword(&p, 0x20C)),
            (0, 0, 0x1111_1111)
        );
        assert_eq!(dword(&p, 0x108), 0);
        let mut p = filled();
        p.repair_car();
        for at in [
            0xD4, 0xDC, 0xE0, 0xE4, 0xEC, 0xF0, 0xF4, 0xFC, 0x104, 0x17C, 0x268,
        ] {
            assert_eq!(dword(&p, at), 0, "{at:#x}");
        }
        for at in [0xD0, 0x10C, 0x1F4 + 0x18, 0x26C] {
            assert_eq!(dword(&p, at), 0x1111_1111, "{at:#x}");
        }
        assert_eq!(dword(&p, 0x278), u32::MAX);
    }

    #[test]
    fn pictures() {
        assert_eq!(background(0x21), Ok(0x17));
        assert_eq!(background(5), Ok(5));
        assert!(background(60).is_err());
    }
}
