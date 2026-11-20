//! What Steam's callbacks and call results carry, read out of their bytes: the layouts of the
//! Steamworks SDK this program is written against (see `flat`), kept apart from the library so
//! they are tested everywhere. A structure's packing depends on the platform (4 bytes on Linux
//! and macOS, 8 on Windows), so each reader takes it as `large`.

/// Callback numbers (the SDK's `k_iCallback` values). The answers of lobby calls
/// (`LobbyCreated_t`, `LobbyEnter_t`, `LobbyMatchList_t`) need none: they come to the handler of
/// the call that asked.
pub mod id {
    /// `SteamAPICallCompleted_t`: an asynchronous call has an answer.
    pub const API_CALL_COMPLETED: i32 = 703;
    /// `GameLobbyJoinRequested_t`: the player chose "join game" for a lobby.
    pub const GAME_LOBBY_JOIN_REQUESTED: i32 = 333;
    /// `SteamNetConnectionStatusChangedCallback_t`.
    pub const CONNECTION_STATUS_CHANGED: i32 = 1221;
}

/// Whether this platform's callback structures are packed to 8 bytes (Windows) and not 4.
pub(super) const LARGE_PACK: bool = !cfg!(any(target_os = "linux", target_os = "macos"));

/// `ESteamNetworkingConnectionState`.
pub mod state {
    pub const CONNECTING: i32 = 1;
    pub const FINDING_ROUTE: i32 = 2;
    pub const CONNECTED: i32 = 3;
}

/// `EResult::k_EResultOK`.
pub(super) const RESULT_OK: i32 = 1;
/// `EChatRoomEnterResponse::k_EChatRoomEnterResponseSuccess`.
const ENTER_SUCCESS: u32 = 1;

/// Where `SteamNetConnectionInfo_t` starts inside a connection-status callback, and its fields'
/// offsets and size.
const INFO_LISTEN_SOCKET: usize = 144;
const INFO_STATE: usize = 176;
pub const INFO_SIZE: usize = 696;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// An answer to an asynchronous call is ready: the call, the callback its answer is, the
/// answer's size.
pub(super) fn api_call_completed(b: &[u8]) -> Option<(u64, i32, u32)> {
    Some((u64_at(b, 0)?, u32_at(b, 8)? as i32, u32_at(b, 12)?))
}

/// The lobby `LobbyCreated_t` reports, or the `EResult` that says why not.
pub(super) fn lobby_created(b: &[u8], large: bool) -> Option<Result<u64, i32>> {
    let result = u32_at(b, 0)? as i32;
    if result != RESULT_OK {
        return Some(Err(result));
    }
    Some(Ok(u64_at(b, if large { 8 } else { 4 })?))
}

/// Whether `LobbyEnter_t` says the lobby was entered.
pub(super) fn lobby_entered(b: &[u8]) -> Option<bool> {
    Some(u32_at(b, 16)? == ENTER_SUCCESS)
}

/// How many lobbies `LobbyMatchList_t` found.
pub(super) fn lobby_match_count(b: &[u8]) -> Option<u32> {
    u32_at(b, 0)
}

/// The lobby `GameLobbyJoinRequested_t` names.
pub(super) fn join_requested_lobby(b: &[u8]) -> Option<u64> {
    u64_at(b, 0)
}

/// A relay connection's change: the connection, the listen socket it came in through (0 for
/// one the program made), and its new state.
#[derive(Debug, PartialEq, Eq)]
pub struct StatusChange {
    pub conn: u32,
    pub listen_socket: u32,
    pub state: i32,
}

pub(super) fn connection_status_changed(b: &[u8], large: bool) -> Option<StatusChange> {
    let info = if large { 8 } else { 4 };
    if b.len() < info + INFO_STATE + 4 {
        return None;
    }
    Some(StatusChange {
        conn: u32_at(b, 0)?,
        listen_socket: u32_at(b, info + INFO_LISTEN_SOCKET)?,
        state: u32_at(b, info + INFO_STATE)? as i32,
    })
}

/// The state in a `SteamNetConnectionInfo_t`.
pub(super) fn info_state(info: &[u8]) -> Option<i32> {
    Some(u32_at(info, INFO_STATE)? as i32)
}

/// Whether a connection in this state is still usable or on its way.
pub(super) fn is_live(state: i32) -> bool {
    matches!(
        state,
        state::CONNECTING | state::FINDING_ROUTE | state::CONNECTED
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_completed_call_is_read() {
        let mut b = vec![];
        b.extend(0x1122_3344_5566_7788u64.to_le_bytes());
        b.extend(513i32.to_le_bytes());
        b.extend(16u32.to_le_bytes());
        assert_eq!(
            api_call_completed(&b),
            Some((0x1122_3344_5566_7788, 513, 16))
        );
        assert_eq!(api_call_completed(&b[..12]), None);
    }

    #[test]
    fn a_made_lobby_is_at_the_packing_offset() {
        let mut small = vec![];
        small.extend(1i32.to_le_bytes());
        small.extend(77u64.to_le_bytes());
        assert_eq!(lobby_created(&small, false), Some(Ok(77)));
        let mut large = vec![];
        large.extend(1i32.to_le_bytes());
        large.extend([0; 4]);
        large.extend(77u64.to_le_bytes());
        assert_eq!(lobby_created(&large, true), Some(Ok(77)));
        let mut failed = vec![];
        failed.extend(16i32.to_le_bytes());
        failed.extend([0; 12]);
        assert_eq!(lobby_created(&failed, false), Some(Err(16)));
        assert_eq!(lobby_created(&small[..8], false), None);
    }

    #[test]
    fn entering_a_lobby_needs_the_success_response() {
        let mut b = vec![0u8; 20];
        b[16] = 1;
        assert_eq!(lobby_entered(&b), Some(true));
        b[16] = 5;
        assert_eq!(lobby_entered(&b), Some(false));
        assert_eq!(lobby_entered(&b[..8]), None);
    }

    #[test]
    fn a_status_change_is_read_at_both_packings() {
        for (large, info) in [(false, 4usize), (true, 8)] {
            let mut b = vec![0u8; info + INFO_SIZE + 4];
            b[..4].copy_from_slice(&9u32.to_le_bytes());
            b[info + INFO_LISTEN_SOCKET..][..4].copy_from_slice(&5u32.to_le_bytes());
            b[info + INFO_STATE..][..4].copy_from_slice(&3i32.to_le_bytes());
            assert_eq!(
                connection_status_changed(&b, large),
                Some(StatusChange {
                    conn: 9,
                    listen_socket: 5,
                    state: state::CONNECTED
                })
            );
            assert_eq!(connection_status_changed(&b[..100], large), None);
        }
    }

    #[test]
    fn states_are_live_until_the_connection_ends() {
        assert!(is_live(state::CONNECTING));
        assert!(is_live(state::CONNECTED));
        assert!(!is_live(4));
        assert!(!is_live(5));
        assert!(!is_live(0));
        let mut info = vec![0u8; INFO_SIZE];
        info[INFO_STATE] = 3;
        assert_eq!(info_state(&info), Some(3));
    }
}
