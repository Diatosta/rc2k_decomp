//! The championship's finish routine's set-up: one car in race mode 3 set up before the stage is
//! loaded and the routine played. What most variables are is a reading.

/// A variable the routine sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    D60101C,
    D621514,
    D623854,
    D6235FC,
    /// The race mode.
    RaceMode,
    D6238A4,
    /// The race's cars.
    RaceCars,
    D63DB90,
    D6238A8,
    D6238AC,
    D6238B0,
    D6238B4,
    D6238B8,
    D621890,
    D6238CC,
    D6238D0,
    D64BB18,
    D5C2F88,
    D606744,
    D606748,
    D63DB84,
}

/// A field of the first car block that the set-up stores into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lead {
    /// `camera_view`.
    CameraView,
    GearRatioSetting,
    SuspensionStiffness,
    RideHeight,
    BrakeBalance,
    SteeringSensitivity,
    PartsPresent,
    /// `impact_octant_marks`, by octant.
    ImpactMark(u32),
}

/// Where a store goes: a variable, or a field of the first car block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    Var(Var),
    Lead(Lead),
}

/// The set-up's stores in order (`RaceVars.direction` is stored twice).
pub fn set_up() -> Vec<(Set, u32)> {
    use Lead::*;
    use Set::{Lead as L, Var as V};
    use Var::*;
    let mut s = vec![(V(D60101C), 2), (V(D621514), 0), (V(D623854), 0)];
    for (o, v) in [
        (CameraView, 6),
        (GearRatioSetting, 2),
        (SuspensionStiffness, 0x80),
        (RideHeight, 0x80),
        (BrakeBalance, 0x80),
        (SteeringSensitivity, 0x80),
    ] {
        s.push((L(o), v));
    }
    s.push((V(D6235FC), 0));
    s.push((L(PartsPresent), u32::MAX));
    for k in 0..8 {
        s.push((L(ImpactMark(k)), 0));
    }
    for (x, v) in [
        (D623854, 0),
        (RaceMode, 3),
        (D6238A4, 0),
        (RaceCars, 1),
        (D63DB90, 1),
        (D6238A8, 1),
        (D6238AC, 1),
        (D6238B0, 1),
        (D6238B4, 0),
        (D6238B8, 1),
        (D621890, 1),
        (D6238CC, 0),
        (D6238D0, 0),
        (D64BB18, 1),
        (D5C2F88, 0),
    ] {
        s.push((V(x), v));
    }
    s
}

/// Cleared after the stage is loaded, before the routine.
pub const BEFORE_RUN: Var = Var::D63DB84;

/// Cleared after everything is tidied, in order.
pub const AFTER: [Var; 4] = [Var::D5C2F88, Var::D64BB18, Var::D606744, Var::D606748];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores() {
        let s = set_up();
        assert_eq!(s.len(), 3 + 6 + 2 + 8 + 15);
        assert_eq!(s[10], (Set::Lead(Lead::PartsPresent), u32::MAX));
        assert!(s.contains(&(Set::Var(Var::RaceMode), 3)));
    }
}
