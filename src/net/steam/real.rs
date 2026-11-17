//! The Steam side, over the `steamworks` crate. One Steam client for the program, started the
//! first time something asks for it; a thread of its own runs Steam's callbacks and moves the
//! relay connections' messages into their readers' buffers.

use super::{desc_from_data, desc_to_data, instance_lobby, keys};
use crate::net::dp_session::wire::Inbox;
use crate::net::dp_session::{Desc, Session, Wire, protocol_tag};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};
use steamworks::networking_types::{
    AppNetConnectionEnd, ListenSocketEvent, NetConnectionEnd, NetworkingConnectionState,
    NetworkingIdentity, SendFlags,
};
use steamworks::{
    CallbackHandle, Client, DistanceFilter, FriendFlags, GameLobbyJoinRequested, LobbyId, LobbyKey,
    LobbyType, StringFilter, StringFilterKind,
};

/// The application number the game runs under (Valve's Spacewar test application).
const APP_ID: u32 = 480;
/// How long Steam may take to answer a lobby or connection step.
const STEP_WAIT: Duration = Duration::from_secs(15);
/// The most players a lobby takes when the session names no limit.
const LOBBY_MOST: u32 = 16;

struct Steam {
    client: Client,
    /// The relay connections the thread feeds.
    wires: Mutex<Vec<Weak<Link>>>,
    /// The lobbies the last search found (and when it ended).
    found: Mutex<Found>,
    /// A lobby the player was invited to, not yet listed.
    invited: Mutex<Option<u64>>,
    _callbacks: Mutex<Vec<CallbackHandle>>,
}

#[derive(Default)]
struct Found {
    lobbies: Vec<u64>,
    asked: Option<Instant>,
    searching: bool,
}

fn log(text: &str) {
    crate::logfile::program_log::log(&format!("net: steam: {text}"));
}

fn state() -> Option<&'static Steam> {
    static STEAM: OnceLock<Option<Steam>> = OnceLock::new();
    STEAM.get_or_init(Steam::start).as_ref()
}

impl Steam {
    fn start() -> Option<Steam> {
        let client = match Client::init_app(APP_ID) {
            Ok(client) => client,
            Err(e) => {
                log(&format!("not available: {e}"));
                return None;
            }
        };
        client.networking_utils().init_relay_network_access();
        let invited = Mutex::new(super::connect_lobby_argument(std::env::args()));
        let steam = Steam {
            client,
            wires: Mutex::new(Vec::new()),
            found: Mutex::new(Found::default()),
            invited,
            _callbacks: Mutex::new(Vec::new()),
        };
        let handle = steam
            .client
            .register_callback(|joined: GameLobbyJoinRequested| {
                if let Some(steam) = state() {
                    *steam
                        .invited
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) =
                        Some(joined.lobby_steam_id.raw());
                }
            });
        steam
            ._callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(handle);
        let client = steam.client.clone();
        std::thread::spawn(move || pump(&client));
        log("started");
        Some(steam)
    }

    /// Waits for what a Steam call reports through its callback.
    fn wait<T>(rx: &mpsc::Receiver<T>) -> Option<T> {
        rx.recv_timeout(STEP_WAIT).ok()
    }
}

