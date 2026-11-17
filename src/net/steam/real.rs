//! The Steam side, over Valve's library loaded at run time (`flat`). One Steam client for the
//! program, started the first time something asks for it; a thread of its own runs Steam's
//! callbacks and moves the relay connections' messages into their readers' buffers.

use super::callbacks::{self, id};
use super::flat::Flat;
use super::{PROVIDER, desc_from_data, desc_to_data, instance_lobby, keys, lobby_instance};
use crate::net::dp_session::wire::Inbox;
use crate::net::dp_session::{Desc, Session, Wire, protocol_tag};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

/// The application number the game runs under (Valve's Spacewar test application).
const APP_ID: u32 = 480;
/// How long Steam may take to answer a lobby or connection step.
const STEP_WAIT: Duration = Duration::from_secs(15);
/// The most players a lobby takes when the session names no limit.
const LOBBY_MOST: u32 = 16;
/// The most lobbies one search asks for.
const SEARCH_MOST: u32 = 50;

/// What a listen socket tells the host's thread.
enum ListenEvent {
    /// A machine asks to connect.
    Connecting(u32),
    /// A connection is up.
    Connected(u32),
}

struct Steam {
    flat: Flat,
    /// The relay connections the thread feeds.
    wires: Mutex<Vec<Weak<Link>>>,
    /// The lobbies the last search found (and when it ended).
    found: Mutex<Found>,
    /// A lobby the player was invited to, not yet listed.
    invited: Mutex<Option<u64>>,
    /// Where each listen socket's events go.
    listeners: Mutex<HashMap<u32, mpsc::Sender<ListenEvent>>>,
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
        // Test runs of the remote driver keep Steam's library out entirely.
        #[cfg(feature = "remote")]
        if std::env::var_os("RC2K_STEAM_OFF").is_some() {
            log("not available: RC2K_STEAM_OFF is set");
            return None;
        }
        let flat = match Flat::start(APP_ID) {
            Ok(flat) => flat,
            Err(e) => {
                log(&format!("not available: {e}"));
                return None;
            }
        };
        let steam = Steam {
            flat,
            wires: Mutex::new(Vec::new()),
            found: Mutex::new(Found::default()),
            invited: Mutex::new(super::connect_lobby_argument(std::env::args())),
            listeners: Mutex::new(HashMap::new()),
        };
        std::thread::spawn(pump);
        log("started");
        Some(steam)
    }

    /// Waits for what a Steam call reports through its answer.
    fn wait<T>(rx: &mpsc::Receiver<T>) -> Option<T> {
        rx.recv_timeout(STEP_WAIT).ok()
    }

    /// One of Steam's callbacks (not an answer to a call).
    fn callback(&self, number: i32, bytes: &[u8]) {
        match number {
            id::GAME_LOBBY_JOIN_REQUESTED => {
                if let Some(lobby) = callbacks::join_requested_lobby(bytes) {
                    *self
                        .invited
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(lobby);
                    // The menus take the player into the lobby (`net::invite`).
                    crate::net::invite::offer(crate::net::invite::Invite {
                        provider: PROVIDER,
                        instance: Some(lobby_instance(lobby)),
                    });
                }
            }
            id::CONNECTION_STATUS_CHANGED => {
                let Some(change) =
                    callbacks::connection_status_changed(bytes, callbacks::LARGE_PACK)
                else {
                    return;
                };
                let event = match change.state {
                    callbacks::state::CONNECTING => ListenEvent::Connecting(change.conn),
                    callbacks::state::CONNECTED => ListenEvent::Connected(change.conn),
                    _ => return,
                };
                if let Some(to) = self
                    .listeners
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(&change.listen_socket)
                {
                    let _ = to.send(event);
                }
            }
            _ => {}
        }
    }
}

