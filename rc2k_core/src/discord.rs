//! Discord Rich Presence: what the status says and how it is put on the wire. No I/O here: the
//! program reads the game's state into a [`Situation`], this makes the [`Activity`] of it and the
//! frames Discord's local IPC takes (`src/platform/discord.rs` carries them).

use crate::menu::launch::english_name;

/// The longest text Discord shows in one line of the status.
pub const MAX_TEXT: usize = 128;

/// The application's own picture's key in its art assets, and its caption.
pub const LOGO_IMAGE: &str = "logo";
pub const LOGO_TEXT: &str = "Rally Championship";

/// The kind of race being driven.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    SingleRally,
    TimeTrial,
    Championship,
    A8Championship,
    /// The arcade, at a level as its screen counts them (1 up).
    Arcade {
        level: u32,
    },
    /// A network race with this many human players.
    Network {
        players: u32,
    },
    /// A replay in the viewer.
    Replay,
}

/// What the player is doing now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Situation {
    Menus,
    Racing {
        event: Event,
        /// The car's English name without its class, when known.
        car: Option<&'static str>,
        /// The stage's English name, when known.
        stage: Option<&'static str>,
    },
}

/// The status Discord shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activity {
    pub details: String,
    pub state: Option<String>,
    /// When the race began, seconds since 1970; Discord shows the time elapsed from it.
    pub start: Option<u64>,
    /// The big picture's asset key and caption: the car's, else the logo.
    pub large_image: String,
    pub large_text: String,
    /// The logo beside the car's picture (none when the logo is the big one).
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

/// The game's text number of the update disc's Clio.
const CLIO_TEXT: u32 = 0x1038;

/// A car's English name from the game's text number, the class suffix (" (A8)") taken off.
pub fn car_name(text_id: u32) -> Option<&'static str> {
    // The update disc's Clio (car 33) is not in the launch's table.
    if text_id == CLIO_TEXT {
        return Some("Renault Clio");
    }
    let name = english_name(text_id)?;
    Some(strip_class(name))
}

/// `name` without a trailing " (A8)"-like class mark.
pub fn strip_class(name: &str) -> &str {
    match name.rfind(" (") {
        Some(at) if name.ends_with(')') => &name[..at],
        _ => name,
    }
}

/// The art asset key of a car by its name: lowercase, every run of characters that are not
/// letters or digits one `_`, none at the ends ("Moo 1.8bse Turbo" is `moo_1_8bse_turbo`).
pub fn car_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());
    for c in strip_class(name).chars() {
        if c.is_ascii_alphanumeric() {
            key.push(c.to_ascii_lowercase());
        } else if !key.ends_with('_') && !key.is_empty() {
            key.push('_');
        }
    }
    while key.ends_with('_') {
        key.pop();
    }
    key
}

/// A stage's English name from the game's text number (a stage name, not a car's).
pub fn stage_name(text_id: u32) -> Option<&'static str> {
    if (0x3E8..=0x40B).contains(&text_id) {
        english_name(text_id)
    } else {
        None
    }
}

fn article(word: &str) -> &'static str {
    match word.chars().next() {
        Some('A' | 'E' | 'I' | 'O' | 'U') => "an",
        _ => "a",
    }
}

fn clip(text: String) -> String {
    if text.chars().count() <= MAX_TEXT {
        text
    } else {
        text.chars().take(MAX_TEXT).collect()
    }
}

