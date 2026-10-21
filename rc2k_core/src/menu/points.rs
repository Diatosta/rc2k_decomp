//! The championship's points (converted 2026-10-21 from `menu::award_stage_points`
//! 0x4A47EA and `award_overall_points` 0x4A4933): handed out in the standings' order from
//! 0x14 down by one, kept or not, and each entry's total of its six sections.

/// What is handed out (`seconds`, 0x7559AC, in the game): from 0x14 down by one, not below
/// 0 (signed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Points(pub u32);

impl Default for Points {
    fn default() -> Self {
        Points(0x14)
    }
}

impl Points {
    /// The next entry's points.
    pub fn take(&mut self) -> u32 {
        let p = self.0;
        if (p as i32) > 0 {
            self.0 = p - 1;
        }
        p
    }
}

/// A stage's points as the entry and its driver or player keep them: both the points when
/// the owner's switch (a driver's +0x24, a player's +0xD0) is 1, else both 0.
pub fn kept(points: u32, keeps: bool) -> u32 {
    if keeps { points } else { 0 }
}

/// An entry's total of its six sections' points, the negative ones left out.
pub fn total(sections: [u32; 6]) -> u32 {
    sections
        .iter()
        .map(|&v| v as i32)
        .filter(|&v| v >= 0)
        .fold(0u32, |s, v| s.wrapping_add(v as u32))
}

/// `menu::find_section_player` (0x4A940E): the first of the four players whose points for
/// the section are 0x14 (a reading: the section's winner).
pub fn section_player(points: [u32; 4]) -> Option<u32> {
    points.iter().position(|&p| p == 0x14).map(|n| n as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points() {
        let mut p = Points::default();
        let first: Vec<u32> = (0..22).map(|_| p.take()).collect();
        assert_eq!(first[0], 0x14);
        assert_eq!(first[19], 1);
        assert_eq!(first[20..], [0, 0]);
        let mut odd = Points(u32::MAX);
        assert_eq!((odd.take(), odd.0), (u32::MAX, u32::MAX));
        assert_eq!((kept(7, true), kept(7, false)), (7, 0));
        assert_eq!(total([1, 2, u32::MAX, 3, 0, 4]), 10);
    }

    #[test]
    fn section_winner() {
        assert_eq!(section_player([3, 0x14, 0x14, 0]), Some(1));
        assert_eq!(section_player([3, 2, 1, 0]), None);
    }
}
