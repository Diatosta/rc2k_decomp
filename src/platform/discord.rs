//! The status in the Discord app (Rich Presence), over Discord's local IPC: a Unix socket
//! (`discord-ipc-0` .. `9`, in the runtime or temporary folder, or under the Flatpak's and the
//! Snap's) or, on Windows, the named pipe of the same name. Only the Discord app on this machine
//! is talked to; no token or secret is involved, the application's public number names the
//! status's owner.
//!
//! The game's thread only posts the status it wants (`post`); a thread of its own holds the
//! connection, sends what changed at most every couple of seconds (the latest wins), looks for
//! Discord again every fifteen seconds while it is not there and drops every error silently
//! (one log line when it connects and one when it loses Discord). The texts and frames are
//! `rc2k_core::discord`'s.

use rc2k_core::discord::{
    Activity, MAX_REPLY, OP_FRAME, OP_HANDSHAKE, encode_frame, handshake_json, parse_header,
    set_activity_json,
};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The public number of the "Rally Championship" application in Discord's developer portal (an
/// application's number is not a secret). `discord_application_id` in `rc2k.toml` replaces it;
/// empty, the status is not set.
pub const APPLICATION_ID: &str = "1574439577329074347";

/// How long to wait before looking for Discord again.
const RETRY: Duration = Duration::from_secs(15);
/// The least time between two sends.
const GAP: Duration = Duration::from_secs(2);
/// How long a reply of Discord's is waited for.
#[cfg(unix)]
const REPLY_WAIT: Duration = Duration::from_secs(5);

static ENABLED: AtomicBool = AtomicBool::new(true);
static ID_OVERRIDE: Mutex<Option<String>> = Mutex::new(None);
static CLIENT: OnceLock<Client> = OnceLock::new();

/// The options' two keys (`display.discord_status`, `display.discord_application_id`).
pub fn configure(enabled: bool, id: Option<String>) {
    ENABLED.store(enabled, Relaxed);
    *ID_OVERRIDE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = id;
}

pub fn enabled() -> bool {
    ENABLED.load(Relaxed)
}

pub fn id_override() -> Option<String> {
    ID_OVERRIDE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// The application's number now in use: the override's, else the built-in one.
fn application_id() -> String {
    id_override()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| APPLICATION_ID.to_owned())
}

/// Whether the status is wanted and has an application to belong to.
pub fn wanted() -> bool {
    enabled() && !application_id().is_empty()
}

/// Asks for the status `activity` (none: no status). Cheap; the connection is another thread's.
pub fn post(activity: Option<Activity>) {
    if !wanted() {
        return;
    }
    let client = CLIENT
        .get_or_init(|| Client::start(application_id(), socket_dirs(), RETRY, GAP, "discord"));
    client.post(activity);
}

/// The folders the sockets may be in (Windows has pipes by name instead).
#[cfg(unix)]
fn socket_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut add = |dir: PathBuf| {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    };
    if let Some(run) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from) {
        add(run.clone());
        add(run.join("app/com.discordapp.Discord"));
        add(run.join("snap.discord"));
    }
    if let Some(tmp) = std::env::var_os("TMPDIR").map(PathBuf::from) {
        add(tmp);
    }
    add(PathBuf::from("/tmp"));
    dirs
}

#[cfg(not(unix))]
fn socket_dirs() -> Vec<PathBuf> {
    Vec::new()
}

trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

/// A connection to Discord's IPC: the first `discord-ipc-N` that answers.
fn open(dirs: &[PathBuf]) -> Option<Box<dyn Stream>> {
    for n in 0..10 {
        #[cfg(unix)]
        for dir in dirs {
            let path = dir.join(format!("discord-ipc-{n}"));
            if let Ok(stream) = std::os::unix::net::UnixStream::connect(path) {
                let _ = stream.set_read_timeout(Some(REPLY_WAIT));
                let _ = stream.set_write_timeout(Some(REPLY_WAIT));
                return Some(Box::new(stream));
            }
        }
        #[cfg(windows)]
        {
            let _ = dirs;
            let path = format!(r"\\.\pipe\discord-ipc-{n}");
            if let Ok(file) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
            {
                return Some(Box::new(file));
            }
        }
    }
    None
}

/// Sends a frame and reads (and drops) Discord's reply.
fn exchange(stream: &mut dyn Stream, op: u32, json: &str) -> std::io::Result<()> {
    stream.write_all(&encode_frame(op, json))?;
    stream.flush()?;
    let mut header = [0u8; 8];
    stream.read_exact(&mut header)?;
    let (_, len) = parse_header(header);
    if len > MAX_REPLY {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    let mut body = vec![0u8; len as usize];
    stream.read_exact(&mut body)?;
    Ok(())
}

struct State {
    wanted: Option<Activity>,
    posted: bool,
    closed: bool,
}

struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

/// The status thread and what the game's thread hands it.
pub struct Client {
    shared: Arc<Shared>,
}

impl Client {
    /// Starts the thread for application `id`, looking for Discord in `dirs`. `retry` and `gap`
    /// are the waits described above; `log_name` begins the log lines.
    pub fn start(
        id: String,
        dirs: Vec<PathBuf>,
        retry: Duration,
        gap: Duration,
        log_name: &'static str,
    ) -> Client {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                wanted: None,
                posted: false,
                closed: false,
            }),
            changed: Condvar::new(),
        });
        let worker = shared.clone();
        let spawned = std::thread::Builder::new()
            .name("discord".to_owned())
            .spawn(move || work(&worker, &id, &dirs, retry, gap, log_name));
        if spawned.is_err() {
            crate::program_log::log(&format!("{log_name}: no thread for the status"));
        }
        Client { shared }
    }

    /// The status wanted now; the same one again is not sent again.
    pub fn post(&self, activity: Option<Activity>) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.posted && state.wanted == activity {
            return;
        }
        state.wanted = activity;
        state.posted = true;
        self.shared.changed.notify_all();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed = true;
        self.shared.changed.notify_all();
    }
}