/// The status for `situation`; `start` is when the race began.
pub fn activity(situation: &Situation, start: u64) -> Activity {
    let logo = || LOGO_IMAGE.to_owned();
    let Situation::Racing { event, car, stage } = *situation else {
        return Activity {
            details: "In the menus".to_owned(),
            state: None,
            start: None,
            large_image: logo(),
            large_text: LOGO_TEXT.to_owned(),
            small_image: None,
            small_text: None,
        };
    };
    let kind = match event {
        Event::SingleRally => Some("Single rally"),
        Event::TimeTrial => Some("Time trial"),
        Event::Championship => Some("Championship"),
        Event::A8Championship => Some("A8 championship"),
        _ => None,
    };
    let (details, state) = match event {
        Event::SingleRally | Event::TimeTrial | Event::Championship | Event::A8Championship => (
            match car {
                Some(car) => format!("Rallying {} {car}", article(car)),
                None => "Rallying".to_owned(),
            },
            match (kind, stage) {
                (Some(kind), Some(stage)) => Some(format!("{kind} \u{b7} {stage}")),
                (Some(kind), None) => Some(kind.to_owned()),
                _ => None,
            },
        ),
        Event::Arcade { level } => (
            format!("Arcade, level {level}"),
            match (car, stage) {
                (Some(car), Some(stage)) => Some(format!("{car} at {stage}")),
                (Some(only), None) | (None, Some(only)) => Some(only.to_owned()),
                (None, None) => None,
            },
        ),
        Event::Network { players } => (
            "Network race".to_owned(),
            match (stage, players) {
                (Some(stage), 2..) => Some(format!("{stage} \u{b7} {players} players")),
                (Some(stage), _) => Some(stage.to_owned()),
                (None, 2..) => Some(format!("{players} players")),
                (None, _) => None,
            },
        ),
        Event::Replay => ("Watching a replay".to_owned(), stage.map(str::to_owned)),
    };
    // Network races and replays do not know the car's name here; a car given is shown.
    let (large_image, large_text, small_image, small_text) = match car {
        Some(car) => (
            car_key(car),
            car.to_owned(),
            Some(logo()),
            Some(LOGO_TEXT.to_owned()),
        ),
        None => (logo(), LOGO_TEXT.to_owned(), None, None),
    };
    Activity {
        details: clip(details),
        state: state.map(clip),
        start: Some(start),
        large_image,
        large_text,
        small_image,
        small_text,
    }
}

/// `text` as the inside of a JSON string.
pub fn escape_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// The opcode of the handshake frame.
pub const OP_HANDSHAKE: u32 = 0;
/// The opcode of a command frame.
pub const OP_FRAME: u32 = 1;

/// The handshake's JSON for application `client_id`.
pub fn handshake_json(client_id: &str) -> String {
    format!("{{\"v\":1,\"client_id\":\"{}\"}}", escape_json(client_id))
}

/// The `SET_ACTIVITY` command's JSON: `None` clears the status.
pub fn set_activity_json(pid: u32, activity: Option<&Activity>, nonce: u64) -> String {
    let body = match activity {
        None => "null".to_owned(),
        Some(a) => {
            let mut s = format!("{{\"details\":\"{}\"", escape_json(&a.details));
            if let Some(state) = &a.state {
                s.push_str(&format!(",\"state\":\"{}\"", escape_json(state)));
            }
            if let Some(start) = a.start {
                s.push_str(&format!(",\"timestamps\":{{\"start\":{start}}}"));
            }
            s.push_str(&format!(
                ",\"assets\":{{\"large_image\":\"{}\",\"large_text\":\"{}\"",
                escape_json(&a.large_image),
                escape_json(&a.large_text)
            ));
            if let (Some(image), Some(text)) = (&a.small_image, &a.small_text) {
                s.push_str(&format!(
                    ",\"small_image\":\"{}\",\"small_text\":\"{}\"",
                    escape_json(image),
                    escape_json(text)
                ));
            }
            s.push_str("}}");
            s
        }
    };
    format!(
        "{{\"cmd\":\"SET_ACTIVITY\",\"args\":{{\"pid\":{pid},\"activity\":{body}}},\"nonce\":\"{nonce}\"}}"
    )
}

/// A frame: the opcode and the JSON's length as little-endian dwords, then the JSON.
pub fn encode_frame(op: u32, json: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + json.len());
    out.extend_from_slice(&op.to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(json.as_bytes());
    out
}

/// A frame's opcode and body length from its eight header bytes.
pub fn parse_header(header: [u8; 8]) -> (u32, u32) {
    (
        u32::from_le_bytes([header[0], header[1], header[2], header[3]]),
        u32::from_le_bytes([header[4], header[5], header[6], header[7]]),
    )
}