/// Runs Steam's callbacks and passes the relay connections' messages on, for ever.
fn pump() {
    loop {
        if let Some(steam) = state() {
            steam
                .flat
                .run_frame(&mut |n, bytes| steam.callback(n, bytes));
            let mut wires = steam
                .wires
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            wires.retain(|w| w.upgrade().is_some_and(|link| link.pump()));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Whether Steam can be used: the library is there and its client runs and took the
/// application number.
pub fn available() -> bool {
    state().is_some()
}

/// A relay connection, closed when dropped.
struct Connection(u32);

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(steam) = state() {
            steam.flat.close(self.0);
        }
    }
}

/// A listen socket and its events; closed when dropped.
struct Listener {
    socket: u32,
    events: mpsc::Receiver<ListenEvent>,
}

impl Listener {
    fn open(steam: &Steam) -> Option<Listener> {
        let (tx, events) = mpsc::channel();
        // Held across the call: no event for the socket can be taken before it is registered.
        let mut listeners = steam
            .listeners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let socket = steam.flat.listen_p2p();
        if socket == 0 {
            return None;
        }
        listeners.insert(socket, tx);
        Some(Listener { socket, events })
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(steam) = state() {
            steam
                .listeners
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&self.socket);
            steam.flat.close_listen(self.socket);
        }
    }
}

// ---- the relay connection ----

/// One relay connection and the bytes it has brought.
struct Link {
    conn: Mutex<Option<Connection>>,
    inbox: Inbox,
    ended: AtomicBool,
    /// Leaves the lobby when the connection goes (a joined machine's).
    _lobby: Option<LobbyGuard>,
}

impl Link {
    /// Moves what has come into the inbox; false once the connection is over.
    fn pump(&self) -> bool {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (Some(c), Some(steam)) = (conn.as_ref(), state()) else {
            return false;
        };
        steam.flat.receive(c.0, &mut |m| self.inbox.put(m));
        let over = !steam.flat.state(c.0).is_some_and(callbacks::is_live);
        if over {
            // what arrived with the end is taken first
            steam.flat.receive(c.0, &mut |m| self.inbox.put(m));
            self.ended.store(true, Ordering::SeqCst);
            self.inbox.end();
        }
        !over
    }

    fn close(&self) {
        self.ended.store(true, Ordering::SeqCst);
        self.inbox.end();
        // dropping the connection closes it
        drop(
            self.conn
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
        );
    }
}

/// A handle on a relay connection, as the session's transport.
struct SteamWire {
    link: Arc<Link>,
    read_limit: Arc<Mutex<Option<Duration>>>,
}

impl SteamWire {
    fn new(conn: Connection, lobby: Option<LobbyGuard>) -> Self {
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
        let (Some(c), Some(steam)) = (conn.as_ref(), state()) else {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        };
        steam
            .flat
            .send(c.0, data)
            .map_err(|e| std::io::Error::other(format!("Steam result {e}")))?;
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
    lobby: u64,
    /// Stops the host's listening thread.
    stop: Arc<AtomicBool>,
}

impl Drop for LobbyGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(steam) = state() {
            steam.flat.leave_lobby(self.lobby);
        }
        status(None);
    }
}