fn work(
    shared: &Shared,
    id: &str,
    dirs: &[PathBuf],
    retry: Duration,
    gap: Duration,
    log_name: &str,
) {
    use crate::program_log::log;
    let mut stream: Option<Box<dyn Stream>> = None;
    // What Discord holds from this connection (none sent yet: `None`).
    let mut sent: Option<Option<Activity>> = None;
    let mut last_try: Option<Instant> = None;
    let mut last_send: Option<Instant> = None;
    let mut nonce = 0u64;
    let mut said_absent = false;
    loop {
        let want = {
            let mut state = shared
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                if state.closed {
                    return;
                }
                let pending = state.posted && sent.as_ref() != Some(&state.wanted);
                if !pending {
                    state = shared
                        .changed
                        .wait(state)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    continue;
                }
                let for_retry = if stream.is_none() {
                    last_try.map(|t| retry.saturating_sub(t.elapsed()))
                } else {
                    None
                };
                let for_gap = last_send.map(|t| gap.saturating_sub(t.elapsed()));
                let delay = for_retry
                    .unwrap_or(Duration::ZERO)
                    .max(for_gap.unwrap_or(Duration::ZERO));
                if delay.is_zero() {
                    break state.wanted.clone();
                }
                state = shared
                    .changed
                    .wait_timeout(state, delay)
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .0;
            }
        };
        if stream.is_none() {
            if want.is_none() {
                // Nothing to clear where nothing was set.
                sent = Some(None);
                continue;
            }
            last_try = Some(Instant::now());
            let connected = open(dirs).and_then(|mut s| {
                exchange(&mut *s, OP_HANDSHAKE, &handshake_json(id))
                    .ok()
                    .map(|()| s)
            });
            match connected {
                Some(s) => {
                    stream = Some(s);
                    said_absent = false;
                    log(&format!("{log_name}: connected to Discord"));
                }
                None => {
                    if !said_absent {
                        said_absent = true;
                        log(&format!(
                            "{log_name}: Discord is not there; looking again now and then"
                        ));
                    }
                    continue;
                }
            }
        }
        nonce += 1;
        let json = set_activity_json(std::process::id(), want.as_ref(), nonce);
        let connection = stream.as_mut().expect("connected above");
        match exchange(&mut **connection, OP_FRAME, &json) {
            Ok(()) => {
                sent = Some(want);
                last_send = Some(Instant::now());
            }
            Err(_) => {
                stream = None;
                sent = None;
                last_send = Some(Instant::now());
                log(&format!("{log_name}: lost Discord"));
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;

    fn read_frame(stream: &mut impl Read) -> Option<(u32, String)> {
        let mut header = [0u8; 8];
        stream.read_exact(&mut header).ok()?;
        let (op, len) = parse_header(header);
        let mut body = vec![0u8; len as usize];
        stream.read_exact(&mut body).ok()?;
        Some((op, String::from_utf8(body).ok()?))
    }

    #[test]
    fn a_fake_discord_gets_the_handshake_and_the_status() {
        let dir = PathBuf::from("/mnt/discoII/rc2k_scratch/checks/discord")
            .join(format!("fake_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let listener = UnixListener::bind(dir.join("discord-ipc-0")).unwrap();
        let (frames, received) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let ready = "{\"cmd\":\"DISPATCH\",\"evt\":\"READY\"}";
            while let Some(frame) = read_frame(&mut stream) {
                frames.send(frame).unwrap();
                stream.write_all(&encode_frame(1, ready)).unwrap();
            }
        });
        let client = Client::start(
            "4242".to_owned(),
            vec![dir.clone()],
            Duration::from_millis(50),
            Duration::from_millis(50),
            "discord test",
        );
        let activity = Activity {
            details: "Rallying a \"Seat\"".to_owned(),
            state: Some("Single rally \u{b7} Ballagyr".to_owned()),
            start: Some(1_700_000_000),
            large_image: "seat_cordoba_wrc".to_owned(),
            large_text: "Seat Cordoba WRC".to_owned(),
            small_image: Some("logo".to_owned()),
            small_text: Some("Rally Championship".to_owned()),
        };
        client.post(Some(activity.clone()));
        let wait = Duration::from_secs(10);
        let (op, hello) = received.recv_timeout(wait).unwrap();
        assert_eq!(
            (op, hello.as_str()),
            (0, "{\"v\":1,\"client_id\":\"4242\"}")
        );
        let (op, command) = received.recv_timeout(wait).unwrap();
        assert_eq!(op, 1);
        assert_eq!(
            command,
            set_activity_json(std::process::id(), Some(&activity), 1)
        );
        assert!(command.contains("\"details\":\"Rallying a \\\"Seat\\\"\""));
        assert!(command.contains("\"start\":1700000000"));
        // The same status again sends nothing; a changed one is sent.
        client.post(Some(activity.clone()));
        client.post(None);
        let (_, cleared) = received.recv_timeout(wait).unwrap();
        assert!(cleared.contains("\"activity\":null") && cleared.contains("\"nonce\":\"2\""));
        assert!(received.try_recv().is_err());
        drop(client);
        drop(server);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
