//! What the layout byte of a replay block shows: the one rule the viewer's car item, the drawing
//! of the cars shown, the camera choice and the HUD ask. The byte names players, by their number
//! in the order of the replay's players (the first player is 0):
//!
//! - 0 and 1: the first and the second player alone; 4 .. 9: the third to the eighth alone;
//! - 2 and 3: the first two players together, side by side or one above the other;
//! - 10: the first group of players, split as a local split-screen race is: up to four, each in
//!   a part of the screen (for a three- or four-player replay, every player);
//! - 11: the second group, the players after the first four (three or four of them).

/// The layout of the first group of players.
pub const FIRST_GROUP: u8 = 10;
/// The layout of the second group of players.
pub const SECOND_GROUP: u8 = 11;
/// How many players a group splits the screen between at most.
pub const GROUP_MOST: usize = 4;
/// The fewest players a group splits the screen between (two players are the layouts 2 and 3).
pub const GROUP_LEAST: usize = 3;

/// The players a layout shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// One player alone.
    Player(usize),
    /// The first two players, in two views.
    Pair { stacked: bool },
    /// `count` players from player `first`, each in a part of the screen.
    Group { first: usize, count: usize },
}

/// What `layout` shows of a replay of `players` players; none when the layout is none of the
/// ones this many players have.
pub fn shown(layout: u8, players: usize) -> Option<Shown> {
    let player = |k: usize| (k < players).then_some(Shown::Player(k));
    match layout {
        0 | 1 => player(usize::from(layout)),
        2 | 3 if players >= 2 => Some(Shown::Pair {
            stacked: layout == 3,
        }),
        4..=9 => player(usize::from(layout) - 2),
        FIRST_GROUP if players >= GROUP_LEAST => Some(Shown::Group {
            first: 0,
            count: players.min(GROUP_MOST),
        }),
        SECOND_GROUP if players >= GROUP_MOST + GROUP_LEAST => Some(Shown::Group {
            first: GROUP_MOST,
            count: players - GROUP_MOST,
        }),
        _ => None,
    }
}

/// Whether `layout` is one of the groups (whatever the replay's players).
pub fn is_group(layout: u8) -> bool {
    layout == FIRST_GROUP || layout == SECOND_GROUP
}

/// The layout that shows player `k` alone.
pub fn layout_of_player(k: usize) -> u8 {
    if k < 2 { k as u8 } else { k as u8 + 2 }
}

/// Where the camera of a car is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraSource {
    /// Byte `n` of the block's head: 0 and 1 the first two cars', 6 and 7 the third's and the
    /// fourth's.
    Head(usize),
    /// The choice made in the viewer for car `4 + n`: the blocks hold no byte for it.
    Live(usize),
}

/// Where car `car`'s camera is kept; none for a car past the eighth.
pub fn camera_source(car: usize) -> Option<CameraSource> {
    match car {
        0 | 1 => Some(CameraSource::Head(car)),
        2 | 3 => Some(CameraSource::Head(car + 4)),
        4..=7 => Some(CameraSource::Live(car - 4)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_players_pairs_and_groups() {
        let layouts = |players| (0..=12).map(|l| shown(l, players)).collect::<Vec<_>>();
        // One or two players: the players alone, and for two the pair.
        assert_eq!(shown(1, 1), None);
        assert_eq!(shown(0, 1), Some(Shown::Player(0)));
        assert_eq!(shown(2, 2), Some(Shown::Pair { stacked: false }));
        assert_eq!(shown(FIRST_GROUP, 2), None);
        // Three and four: every player alone, the pair, one group of all of them.
        assert_eq!(
            shown(FIRST_GROUP, 3),
            Some(Shown::Group { first: 0, count: 3 })
        );
        assert_eq!(
            shown(FIRST_GROUP, 4),
            Some(Shown::Group { first: 0, count: 4 })
        );
        assert_eq!(shown(SECOND_GROUP, 4), None);
        assert_eq!(shown(5, 4), Some(Shown::Player(3)));
        assert_eq!(shown(6, 4), None);
        // Five and six: the first four split; no second group (it would be of one or two).
        assert_eq!(
            shown(FIRST_GROUP, 5),
            Some(Shown::Group { first: 0, count: 4 })
        );
        assert_eq!(shown(SECOND_GROUP, 6), None);
        // Seven: the second group of three; eight: of four.
        assert_eq!(
            shown(SECOND_GROUP, 7),
            Some(Shown::Group { first: 4, count: 3 })
        );
        assert_eq!(
            shown(SECOND_GROUP, 8),
            Some(Shown::Group { first: 4, count: 4 })
        );
        assert_eq!(layouts(8)[9], Some(Shown::Player(7)));
        assert_eq!(layouts(8)[12], None);
    }

    #[test]
    fn a_layout_names_its_player() {
        for k in 0..8 {
            assert_eq!(shown(layout_of_player(k), 8), Some(Shown::Player(k)));
        }
    }

    #[test]
    fn cameras_by_car() {
        assert_eq!(camera_source(0), Some(CameraSource::Head(0)));
        assert_eq!(camera_source(1), Some(CameraSource::Head(1)));
        assert_eq!(camera_source(2), Some(CameraSource::Head(6)));
        assert_eq!(camera_source(3), Some(CameraSource::Head(7)));
        assert_eq!(camera_source(4), Some(CameraSource::Live(0)));
        assert_eq!(camera_source(7), Some(CameraSource::Live(3)));
        assert_eq!(camera_source(8), None);
    }
}