/// Runs Steam's callbacks and passes the relay connections' messages on, for ever.
fn pump(client: &Client) {
    loop {
        client.run_callbacks();
        if let Some(steam) = state() {
            let mut wires = steam
                .wires
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            wires.retain(|w| w.upgrade().is_some_and(|link| link.pump()));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Whether Steam can be used: its client runs and took the application number.
pub fn available() -> bool {
    state().is_some()
}

// ---- the relay connection ----

/// One relay connection and the bytes it has brought.
struct Link {
    conn: Mutex<Option<steamworks::networking_sockets::NetConnection>>,
    inbox: Inbox,
    ended: AtomicBool,
    /// Leaves the lobby when the connection goes (a joined machine's).
    _lobby: Option<LobbyGuard>,
}

impl Link {
    /// Moves what has come into the inbox; false once the connection is over.
    fn pump(&self) -> bool {
        let mut conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(c) = conn.as_mut() else {
            return false;
        };
        c.receive_messages_with(|m| self.inbox.put(m.data()));
        let over = !matches!(
            c.info().ok().and_then(|i| i.state().ok()),
            Some(
                NetworkingConnectionState::Connected
                    | NetworkingConnectionState::Connecting
                    | NetworkingConnectionState::FindingRoute
            )
        );
        if over {
            // what arrived with the end is taken first
            c.receive_messages_with(|m| self.inbox.put(m.data()));
            self.ended.store(true, Ordering::SeqCst);
            self.inbox.end();
        }
        !over
    }

    fn close(&self) {
        self.ended.store(true, Ordering::SeqCst);
        self.inbox.end();
        let taken = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(c) = taken {
            c.close(
                NetConnectionEnd::App(AppNetConnectionEnd::generic_normal()),
                Some("closed"),
                true,
            );
        }
    }
}

/// A handle on a relay connection, as the session's transport.
struct SteamWire {
    link: Arc<Link>,
    read_limit: Arc<Mutex<Option<Duration>>>,
}

impl SteamWire {
    fn new(conn: steamworks::networking_sockets::NetConnection, lobby: Option<LobbyGuard>) -> Self {
        let link = Arc::new(Link {
            conn: Mutex::new(Some(conn)),
            inbox: Inbox::default(),
            ended: AtomicBool::new(false),
            _lobby: lobby,
        });
        if let Some(steam) = state() {
            steam
                .wires
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(Arc::downgrade(&link));
        }
        SteamWire {
            link,
            read_limit: Arc::new(Mutex::new(None)),
        }
    }
}

impl Read for SteamWire {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let limit = *self
            .read_limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.link.inbox.take(out, limit)
    }
}

impl Write for SteamWire {
    /// One write is one reliable message: the session writes a frame at a time.
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let conn = self
            .link
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(c) = conn.as_ref() else {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        };
        c.send_message(data, SendFlags::RELIABLE_NO_NAGLE)
            .map_err(|e| std::io::Error::other(format!("{e}")))?;
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Wire for SteamWire {
    fn try_clone(&self) -> std::io::Result<Box<dyn Wire>> {
        Ok(Box::new(SteamWire {
            link: self.link.clone(),
            read_limit: self.read_limit.clone(),
        }))
    }

    fn shutdown(&self) -> std::io::Result<()> {
        self.link.close();
        Ok(())
    }

    fn set_read_timeout(&self, limit: Option<Duration>) -> std::io::Result<()> {
        *self
            .read_limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = limit;
        Ok(())
    }

    fn set_write_timeout(&self, _limit: Option<Duration>) -> std::io::Result<()> {
        Ok(())
    }
}

// ---- lobbies ----

/// Leaves the lobby and clears the rich presence when dropped.
struct LobbyGuard {
    lobby: LobbyId,
    /// Stops the host's listening thread.
    stop: Arc<AtomicBool>,
}

impl Drop for LobbyGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(steam) = state() {
            steam.client.matchmaking().leave_lobby(self.lobby);
        }
        status(None);
    }
}

/// Makes the host's lobby (public, so the list shows it) and listens for the machines that
/// connect through it; the session keeps both until it closes. `password` says the session is
/// locked, which the list shows.
pub fn host(session: &Arc<Session>, desc: &Desc, password: bool) -> Result<(), String> {
    let steam = state().ok_or("Steam is not running")?;
    let mm = steam.client.matchmaking();
    let (tx, rx) = mpsc::channel();
    let most = if desc.max_players == 0 {
        LOBBY_MOST
    } else {
        desc.max_players.min(250)
    };
    mm.create_lobby(LobbyType::Public, most, move |r| {
        let _ = tx.send(r);
    });
    let lobby = Steam::wait(&rx)
        .ok_or("Steam did not answer")?
        .map_err(|e| format!("the lobby was not made: {e}"))?;
    let mut data = desc_to_data(desc);
    data.push((keys::VERSION, protocol_tag().to_string()));
    data.push(("locked", u8::from(password).to_string()));
    for (key, value) in data {
        mm.set_lobby_data(lobby, key, &value);
    }
    let listener = steam
        .client
        .networking_sockets()
        .create_listen_socket_p2p(0, [])
        .map_err(|_| "the listening socket was not made".to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let (flag, weak) = (stop.clone(), Arc::downgrade(session));
    std::thread::spawn(move || {
        while !flag.load(Ordering::SeqCst) {
            while let Some(event) = listener.try_receive_event() {
                match event {
                    ListenSocketEvent::Connecting(request) => {
                        if request.accept().is_err() {
                            log("a connection could not be accepted");
                        }
                    }
                    ListenSocketEvent::Connected(connected) => {
                        let wire = SteamWire::new(connected.take_connection(), None);
                        match weak.upgrade() {
                            Some(session) => session.serve_wire(Box::new(wire)),
                            None => return,
                        }
                    }
                    ListenSocketEvent::Disconnected(_) => {}
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    status_in_lobby(lobby, true);
    log(&format!("lobby {} made", lobby.raw()));
    session.hold(Box::new(LobbyGuard { lobby, stop }));
    Ok(())
}

/// Rich presence: the status line, and the argument Steam gives a friend's "join game".
fn status_in_lobby(lobby: LobbyId, hosting: bool) {
    if let Some(steam) = state() {
        let friends = steam.client.friends();
        friends.set_rich_presence("connect", Some(&format!("+connect_lobby {}", lobby.raw())));
    }
    status(Some(if hosting {
        "Hosting a network race"
    } else {
        "In a network race"
    }));
}

/// The rich presence "status" text, or cleared with `None`.
fn status(text: Option<&str>) {
    if let Some(steam) = state() {
        let friends = steam.client.friends();
        friends.set_rich_presence("status", text);
        if text.is_none() {
            friends.clear_rich_presence();
        }
    }
}

/// The sessions Steam knows of for `application`: the lobbies of the last search that carry
/// this protocol's version, friends' lobbies, and one the player was invited to. A new search
/// starts (at most every three seconds); its answer is for the next call, as the TCP finder's.
pub fn lobbies(application: [u8; 16]) -> Vec<Desc> {
    let Some(steam) = state() else {
        return vec![];
    };
    let mm = steam.client.matchmaking();
    let start = {
        let mut found = steam
            .found
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let due = !found.searching
            && found
                .asked
                .is_none_or(|t| t.elapsed() > Duration::from_secs(3));
        if due {
            found.searching = true;
        }
        due
    };
    if start {
        mm.add_request_lobby_list_string_filter(StringFilter(
            LobbyKey::new(keys::VERSION),
            protocol_tag(),
            StringFilterKind::Equal,
        ));
        mm.set_request_lobby_list_distance_filter(DistanceFilter::Worldwide);
        mm.set_request_lobby_list_result_count_filter(50);
        let friends: Vec<u64> = steam
            .client
            .friends()
            .get_friends(FriendFlags::IMMEDIATE)
            .into_iter()
            .filter_map(|f| f.game_played())
            .filter(|g| g.game.raw() & 0x00FF_FFFF == u64::from(APP_ID))
            .map(|g| g.lobby.raw())
            .filter(|&l| l != 0)
            .collect();
        mm.request_lobby_list(move |result| {
            if let Some(steam) = state() {
                let mut found = steam
                    .found
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                found.lobbies = result
                    .map(|ids| ids.into_iter().map(|l| l.raw()).collect())
                    .unwrap_or_default();
                for friend in friends {
                    if !found.lobbies.contains(&friend) {
                        found.lobbies.push(friend);
                    }
                }
                found.asked = Some(Instant::now());
                found.searching = false;
            }
        });
    }
    let mut ids = steam
        .found
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .lobbies
        .clone();
    if let Some(invited) = *steam
        .invited
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        && !ids.contains(&invited)
    {
        ids.push(invited);
    }
    ids.into_iter()
        .filter_map(|id| {
            let lobby = LobbyId::from_raw(id);
            desc_from_data(
                |key| mm.lobby_data(lobby, key).filter(|v| !v.is_empty()),
                protocol_tag(),
                mm.lobby_member_count(lobby) as u32,
                id,
                application,
            )
            .map(|mut d| {
                if mm.lobby_data(lobby, "locked").as_deref() == Some("1") {
                    d.flags |= crate::net::dp_session::DPSESSION_PASSWORDREQUIRED;
                }
                d
            })
        })
        .collect()
}

/// Joins the lobby `instance` stands for and connects to its owner through Steam's relay.
pub fn join(instance: &[u8; 16]) -> Result<Box<dyn Wire>, u32> {
    use crate::net::dp_session::{DPERR_NOCONNECTION, DPERR_TIMEOUT};
    let lobby = instance_lobby(instance).ok_or(DPERR_NOCONNECTION)?;
    let steam = state().ok_or(DPERR_NOCONNECTION)?;
    let mm = steam.client.matchmaking();
    let id = LobbyId::from_raw(lobby);
    let (tx, rx) = mpsc::channel();
    mm.join_lobby(id, move |r| {
        let _ = tx.send(r);
    });
    match Steam::wait(&rx) {
        Some(Ok(_)) => {}
        Some(Err(())) => return Err(DPERR_NOCONNECTION),
        None => return Err(DPERR_TIMEOUT),
    }
    // From here the guard leaves the lobby again on any failure.
    let guard = LobbyGuard {
        lobby: id,
        stop: Arc::new(AtomicBool::new(true)),
    };
    let owner = mm.lobby_owner(id);
    let conn = steam
        .client
        .networking_sockets()
        .connect_p2p(NetworkingIdentity::new_steam_id(owner), 0, [])
        .map_err(|_| DPERR_NOCONNECTION)?;
    let deadline = Instant::now() + STEP_WAIT;
    loop {
        match conn.info().ok().and_then(|i| i.state().ok()) {
            Some(NetworkingConnectionState::Connected) => break,
            Some(
                NetworkingConnectionState::Connecting | NetworkingConnectionState::FindingRoute,
            ) => {}
            _ => return Err(DPERR_NOCONNECTION),
        }
        if Instant::now() > deadline {
            return Err(DPERR_TIMEOUT);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    status_in_lobby(id, false);
    Ok(Box::new(SteamWire::new(conn, Some(guard))))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With a Steam client running and signed in: Steam starts, a lobby is made with a
    /// session's data and read back. Run by hand (`--ignored`): it shows the account as playing
    /// Spacewar for a moment.
    #[test]
    #[ignore = "needs a running, signed-in Steam client"]
    fn a_lobby_is_made_and_read_back() {
        if !available() {
            eprintln!("Steam is not available here");
            return;
        }
        let session = Session::host_over(
            Desc {
                name: "rc2k test".into(),
                max_players: 4,
                ..Desc::default()
            },
            0,
            Arc::new(|_| {}),
            b"",
        );
        host(&session, &session.world().desc, false).expect("a lobby");
        session.close();
    }
}
