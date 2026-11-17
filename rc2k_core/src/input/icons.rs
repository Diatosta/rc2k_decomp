//! Button pictures: which family of pictures a controller's buttons are shown with (Xbox,
//! PlayStation, Steam Deck, or plain numbers), and which picture a button number or hat direction
//! is in that family.
//!
//! A player chooses a style for each controller (`IconStyle`, kept in the controller's profile);
//! "automatic" follows what the platform found out about the controller (`Family` of a gamepad).
//! The pictures' atlas is described by an index of lines "name x y width height" (`parse_index`);
//! the names are `Icon::name`.

use crate::input::profile::{Digital, HatDirection, Kind, MenuControl};
use serde::{Deserialize, Serialize};

/// A family of pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Xbox,
    PlayStation,
    SteamDeck,
}

impl Family {
    pub const ALL: [Family; 3] = [Family::Xbox, Family::PlayStation, Family::SteamDeck];

    /// The family's file name stem in `assets/glyphs`.
    pub fn file(self) -> &'static str {
        match self {
            Family::Xbox => "xbox",
            Family::PlayStation => "playstation",
            Family::SteamDeck => "steamdeck",
        }
    }
}

/// What a controller's buttons are shown as, as the player chose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconStyle {
    /// The pictures of the family the platform found for the controller; numbers for one it
    /// found none for.
    #[default]
    Automatic,
    Xbox,
    PlayStation,
    SteamDeck,
    /// Button numbers and the directions' names, never pictures.
    Numbers,
}

impl IconStyle {
    /// The styles in the order the controller page cycles them.
    pub const ALL: [IconStyle; 5] = [
        IconStyle::Automatic,
        IconStyle::Xbox,
        IconStyle::PlayStation,
        IconStyle::SteamDeck,
        IconStyle::Numbers,
    ];

    /// The style's place in `ALL`.
    pub fn place(self) -> usize {
        self as usize
    }

    /// The style one step on, round the list.
    pub fn stepped(self, up: bool) -> IconStyle {
        let n = Self::ALL.len();
        Self::ALL[(self.place() + if up { 1 } else { n - 1 }) % n]
    }

    /// The pictures' family: the one chosen, or for "automatic" the one found (`detected`);
    /// none for numbers.
    pub fn family(self, detected: Option<Family>) -> Option<Family> {
        match self {
            IconStyle::Automatic => detected,
            IconStyle::Xbox => Some(Family::Xbox),
            IconStyle::PlayStation => Some(Family::PlayStation),
            IconStyle::SteamDeck => Some(Family::SteamDeck),
            IconStyle::Numbers => None,
        }
    }
}

/// The make of a gamepad as SDL says it, as far as the pictures go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadMake {
    Xbox,
    PlayStation,
    /// A Steam controller or the Steam Deck's own pad.
    Steam,
    Other,
}

/// USB ids of Valve's hardware.
pub const VALVE: u16 = 0x28DE;
pub const STEAM_DECK: u16 = 0x1205;

/// The family a gamepad's buttons are drawn from when the player leaves it to the program: a
/// Steam Deck by its USB ids (or Steam's own pad, or, when the program runs on a Deck under
/// Steam, any pad Steam makes: `on_deck`), a PlayStation pad's, else Xbox's (the look of a pad
/// nothing is known about).
pub fn pad_family(make: PadMake, vendor: u16, product: u16, on_deck: bool) -> Family {
    if (vendor == VALVE && product == STEAM_DECK)
        || make == PadMake::Steam
        || (on_deck && vendor == VALVE)
    {
        Family::SteamDeck
    } else if make == PadMake::PlayStation {
        Family::PlayStation
    } else {
        Family::Xbox
    }
}

/// A picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    /// The face buttons by place (A, B, X, Y on an Xbox pad; cross, circle, square, triangle on
    /// a PlayStation's).
    South,
    East,
    West,
    North,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
    Start,
    Back,
    Guide,
    Touchpad,
    /// The left and right sticks, pushed one way (or either way along an axis).
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    LeftStickHorizontal,
    LeftStickVertical,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
    RightStickHorizontal,
    RightStickVertical,
}

