//! The network's variables (and a few of the installer's and the race's), out of RAL.EXE's
//! image (phase 5, `notes/conversion.md`): IDA 0x529DAD .. 0x5AB4AC. The installer's
//! script and drive letters, `nonetwork` and `inetport`, the chat lines, the send and
//! receive buffers, the session's state and description, the received and sorted
//! message stores (0x40260 bytes each), the providers' GUIDs, the lobby's key and names;
//! the split screen's switch (0x52A910) and the chosen car (0x52A920). Up to
//! `g_primary3dCard` (0x5AB4AC).
//!
//! One block with the image's layout (`crate::store`), first values the image's
//! (`store_data.rs`, generated). The code names the variables by IDA's address;
//! `framework::game_addr` takes such an address here (the receive thread's too).

crate::block!(pub mod vars {
    start: 0x529DAD,
    end: 0x5AB4AC,
    runs: crate::net::store_data::RUNS,
});
