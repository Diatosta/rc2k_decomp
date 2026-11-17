//! The lobby's chat and settings: eight lines of text ended by line feeds in 0x300 bytes.

/// The chat with eight empty lines.
pub const EMPTY: [u8; 8] = *b"\n\n\n\n\n\n\n\0";

/// The settings screen's items edited: 1 the player's name, 2 the most players.
pub const NAME: u32 = 1;
pub const MOST_PLAYERS: u32 = 2;
/// The settings screen's port item (below the host's address, above the last item).
pub const PORT: u32 = 4;
/// The port the network game uses when none is set.
pub const DEFAULT_PORT: u32 = 47624;

/// The port as typed: 0 (the default) up to 65535; a typed number past that is held at 65535.
pub fn port(typed: u32) -> u32 {
    typed.min(0xFFFF)
}

/// The port shown: the default's number for 0.
pub fn shown_port(port: u32) -> u32 {
    if port == 0 { DEFAULT_PORT } else { port }
}

/// The most players as typed, held to 2 .. 8 (signed), and to 4 with `notice_kind` 2.
pub fn most_players(typed: u32, notice_kind: u8) -> u32 {
    let n = (typed as i32).clamp(2, 8);
    (if notice_kind == 2 { n.min(4) } else { n }) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports() {
        assert_eq!((port(0), port(47624), port(70000)), (0, 47624, 0xFFFF));
        assert_eq!((shown_port(0), shown_port(5000)), (47624, 5000));
    }

    #[test]
    fn players() {
        assert_eq!(most_players(1, 0), 2);
        assert_eq!(most_players(9, 0), 8);
        assert_eq!(most_players(u32::MAX, 0), 2);
        assert_eq!(most_players(6, 2), 4);
        assert_eq!(most_players(3, 2), 3);
    }
}