impl Icon {
    pub const ALL: [Icon; 30] = [
        Icon::South,
        Icon::East,
        Icon::West,
        Icon::North,
        Icon::DpadUp,
        Icon::DpadDown,
        Icon::DpadLeft,
        Icon::DpadRight,
        Icon::L1,
        Icon::R1,
        Icon::L2,
        Icon::R2,
        Icon::L3,
        Icon::R3,
        Icon::Start,
        Icon::Back,
        Icon::Guide,
        Icon::Touchpad,
        Icon::LeftStickUp,
        Icon::LeftStickDown,
        Icon::LeftStickLeft,
        Icon::LeftStickRight,
        Icon::LeftStickHorizontal,
        Icon::LeftStickVertical,
        Icon::RightStickUp,
        Icon::RightStickDown,
        Icon::RightStickLeft,
        Icon::RightStickRight,
        Icon::RightStickHorizontal,
        Icon::RightStickVertical,
    ];

    /// The name the atlas' index gives the picture.
    pub fn name(self) -> &'static str {
        match self {
            Icon::South => "south",
            Icon::East => "east",
            Icon::West => "west",
            Icon::North => "north",
            Icon::DpadUp => "dpad_up",
            Icon::DpadDown => "dpad_down",
            Icon::DpadLeft => "dpad_left",
            Icon::DpadRight => "dpad_right",
            Icon::L1 => "l1",
            Icon::R1 => "r1",
            Icon::L2 => "l2",
            Icon::R2 => "r2",
            Icon::L3 => "l3",
            Icon::R3 => "r3",
            Icon::Start => "start",
            Icon::Back => "back",
            Icon::Guide => "guide",
            Icon::Touchpad => "touchpad",
            Icon::LeftStickUp => "ls_up",
            Icon::LeftStickDown => "ls_down",
            Icon::LeftStickLeft => "ls_left",
            Icon::LeftStickRight => "ls_right",
            Icon::LeftStickHorizontal => "ls_horizontal",
            Icon::LeftStickVertical => "ls_vertical",
            Icon::RightStickUp => "rs_up",
            Icon::RightStickDown => "rs_down",
            Icon::RightStickLeft => "rs_left",
            Icon::RightStickRight => "rs_right",
            Icon::RightStickHorizontal => "rs_horizontal",
            Icon::RightStickVertical => "rs_vertical",
        }
    }

    /// The picture of a gamepad's axis (numbered as `axis_label` does: x, y, rx, ry, lt, rt)
    /// and the side of it in use: a stick pushed one way (`Some(true)` is right or down), the
    /// stick along its whole axis (`None`), or a trigger. None for an axis a gamepad has not.
    pub fn of_pad_axis(axis: u8, side: Option<bool>) -> Option<Icon> {
        Some(match (axis, side) {
            (0, Some(true)) => Icon::LeftStickRight,
            (0, Some(false)) => Icon::LeftStickLeft,
            (0, None) => Icon::LeftStickHorizontal,
            (1, Some(true)) => Icon::LeftStickDown,
            (1, Some(false)) => Icon::LeftStickUp,
            (1, None) => Icon::LeftStickVertical,
            (2, Some(true)) => Icon::RightStickRight,
            (2, Some(false)) => Icon::RightStickLeft,
            (2, None) => Icon::RightStickHorizontal,
            (3, Some(true)) => Icon::RightStickDown,
            (3, Some(false)) => Icon::RightStickUp,
            (3, None) => Icon::RightStickVertical,
            (4, _) => Icon::L2,
            (5, _) => Icon::R2,
            _ => return None,
        })
    }

    /// The picture of a hat's direction.
    pub fn of_hat(direction: HatDirection) -> Icon {
        match direction {
            HatDirection::Up => Icon::DpadUp,
            HatDirection::Right => Icon::DpadRight,
            HatDirection::Down => Icon::DpadDown,
            HatDirection::Left => Icon::DpadLeft,
        }
    }
}

