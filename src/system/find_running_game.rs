//! Whether a window of the game's class ("MF-SDG R5") is open already. SDL makes the window
//! under its own class, so none is ever found: a second copy runs.

/// True if the game runs already: never (see the header).
pub fn find_running_game() -> bool {
    false
}
