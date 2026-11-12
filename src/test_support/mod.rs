//! What the tests share to read the game's first values and to put the game's state back,
//! by store name and field path rather than by address: `first_values` (the frozen first
//! values in `data/first_values.golden`) and `state` (a snapshot of every store).

pub mod first_values;
mod freeze;
pub mod front;
pub mod race_places;
pub mod race_vectors;
pub mod state;
pub mod vectors;
