//! The championship finish routine's time and shares (from
//! `drive::finish_timing` 0x45948A, after its clock call): the time since the start in
//! seconds, eased to a stop between two times, and a share rising between two others.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use crate::physics::x87::{below, loaded};

/// The routine's figures (`Finish.limit_a` ..).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FinishFigures {
    /// The clock's units a second.
    pub units: f32,
    /// The easing's start and end.
    pub ease: [f32; 2],
    /// A half.
    pub half: f32,
    /// The share's start and end.
    pub share: [f32; 2],
    /// The share's switch.
    pub switch: bool,
}

/// What it stores.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FinishTiming {
    /// The time.
    pub time: f32,
    /// The eased time.
    pub eased: f32,
    /// The share.
    pub share: f32,
    /// `ViewerAndKinds.marks_settings.race_done` set to 1.
    pub flag: bool,
}

/// The time and shares from the clock less the start.
pub fn finish_timing(since: u32, f: &FinishFigures) -> FinishTiming {
    let d = |x: f32| x as f64;
    let t = (since as i32 as f64 / d(f.units)) as f32;
    let [a, b] = f.ease;
    let eased = if below(d(t), d(a)) {
        loaded(t)
    } else if below(d(t), d(b)) {
        let q = (d(t) - d(a)) / (d(b) - d(a));
        let fall = q * q * ((d(b) - d(a)) * d(f.half));
        (-fall + d(t)) as f32
    } else {
        ((d(b) - d(a)) * d(f.half) + d(a)) as f32
    };
    let [c, e] = f.share;
    let late = !below(d(t), d(c));
    let mut share = 0.0;
    if f.switch && late {
        share = 1.0;
        if below(d(t), d(e)) {
            share = ((d(t) - d(c)) / (d(e) - d(c))) as f32;
        }
    }
    FinishTiming {
        time: t,
        eased,
        share,
        flag: late && f.switch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing() {
        let f = FinishFigures {
            units: 100.0,
            ease: [2.0, 5.0],
            half: 0.5,
            share: [3.0, 6.0],
            switch: true,
        };
        let t = finish_timing(100, &f);
        assert_eq!((t.time, t.eased, t.share, t.flag), (1.0, 1.0, 0.0, false));
        let t = finish_timing(350, &f);
        assert_eq!(t.time, 3.5);
        assert_eq!(t.eased, (3.5 - 0.25 * 1.5) as f32);
        assert_eq!(t.share, (0.5f64 / 3.0) as f32);
        assert!(t.flag);
        let t = finish_timing(800, &f);
        assert_eq!((t.eased, t.share), (3.5, 1.0));
        let off = FinishFigures { switch: false, ..f };
        assert_eq!(finish_timing(800, &off).share, 0.0);
    }
}
