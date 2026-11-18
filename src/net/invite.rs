//! The invitation to a network game that waits for the front end's menus (`rc2k_core::net::
//! invite` holds the rules): an invitation accepted in Steam's friends window, a `+connect_lobby`
//! launch argument, or (in test builds) the remote driver's pretended one.

use rc2k_core::net::invite::{Invites, Place};
use std::sync::Mutex;

/// Where an invitation leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invite {
    /// The connection to make (its provider's GUID in the connection list).
    pub provider: [u8; 16],
    /// The session to join; the first one listed when none is named.
    pub instance: Option<[u8; 16]>,
}

static WAITING: Mutex<Invites<Invite>> = Mutex::new(Invites::new());

fn waiting() -> std::sync::MutexGuard<'static, Invites<Invite>> {
    WAITING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// An invitation has arrived.
#[cfg_attr(
    all(target_os = "linux", target_arch = "aarch64", not(feature = "remote")),
    allow(dead_code, reason = "the handheld has no Steam to bring invitations")
)]
pub fn offer(invite: Invite) {
    waiting().offer(invite);
}

/// The invitation of the program's launch argument, noted once when the menus first ask.
#[cfg(steam_support)]
fn note_launch_argument() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if let Some(lobby) = crate::net::steam::connect_lobby_argument(std::env::args()) {
            offer(Invite {
                provider: crate::net::steam::PROVIDER,
                instance: Some(crate::net::steam::lobby_instance(lobby)),
            });
        }
    });
}

/// The invitation waiting for the menus, if there is one (it no longer waits).
pub fn take_in_menus() -> Option<Invite> {
    #[cfg(steam_support)]
    note_launch_argument();
    waiting().take(Place::Menus)
}

/// The invitations that came during a network game are dropped when it ends.
pub fn forget() {
    waiting().forget();
}
