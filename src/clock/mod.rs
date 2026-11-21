//! The game's time: the millisecond clock (`tick_count`), sleeping (`sleep`) and the local
//! date and time packed into one dword (`packed_local_time`). They read the machine's clock
//! and use nothing of the game, so every module may use them.

pub(crate) mod packed_local_time;
pub(crate) mod sleep;
pub(crate) mod tick_count;
