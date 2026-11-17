//! How a session's description lives in a Steam lobby's data, and how a lobby and a session
//! instance stand for each other. No Steam in it: it is tested everywhere.

use crate::net::dp_session::Desc;

/// A lobby's instance GUID: its Steam ID and a mark that it is one.
pub fn lobby_instance(lobby: u64) -> [u8; 16] {
    let mut g = *b"\0\0\0\0\0\0\0\0STEAMLBY";
    g[..8].copy_from_slice(&lobby.to_le_bytes());
    g
}

/// The lobby a session instance stands for, when it is one.
pub fn instance_lobby(instance: &[u8; 16]) -> Option<u64> {
    (&instance[8..] == b"STEAMLBY").then(|| {
        let mut id = [0u8; 8];
        id.copy_from_slice(&instance[..8]);
        u64::from_le_bytes(id)
    })
}

/// The lobby data's keys.
pub mod keys {
    pub const VERSION: &str = "rc2k";
    pub const NAME: &str = "name";
    pub const FLAGS: &str = "flags";
    pub const MOST: &str = "most";
    pub const USER: &str = "user";
}

/// A session's description as lobby data (key, value), without the version.
pub fn desc_to_data(desc: &Desc) -> Vec<(&'static str, String)> {
    vec![
        (keys::NAME, desc.name.clone()),
        (keys::FLAGS, desc.flags.to_string()),
        (keys::MOST, desc.max_players.to_string()),
        (keys::USER, desc.user.map(|u| u.to_string()).join(",")),
    ]
}

/// A session's description from a lobby's data (`get`), its members' count and its
/// instance; `None` when the data is not a session of this game's.
pub fn desc_from_data(
    get: impl Fn(&str) -> Option<String>,
    version: &str,
    members: u32,
    lobby: u64,
    application: [u8; 16],
) -> Option<Desc> {
    if get(keys::VERSION)? != version {
        return None;
    }
    let number = |key: &str| get(key).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    let mut user = [0u32; 4];
    if let Some(text) = get(keys::USER) {
        for (slot, part) in user.iter_mut().zip(text.split(',')) {
            *slot = part.trim().parse().unwrap_or(0);
        }
    }
    Some(Desc {
        flags: number(keys::FLAGS),
        instance: lobby_instance(lobby),
        application,
        max_players: number(keys::MOST),
        current_players: members,
        name: get(keys::NAME).unwrap_or_default(),
        user,
    })
}

/// The lobby named by a `+connect_lobby <id>` launch argument, if there is one.
pub fn connect_lobby_argument(args: impl IntoIterator<Item = String>) -> Option<u64> {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "+connect_lobby" {
            return args.next()?.parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lobby_is_its_instance_and_back() {
        let g = lobby_instance(0x0110_0001_2345_6789);
        assert_eq!(instance_lobby(&g), Some(0x0110_0001_2345_6789));
        assert_eq!(instance_lobby(&[7; 16]), None);
    }

    #[test]
    fn a_description_survives_the_lobby_data() {
        let desc = Desc {
            flags: 0x44,
            max_players: 8,
            name: "Sunday".into(),
            user: [1, 2, 3, 4],
            ..Desc::default()
        };
        let mut data = desc_to_data(&desc);
        data.push((keys::VERSION, "V".into()));
        let get = |k: &str| {
            data.iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.clone())
        };
        let back = desc_from_data(get, "V", 3, 99, [9; 16]).expect("a session");
        assert_eq!(back.name, "Sunday");
        assert_eq!(
            (back.flags, back.max_players, back.current_players),
            (0x44, 8, 3)
        );
        assert_eq!(back.user, [1, 2, 3, 4]);
        assert_eq!(instance_lobby(&back.instance), Some(99));
        // another version of the protocol is not listed
        assert!(desc_from_data(|k| get(k), "W", 3, 99, [9; 16]).is_none());
    }

    #[test]
    fn the_launch_argument_names_a_lobby() {
        let args = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(
            connect_lobby_argument(args(&["rc2k", "+connect_lobby", "42"])),
            Some(42)
        );
        assert_eq!(
            connect_lobby_argument(args(&["rc2k", "+connect_lobby"])),
            None
        );
        assert_eq!(connect_lobby_argument(args(&["rc2k"])), None);
    }
}