/// A gamepad's buttons as the profiles number them (from 0): the face buttons in the order
/// south, east, west, north, the shoulders, back, start, the sticks' presses.
const PAD_ORDER: [Icon; 10] = [
    Icon::South,
    Icon::East,
    Icon::West,
    Icon::North,
    Icon::L1,
    Icon::R1,
    Icon::Back,
    Icon::Start,
    Icon::L3,
    Icon::R3,
];

/// The buttons of a joystick or wheel that DirectInput numbers as a PlayStation pad's (the
/// first is the square): from 0.
const PLAYSTATION_ORDER: [Icon; 14] = [
    Icon::West,
    Icon::South,
    Icon::East,
    Icon::North,
    Icon::L1,
    Icon::R1,
    Icon::L2,
    Icon::R2,
    Icon::Back,
    Icon::Start,
    Icon::L3,
    Icon::R3,
    Icon::Guide,
    Icon::Touchpad,
];

/// The picture of button `button` (from 0, as a profile binds it) of a controller of `kind`,
/// shown as `family`. A gamepad's buttons are in the profiles' own numbering whatever its make;
/// another controller's are in DirectInput's order for a PlayStation pad, or for an Xbox one
/// (the Steam Deck's as an Xbox's). None for a button the order has no picture for.
pub fn button_icon(family: Family, kind: Kind, button: u8) -> Option<Icon> {
    let order: &[Icon] = if kind == Kind::Pad || family != Family::PlayStation {
        &PAD_ORDER
    } else {
        &PLAYSTATION_ORDER
    };
    order.get(button as usize).copied()
}

/// What a controller's buttons are drawn with: the family of pictures and the controller's kind
/// (which says how its button numbers are ordered).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pictures {
    pub family: Family,
    pub kind: Kind,
}

impl Pictures {
    /// The pictures for a controller: the family its profile's style gives (`detected` for
    /// "automatic"); none when the buttons are shown as numbers.
    pub fn of(kind: Kind, style: IconStyle, detected: Option<Family>) -> Option<Pictures> {
        style
            .family(detected)
            .map(|family| Pictures { family, kind })
    }

    /// What a button or hat direction is bound to, as a picture; none for a button the
    /// controller's order has no picture for.
    pub fn digital(&self, digital: Digital) -> Option<Icon> {
        match digital {
            Digital::Button { button } => button_icon(self.family, self.kind, button),
            Digital::Hat { direction, .. } => Some(Icon::of_hat(direction)),
        }
    }

    /// The picture of a menu control: the button the profile binds to it, or where the controller
    /// has it by default (a gamepad's south enters, east goes back, west is the third answer;
    /// another controller's second button enters and its third goes back).
    pub fn menu(&self, control: MenuControl, bound: Option<Digital>) -> Option<Icon> {
        if let Some(digital) = bound {
            return self.digital(digital);
        }
        let by_place = |place: u8| button_icon(self.family, self.kind, place);
        match (self.kind, control) {
            (Kind::Pad, MenuControl::Enter) => Some(Icon::South),
            (Kind::Pad, MenuControl::Back) => Some(Icon::East),
            (Kind::Pad, MenuControl::Third) => Some(Icon::West),
            (Kind::Pad, MenuControl::Tab) => Some(Icon::L1),
            (_, MenuControl::Enter) => by_place(1),
            (_, MenuControl::Back) => by_place(2),
            _ => None,
        }
    }
}

