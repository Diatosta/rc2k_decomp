//! The service area's inspection status: 100 less the weights of the damaged parts not chosen
//! for repair, as a bar between "fail" and "pass" with a sprite at its left.

use crate::screens::screen::{Bar, Drawn, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Labels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// The status.
    Status,
}

/// A part: the car's damage (+0xCC + 4 x item), its threshold
/// (`MenuState.driver_stats.highs[12][item]`), its weight (`weights[12][item]`) and whether it is
/// chosen for repair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    pub damage: i32,
    pub threshold: i32,
    pub weight: i32,
    pub repairing: bool,
}

/// The menu items whose parts count (the repair section's fourteen without the turbo, item 6).
pub const PARTS: [u32; 13] = [2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15];

/// The status and the index (into `PARTS`) of the last damaged part looked at.
pub fn status(parts: &[Part; 13]) -> (i32, Option<usize>) {
    let mut lost = 0i32;
    let mut last = None;
    for (n, p) in parts.iter().enumerate() {
        if p.damage < p.threshold {
            continue;
        }
        last = Some(n);
        if !p.repairing {
            lost = lost.wrapping_add(p.weight);
        }
    }
    (100i32.wrapping_sub(lost), last)
}

/// The status drawn. What the last text left.
pub fn inspection_status<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    status: i32,
) -> Result<Drawn, S::Error> {
    s.keep(Work::Status, status as u32);
    s.bar(Bar {
        x: 0x19D,
        y: 0x229,
        width: 0xD0,
        fill_glyph: 0x89,
        end_glyph: 0x87,
        percent: status,
    })?;
    s.sprite(0x88, 0x161, 0x228)?;
    s.string(Font::Labels, 0x4AE, 0x17F, 0x22C)?;
    s.string(Font::Labels, 0x4AF, 0x28C, 0x22C)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights() {
        let part = |damage, repairing| Part {
            damage,
            threshold: 5,
            weight: 10,
            repairing,
        };
        let mut parts = [part(0, false); 13];
        assert_eq!(status(&parts), (100, None));
        parts[2] = part(5, false);
        parts[7] = part(9, true);
        assert_eq!(status(&parts), (90, Some(7)));
    }
}
