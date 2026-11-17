//! An invitation to a network game, waiting for the player to be somewhere it can be acted on.
//!
//! An invitation is accepted outside the game (in Steam's friends window) or given at the
//! program's start, at any moment of play. It is acted on only in the front end's menus, where the
//! network flow can begin; anywhere else it stays waiting. The latest invitation replaces an
//! earlier one that was not acted on, and invitations that arrive during a network game are
//! dropped when it ends: the player did not ask for them to take effect later.

/// Where the program is when it asks for an invitation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    /// In the front end's menus, outside a network game.
    Menus,
    /// Anywhere else: a race, a replay, a film, the installer, a network game.
    Elsewhere,
}

/// The invitations not yet acted on: at most one, the latest.
#[derive(Debug)]
pub struct Invites<T> {
    waiting: Option<T>,
}

impl<T> Invites<T> {
    pub const fn new() -> Self {
        Invites { waiting: None }
    }

    /// An invitation arrives; it replaces one still waiting.
    pub fn offer(&mut self, invite: T) {
        self.waiting = Some(invite);
    }

    /// The waiting invitation, taken (and no longer waiting) when the player is in the menus.
    pub fn take(&mut self, place: Place) -> Option<T> {
        match place {
            Place::Menus => self.waiting.take(),
            Place::Elsewhere => None,
        }
    }

    /// Whether an invitation waits.
    pub fn waits(&self) -> bool {
        self.waiting.is_some()
    }

    /// The waiting invitation dropped (a network game ended).
    pub fn forget(&mut self) {
        self.waiting = None;
    }
}

impl<T> Default for Invites<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invitation_waits_outside_the_menus() {
        let mut invites = Invites::new();
        invites.offer(7);
        assert_eq!(invites.take(Place::Elsewhere), None);
        assert!(invites.waits());
        assert_eq!(invites.take(Place::Menus), Some(7));
        assert!(!invites.waits());
    }

    #[test]
    fn an_invitation_is_acted_on_once() {
        let mut invites = Invites::new();
        invites.offer(1);
        assert_eq!(invites.take(Place::Menus), Some(1));
        assert_eq!(invites.take(Place::Menus), None);
    }

    #[test]
    fn the_latest_invitation_wins() {
        let mut invites = Invites::new();
        invites.offer(1);
        invites.offer(2);
        assert_eq!(invites.take(Place::Menus), Some(2));
    }

    #[test]
    fn a_forgotten_invitation_is_gone() {
        let mut invites = Invites::new();
        invites.offer(1);
        invites.forget();
        assert_eq!(invites.take(Place::Menus), None);
        // One that arrives after is kept.
        invites.offer(3);
        assert_eq!(invites.take(Place::Menus), Some(3));
    }
}