/// Makes the host's lobby (public, so the list shows it) and listens for the machines that
/// connect through it; the session keeps both until it closes. `password` says the session is
/// locked, which the list shows.
pub fn host(session: &Arc<Session>, desc: &Desc, password: bool) -> Result<(), String> {
    let steam = state().ok_or("Steam is not running")?;
    let (tx, rx) = mpsc::channel();
    let most = if desc.max_players == 0 {
        LOBBY_MOST
    } else {
        desc.max_players.min(250)
    };
    steam.flat.create_lobby(
        most,
        Box::new(move |answer| {
            let made = match answer.and_then(|b| callbacks::lobby_created(b, callbacks::LARGE_PACK))
            {
                Some(Ok(lobby)) => Ok(lobby),
                Some(Err(result)) => Err(format!("Steam result {result}")),
                None => Err("no usable answer".to_string()),
            };
            let _ = tx.send(made);
        }),
    );
    let lobby = Steam::wait(&rx)
        .ok_or("Steam did not answer")?
        .map_err(|e| format!("the lobby was not made: {e}"))?;
    let mut data = desc_to_data(desc);
    data.push((keys::VERSION, protocol_tag().to_string()));
    data.push(("locked", u8::from(password).to_string()));
    for (key, value) in data {
        steam.flat.set_lobby_data(lobby, key, &value);
    }
    let listener =
        Listener::open(steam).ok_or_else(|| "the listening socket was not made".to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let (flag, weak) = (stop.clone(), Arc::downgrade(session));
    std::thread::spawn(move || {
        while !flag.load(Ordering::SeqCst) {
            while let Ok(event) = listener.events.try_recv() {
                let Some(steam) = state() else { return };
                match event {
                    ListenEvent::Connecting(conn) => {
                        if !steam.flat.accept(conn) {
                            log("a connection could not be accepted");
                            steam.flat.close(conn);
                        }
                    }
                    ListenEvent::Connected(conn) => {
                        let wire = SteamWire::new(Connection(conn), None);
                        match weak.upgrade() {
                            Some(session) => session.serve_wire(Box::new(wire)),
                            None => return,
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    status_in_lobby(lobby, true);
    log(&format!("lobby {lobby} made"));
    session.hold(Box::new(LobbyGuard { lobby, stop }));
    Ok(())
}

/// Rich presence: the status line, and the argument Steam gives a friend's "join game".
fn status_in_lobby(lobby: u64, hosting: bool) {
    if let Some(steam) = state() {
        steam
            .flat
            .set_rich_presence("connect", Some(&format!("+connect_lobby {lobby}")));
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
        steam.flat.set_rich_presence("status", text);
        if text.is_none() {
            steam.flat.clear_rich_presence();
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
        let friends: Vec<u64> = steam
            .flat
            .friends_in_games()
            .into_iter()
            .filter(|&(game, _)| game & 0x00FF_FFFF == u64::from(APP_ID))
            .map(|(_, lobby)| lobby)
            .filter(|&l| l != 0)
            .collect();
        steam.flat.request_lobby_list(
            keys::VERSION,
            protocol_tag(),
            SEARCH_MOST,
            Box::new(move |answer| {
                let Some(steam) = state() else { return };
                let count = answer.and_then(callbacks::lobby_match_count).unwrap_or(0);
                let mut found = steam
                    .found
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                found.lobbies = (0..count).map(|i| steam.flat.lobby_by_index(i)).collect();
                for friend in friends {
                    if !found.lobbies.contains(&friend) {
                        found.lobbies.push(friend);
                    }
                }
                found.asked = Some(Instant::now());
                found.searching = false;
            }),
        );
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
        .filter_map(|lobby| {
            desc_from_data(
                |key| steam.flat.lobby_data(lobby, key),
                protocol_tag(),
                steam.flat.member_count(lobby),
                lobby,
                application,
            )
            .map(|mut d| {
                if steam.flat.lobby_data(lobby, "locked").as_deref() == Some("1") {
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
    let (tx, rx) = mpsc::channel();
    steam.flat.join_lobby(
        lobby,
        Box::new(move |answer| {
            let _ = tx.send(answer.and_then(callbacks::lobby_entered).unwrap_or(false));
        }),
    );
    match Steam::wait(&rx) {
        Some(true) => {}
        Some(false) => return Err(DPERR_NOCONNECTION),
        None => return Err(DPERR_TIMEOUT),
    }
    // From here the guard leaves the lobby again on any failure.
    let guard = LobbyGuard {
        lobby,
        stop: Arc::new(AtomicBool::new(true)),
    };
    let owner = steam.flat.lobby_owner(lobby);
    let conn = steam.flat.connect_p2p(owner);
    if conn == 0 {
        return Err(DPERR_NOCONNECTION);
    }
    // closes the connection on any failure below
    let conn = Connection(conn);
    let deadline = Instant::now() + STEP_WAIT;
    loop {
        match steam.flat.state(conn.0) {
            Some(callbacks::state::CONNECTED) => break,
            Some(callbacks::state::CONNECTING | callbacks::state::FINDING_ROUTE) => {}
            _ => return Err(DPERR_NOCONNECTION),
        }
        if Instant::now() > deadline {
            return Err(DPERR_TIMEOUT);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    status_in_lobby(lobby, false);
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