/// The pictures an atlas holds, from its index: each line is "name x y width height".
/// Lines that are blank or name no picture are passed over.
pub fn parse_index(text: &str) -> Vec<(Icon, [u32; 4])> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            let icon = Icon::ALL.into_iter().find(|i| i.name() == name)?;
            let mut rect = [0u32; 4];
            for slot in &mut rect {
                *slot = words.next()?.parse().ok()?;
            }
            Some((icon, rect))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_follows_what_was_found() {
        assert_eq!(IconStyle::Automatic.family(None), None);
        assert_eq!(
            IconStyle::Automatic.family(Some(Family::SteamDeck)),
            Some(Family::SteamDeck)
        );
        assert_eq!(
            IconStyle::PlayStation.family(Some(Family::Xbox)),
            Some(Family::PlayStation)
        );
        assert_eq!(IconStyle::Numbers.family(Some(Family::Xbox)), None);
    }

    #[test]
    fn a_gamepads_family_follows_its_make_and_ids() {
        use PadMake::*;
        assert_eq!(pad_family(Xbox, 0x45E, 0x2FF, false), Family::Xbox);
        assert_eq!(
            pad_family(PlayStation, 0x54C, 0xCE6, false),
            Family::PlayStation
        );
        assert_eq!(pad_family(Other, 1, 2, false), Family::Xbox);
        assert_eq!(
            pad_family(Other, VALVE, STEAM_DECK, false),
            Family::SteamDeck
        );
        assert_eq!(pad_family(Steam, 0, 0, false), Family::SteamDeck);
        // Steam's virtual pad, seen as an Xbox 360 one, on a Deck only
        assert_eq!(pad_family(Xbox, VALVE, 0x11FF, true), Family::SteamDeck);
        assert_eq!(pad_family(Xbox, VALVE, 0x11FF, false), Family::Xbox);
        assert_eq!(pad_family(Xbox, 0x45E, 0x2FF, true), Family::Xbox);
    }

    #[test]
    fn the_style_steps_round_the_list() {
        let mut style = IconStyle::Automatic;
        for _ in 0..IconStyle::ALL.len() {
            style = style.stepped(true);
        }
        assert_eq!(style, IconStyle::Automatic);
        assert_eq!(IconStyle::Automatic.stepped(false), IconStyle::Numbers);
        assert_eq!(IconStyle::Numbers.stepped(true), IconStyle::Automatic);
    }

    #[test]
    fn a_gamepads_buttons_follow_the_profiles_numbers() {
        for family in Family::ALL {
            assert_eq!(button_icon(family, Kind::Pad, 0), Some(Icon::South));
            assert_eq!(button_icon(family, Kind::Pad, 2), Some(Icon::West));
            assert_eq!(button_icon(family, Kind::Pad, 6), Some(Icon::Back));
            assert_eq!(button_icon(family, Kind::Pad, 7), Some(Icon::Start));
            assert_eq!(button_icon(family, Kind::Pad, 10), None);
        }
    }

    #[test]
    fn a_wheel_set_to_playstation_has_the_directinput_order() {
        let p = |n| button_icon(Family::PlayStation, Kind::Wheel, n);
        // 1 square, 2 cross, 3 circle, 4 triangle (the numbers a player sees are from 1)
        assert_eq!(p(0), Some(Icon::West));
        assert_eq!(p(1), Some(Icon::South));
        assert_eq!(p(2), Some(Icon::East));
        assert_eq!(p(3), Some(Icon::North));
        assert_eq!(p(8), Some(Icon::Back));
        assert_eq!(p(9), Some(Icon::Start));
        assert_eq!(p(13), Some(Icon::Touchpad));
        assert_eq!(p(14), None);
    }

    #[test]
    fn a_joystick_set_to_xbox_has_the_xbox_order() {
        let x = |n| button_icon(Family::Xbox, Kind::Joystick, n);
        assert_eq!(x(0), Some(Icon::South));
        assert_eq!(x(1), Some(Icon::East));
        assert_eq!(x(4), Some(Icon::L1));
        assert_eq!(x(6), Some(Icon::Back));
        assert_eq!(x(7), Some(Icon::Start));
        assert_eq!(x(8), Some(Icon::L3));
        assert_eq!(x(10), None);
        assert_eq!(
            button_icon(Family::SteamDeck, Kind::Joystick, 1),
            Some(Icon::East)
        );
    }

    #[test]
    fn menu_controls_are_where_the_controller_has_them() {
        let pad = Pictures::of(Kind::Pad, IconStyle::Automatic, Some(Family::PlayStation)).unwrap();
        assert_eq!(pad.menu(MenuControl::Enter, None), Some(Icon::South));
        assert_eq!(pad.menu(MenuControl::Back, None), Some(Icon::East));
        let wheel = Pictures::of(Kind::Wheel, IconStyle::PlayStation, None).unwrap();
        // the second button is the cross, the third the circle
        assert_eq!(wheel.menu(MenuControl::Enter, None), Some(Icon::South));
        assert_eq!(wheel.menu(MenuControl::Back, None), Some(Icon::East));
        assert_eq!(
            wheel.menu(MenuControl::Back, Some(Digital::Button { button: 3 })),
            Some(Icon::North)
        );
        assert_eq!(Pictures::of(Kind::Wheel, IconStyle::Automatic, None), None);
    }

    #[test]
    fn a_hat_direction_is_a_dpad_picture() {
        assert_eq!(Icon::of_hat(HatDirection::Up), Icon::DpadUp);
        assert_eq!(Icon::of_hat(HatDirection::Left), Icon::DpadLeft);
    }

    #[test]
    fn the_style_is_kept_in_the_profile_only_when_chosen() {
        use crate::input::device_id::DeviceId;
        use crate::input::profile::Profile;
        let id = DeviceId {
            name: "Pad".into(),
            serial: None,
            nth: 1,
        };
        let mut profile = Profile::new(id, Kind::Pad);
        let text = toml::to_string(&profile).unwrap();
        assert!(!text.contains("button_icons"), "{text}");
        profile.button_icons = IconStyle::SteamDeck;
        let text = toml::to_string(&profile).unwrap();
        assert!(text.contains("button_icons = \"steam_deck\""), "{text}");
        let back: Profile = toml::from_str(&text).unwrap();
        assert_eq!(back.button_icons, IconStyle::SteamDeck);
        // a profile written before the setting existed has the automatic style
        let old = text.replace("button_icons = \"steam_deck\"\n", "");
        assert_eq!(
            toml::from_str::<Profile>(&old).unwrap().button_icons,
            IconStyle::Automatic
        );
    }

    #[test]
    fn the_index_parses() {
        let parsed = parse_index("south 0 0 64 64\n\nnothing 1 2 3 4\nl1 64 0 64 64\nbad 1\n");
        assert_eq!(
            parsed,
            vec![(Icon::South, [0, 0, 64, 64]), (Icon::L1, [64, 0, 64, 64])]
        );
        for family in Family::ALL {
            let text = std::fs::read_to_string(format!(
                "{}/../assets/glyphs/{}.idx",
                env!("CARGO_MANIFEST_DIR"),
                family.file()
            ))
            .unwrap();
            let icons = parse_index(&text);
            assert_eq!(icons.len(), text.lines().count());
            for (icon, [x, y, w, h]) in icons {
                assert!(x + w <= 1024 && y + h <= 512, "{family:?} {icon:?}");
            }
        }
    }

    #[test]
    fn a_gamepads_axes_have_pictures() {
        assert_eq!(Icon::of_pad_axis(0, None), Some(Icon::LeftStickHorizontal));
        assert_eq!(Icon::of_pad_axis(1, Some(false)), Some(Icon::LeftStickUp));
        assert_eq!(Icon::of_pad_axis(3, Some(true)), Some(Icon::RightStickDown));
        assert_eq!(Icon::of_pad_axis(4, Some(true)), Some(Icon::L2));
        assert_eq!(Icon::of_pad_axis(5, None), Some(Icon::R2));
        assert_eq!(Icon::of_pad_axis(6, None), None);
        // every picture is named once
        let mut names: Vec<_> = Icon::ALL.iter().map(|i| i.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), Icon::ALL.len());
    }
}