/// The longest reply body read; a larger one means the stream is not Discord's.
pub const MAX_REPLY: u32 = 64 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_marks_come_off() {
        assert_eq!(car_name(0x22F), Some("Seat Cordoba WRC"));
        assert_eq!(car_name(0x22E), Some("Seat Ibiza Cupra Sport"));
        assert_eq!(strip_class("Tractor (A5)"), "Tractor");
        assert_eq!(strip_class("Plain"), "Plain");
        assert_eq!(car_name(1), None);
    }

    #[test]
    fn every_cars_key() {
        let keys: Vec<(u32, String)> = (0x222..=0x244)
            .map(|id| (id, car_key(english_name(id).unwrap())))
            .collect();
        let expected = [
            "citroen_saxo",
            "ford_escort_rs2000",
            "honda_civic",
            "hyundai_coupe_evo_ii",
            "mitsubishi_lancer_evo_v",
            "nissan_almera",
            "nissan_micra_maxi",
            "peugeot_106_maxi",
            "peugeot_206_wrc",
            "proton_compact",
            "proton_wira_persona",
            "renault_megane",
            "seat_ibiza_cupra_sport",
            "seat_cordoba_wrc",
            "skoda_felicia",
            "skoda_octavia",
            "subaru_impreza_wrc",
            "vauxhall_astra",
            "volkswagen_golf_gti_mk_iv",
            "ford_escort_maxi",
            "ford_puma",
            "mitsubishi_lancer_evo_iv",
            "land_rover",
            "tractor",
            "mf_turbo",
            "mf_hothatch",
            "bernie_s_dicemobile",
            "max_power_car",
            "skip",
            "turbotater",
            "moo_1_8bse_turbo",
            "lambaaghini",
            "radio_control_car",
            "welsh_sheep_wagon",
            "local_shop",
        ];
        let got: Vec<&str> = keys.iter().map(|(_, k)| k.as_str()).collect();
        assert_eq!(got, expected);
        assert_eq!(car_name(0x1038), Some("Renault Clio"));
        assert_eq!(car_key("Renault Clio"), "renault_clio");
        assert_eq!(car_key("Seat Cordoba WRC (A8)"), "seat_cordoba_wrc");
        assert_eq!(car_key("  --x//y-- "), "x_y");
    }

    #[test]
    fn stage_names_are_stages_only() {
        assert_eq!(stage_name(0x407), Some("Ballagyr"));
        assert_eq!(stage_name(0x22F), None);
        assert_eq!(stage_name(0x3E7), None);
    }

    fn racing(event: Event, car: Option<&'static str>, stage: Option<&'static str>) -> Activity {
        activity(&Situation::Racing { event, car, stage }, 1000)
    }

    #[test]
    fn menus() {
        let a = activity(&Situation::Menus, 5);
        assert_eq!(a.details, "In the menus");
        assert_eq!((a.state.clone(), a.start), (None, None));
        assert_eq!(a.large_image, "logo");
        assert_eq!(a.small_image, None);
    }

    #[test]
    fn rally_modes() {
        let a = racing(
            Event::SingleRally,
            Some("Seat Cordoba WRC"),
            Some("Ballagyr"),
        );
        assert_eq!(a.details, "Rallying a Seat Cordoba WRC");
        assert_eq!(a.state.as_deref(), Some("Single rally \u{b7} Ballagyr"));
        assert_eq!(a.start, Some(1000));
        assert_eq!(a.large_image, "seat_cordoba_wrc");
        assert_eq!(a.large_text, "Seat Cordoba WRC");
        assert_eq!(a.small_image.as_deref(), Some("logo"));
        let a = racing(Event::Championship, None, Some("Ballagyr"));
        assert_eq!(a.details, "Rallying");
        assert_eq!(a.state.as_deref(), Some("Championship \u{b7} Ballagyr"));
        assert_eq!(a.large_image, "logo");
        let a = racing(Event::TimeTrial, Some("Audi"), None);
        assert_eq!(a.details, "Rallying an Audi");
        assert_eq!(a.state.as_deref(), Some("Time trial"));
        let a = racing(Event::A8Championship, None, None);
        assert_eq!(a.state.as_deref(), Some("A8 championship"));
    }

    #[test]
    fn arcade_network_replay() {
        let a = racing(
            Event::Arcade { level: 1 },
            Some("Seat Ibiza Cupra Sport"),
            Some("Port Soderick"),
        );
        assert_eq!(a.details, "Arcade, level 1");
        assert_eq!(
            a.state.as_deref(),
            Some("Seat Ibiza Cupra Sport at Port Soderick")
        );
        let a = racing(Event::Arcade { level: 3 }, None, Some("Langton"));
        assert_eq!(a.state.as_deref(), Some("Langton"));
        let a = racing(Event::Network { players: 3 }, None, Some("Langton"));
        assert_eq!(a.details, "Network race");
        assert_eq!(a.state.as_deref(), Some("Langton \u{b7} 3 players"));
        let a = racing(Event::Network { players: 1 }, None, Some("Langton"));
        assert_eq!(a.state.as_deref(), Some("Langton"));
        let a = racing(Event::Replay, None, Some("Langton"));
        assert_eq!(a.details, "Watching a replay");
        assert_eq!(a.state.as_deref(), Some("Langton"));
    }

    #[test]
    fn texts_are_clipped() {
        let long = "x".repeat(300);
        assert_eq!(clip(long).chars().count(), MAX_TEXT);
    }

    #[test]
    fn json_escapes() {
        assert_eq!(escape_json("a\"b\\c\nd\u{1}"), "a\\\"b\\\\c\\nd\\u0001");
        assert_eq!(escape_json("Tholt \u{b7} \u{e9}"), "Tholt \u{b7} \u{e9}");
    }

    #[test]
    fn handshake_and_frames() {
        let json = handshake_json("123");
        assert_eq!(json, "{\"v\":1,\"client_id\":\"123\"}");
        let frame = encode_frame(OP_HANDSHAKE, &json);
        assert_eq!(&frame[..4], &[0, 0, 0, 0]);
        assert_eq!(&frame[4..8], &(json.len() as u32).to_le_bytes());
        assert_eq!(&frame[8..], json.as_bytes());
        assert_eq!(
            parse_header(frame[..8].try_into().unwrap()),
            (0, json.len() as u32)
        );
        assert_eq!(parse_header([1, 0, 0, 0, 2, 1, 0, 0]), (1, 258));
    }

    #[test]
    fn set_activity_commands() {
        let a = Activity {
            details: "Rallying a \"X\"".to_owned(),
            state: Some("S".to_owned()),
            start: Some(77),
            large_image: "x".to_owned(),
            large_text: "X".to_owned(),
            small_image: Some("logo".to_owned()),
            small_text: Some("Rally Championship".to_owned()),
        };
        assert_eq!(
            set_activity_json(9, Some(&a), 4),
            "{\"cmd\":\"SET_ACTIVITY\",\"args\":{\"pid\":9,\"activity\":{\"details\":\"Rallying a \\\"X\\\"\",\"state\":\"S\",\"timestamps\":{\"start\":77},\"assets\":{\"large_image\":\"x\",\"large_text\":\"X\",\"small_image\":\"logo\",\"small_text\":\"Rally Championship\"}}},\"nonce\":\"4\"}"
        );
        assert_eq!(
            set_activity_json(9, None, 5),
            "{\"cmd\":\"SET_ACTIVITY\",\"args\":{\"pid\":9,\"activity\":null},\"nonce\":\"5\"}"
        );
        let menus = activity(&Situation::Menus, 0);
        let json = set_activity_json(1, Some(&menus), 1);
        assert!(!json.contains("timestamps") && !json.contains("\"state\""));
        assert!(json.contains("\"large_image\":\"logo\"") && !json.contains("small_image"));
    }
}
