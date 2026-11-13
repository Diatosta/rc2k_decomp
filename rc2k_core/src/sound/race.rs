//! A race's sounds set up, and the car screens' sound stream.

/// The co-driver's two lists of speech files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    Male,
    Female,
}

/// The two cars' lists: the first car's by its +0x298 (0 the male list), the second the other,
/// or the same when the two +0x298 are the same.
pub fn voices(first: u32, second: u32) -> (Voice, Voice) {
    let (a, b) = if first == 0 {
        (Voice::Male, Voice::Female)
    } else {
        (Voice::Female, Voice::Male)
    };
    (a, if first == second { a } else { b })
}

/// The sets of sounds a race asks the audio thread for, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    /// A replay's one set.
    Replay,
    /// The first car's kind's set, then the first car's three.
    FirstKind(u32),
    First(usize),
    /// The second car's kind's set (unless the count is 1), then its two.
    SecondKind(u32),
    Second(usize),
    /// The last set, unless the two counts are the same.
    Last,
}

/// The sets for a race: in a replay only its own; else the first car's kind and three, the second
/// car's kind and two unless `count` is 1, and the last unless `count` is `other_count`.
pub fn sets(
    replay: bool,
    first_kind: u32,
    second_kind: u32,
    count: u32,
    other_count: u32,
) -> Vec<Set> {
    if replay {
        return vec![Set::Replay];
    }
    let mut out = vec![Set::FirstKind(first_kind)];
    out.extend((0..3).map(Set::First));
    if count != 1 {
        out.push(Set::SecondKind(second_kind));
        out.extend((0..2).map(Set::Second));
    }
    if count != other_count {
        out.push(Set::Last);
    }
    out
}

/// What the car screens' stream does in a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamStep {
    /// Not on, or still playing.
    Nothing,
    /// On to `place`, where the list ends (-1): the stream is off.
    Ended { place: u32 },
    /// On to `place`, whose part is started.
    Start { place: u32, part: u32 },
}

/// A frame of the car screens' stream (only in the championship's kind 2, while it is on
/// and not playing): on to the next place of the list (`part` gives a place's part).
pub fn stream_step(
    kind: u32,
    on: u32,
    playing: impl FnOnce() -> bool,
    place: u32,
    part: impl FnOnce(u32) -> u32,
) -> StreamStep {
    if kind != 2 || on != 1 || playing() {
        return StreamStep::Nothing;
    }
    let place = place.wrapping_add(1);
    match part(place) {
        u32::MAX => StreamStep::Ended { place },
        part => StreamStep::Start { place, part },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race() {
        assert_eq!(voices(0, 0), (Voice::Male, Voice::Male));
        assert_eq!(voices(1, 0), (Voice::Female, Voice::Male));
        assert_eq!(sets(true, 0, 0, 0, 0), vec![Set::Replay]);
        assert_eq!(sets(false, 3, 4, 1, 1).len(), 4);
        assert_eq!(sets(false, 3, 4, 2, 1).len(), 8);
        assert_eq!(
            stream_step(2, 1, || false, 4, |p| p * 10),
            StreamStep::Start { place: 5, part: 50 }
        );
        assert_eq!(
            stream_step(2, 1, || false, 4, |_| u32::MAX),
            StreamStep::Ended { place: 5 }
        );
        assert_eq!(stream_step(1, 1, || false, 4, |_| 0), StreamStep::Nothing);
    }
}
