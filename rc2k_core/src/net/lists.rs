//! The lists DirectPlay's enumerations fill: the connections' and sessions' tables (records of 0x54
//! and 0x68 bytes, 0x100 at most) and the players' blocks (eight).

/// The most connections and sessions listed.
pub const MOST_LISTED: u32 = 0x100;
/// The most players listed.
pub const MOST_PLAYERS: u32 = 8;

/// The tables' record bytes: connections, sessions.
pub(crate) const CONNECTION_LEN: u32 = 0x54;
pub(crate) const SESSION_LEN: u32 = 0x68;

/// The fatal errors of a record's number past its list, and of a table not taken.
pub const NO_CONNECTION: u32 = 0x80A6_1010;
pub const NO_CONNECTIONS: u32 = 0x80A6_1011;
pub const NO_SESSION: u32 = 0x80A6_1030;
pub const NO_SESSIONS: u32 = 0x80A6_1031;

/// Why a record cannot be had: a fatal error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Missing(pub u32);

/// Record `index` (unsigned) of a table of `len`-byte records at `table` holding `count`:
/// its address. A number not below the count is the first fatal error (`codes`), a null
/// table (asked only then) the second.
pub fn record(
    index: u32,
    count: u32,
    table: impl FnOnce() -> u32,
    len: u32,
    codes: [u32; 2],
) -> Result<u32, Missing> {
    if index >= count {
        return Err(Missing(codes[0]));
    }
    let table = table();
    if table == 0 {
        return Err(Missing(codes[1]));
    }
    Ok(index.wrapping_mul(len).wrapping_add(table))
}

/// What a session found does to the enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    /// The enumeration stops (a time-out, no session, or the list full).
    Stop,
    /// The session is passed over (its `dwUser2` bit 0 clear: not the game's).
    Pass,
    /// The session is listed as number `n`.
    List(u32),
}

/// A session found: a time-out (flag 1) or none stops; one whose `dwUser2` bit 0 is clear
/// (`user2`, asked only for a session) is passed over; else listed while there is room
/// (`listed`, asked only then).
pub fn session_found(
    flags: u32,
    session: bool,
    user2: impl FnOnce() -> u32,
    listed: impl FnOnce() -> u32,
) -> Found {
    if flags & 1 != 0 || !session {
        return Found::Stop;
    }
    if user2() & 1 == 0 {
        return Found::Pass;
    }
    let listed = listed();
    if listed >= MOST_LISTED {
        return Found::Stop;
    }
    Found::List(listed)
}

/// A player found: listed as number `n` while fewer than eight are, else the enumeration
/// stops.
pub fn player_found(listed: u32) -> Option<u32> {
    (listed < MOST_PLAYERS).then_some(listed)
}

/// The fatal errors when the network's memory for the tables cannot be had: connections,
/// sessions.
pub(crate) const NO_CONNECTION_MEMORY: u32 = 0x80E6_8021;
pub(crate) const NO_SESSION_MEMORY: u32 = 0x80E6_8022;

/// The two tables taken (`allocate_lists`), in order: their bytes and the fatal error of
/// their memory not had.
pub const TABLES: [(u32, u32); 2] = [
    (MOST_LISTED * CONNECTION_LEN, NO_CONNECTION_MEMORY),
    (MOST_LISTED * SESSION_LEN, NO_SESSION_MEMORY),
];

/// The notice kind (the game's edition) whose connections leave out two service
/// providers (`connection_found`).
pub const LEAVES_OUT_PROVIDERS: u8 = 2;

/// A connection found with `count` listed: its number, or `None` when the list is full
/// (the enumeration stops).
pub fn connection_listed(count: u32) -> Option<u32> {
    (count < MOST_LISTED).then_some(count)
}

/// The first of `count` listed that `same` says is the one (asked in order; none with
/// none listed; the count compared with `==`).
pub fn find_listed<E>(
    count: u32,
    mut same: impl FnMut(u32) -> Result<bool, E>,
) -> Result<Option<u32>, E> {
    if count == 0 {
        return Ok(None);
    }
    let mut n = 0u32;
    loop {
        if same(n)? {
            return Ok(Some(n));
        }
        n = n.wrapping_add(1);
        if n == count {
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing() {
        assert_eq!(TABLES, [(0x5400, 0x80E6_8021), (0x6800, 0x80E6_8022)]);
        assert_eq!(connection_listed(0xFF), Some(0xFF));
        assert_eq!(connection_listed(0x100), None);
        let ok = |n: u32| -> Result<bool, ()> { Ok(n == 2) };
        assert_eq!(find_listed(4, ok), Ok(Some(2)));
        assert_eq!(find_listed(2, ok), Ok(None));
        assert_eq!(find_listed(0, ok), Ok(None));
        assert_eq!(find_listed(1, |_| Err::<bool, u8>(7)), Err(7));
    }

    #[test]
    fn records() {
        let codes = [NO_SESSION, NO_SESSIONS];
        assert_eq!(record(2, 3, || 0x1000, SESSION_LEN, codes), Ok(0x10D0));
        assert_eq!(
            record(3, 3, || 0x1000, SESSION_LEN, codes),
            Err(Missing(NO_SESSION))
        );
        assert_eq!(
            record(0, 3, || 0, SESSION_LEN, codes),
            Err(Missing(NO_SESSIONS))
        );
        assert_eq!(
            record(u32::MAX, 0, || panic!(), 1, codes),
            Err(Missing(NO_SESSION))
        );
    }

    #[test]
    fn found() {
        assert_eq!(session_found(1, true, || 1, || 0), Found::Stop);
        assert_eq!(session_found(0, false, || panic!(), || 0), Found::Stop);
        assert_eq!(session_found(0, true, || 2, || 0), Found::Pass);
        assert_eq!(session_found(0, true, || 1, || 5), Found::List(5));
        assert_eq!(session_found(0, true, || 1, || 0x100), Found::Stop);
        assert_eq!(player_found(7), Some(7));
        assert_eq!(player_found(8), None);
    }
}
