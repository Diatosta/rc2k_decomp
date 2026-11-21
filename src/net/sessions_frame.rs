//! The frame of `net::choose_session`, like `net::connections_frame`'s: the title (another one
//! with no session listed) and the sessions' names, a locked session's with a mark before it.

use crate::errors::Refused;
use crate::net::connections_frame::list_frame;

/// A session's name for the list: the mark before it when it is locked, cut to the record's room.
fn listed_name(name: [u8; 0x40], locked: bool, mark: &[u8]) -> [u8; 0x40] {
    if !locked {
        return name;
    }
    let end = name.iter().position(|&b| b == 0).unwrap_or(0x3F);
    let mut shown = mark.to_vec();
    shown.extend_from_slice(&name[..end]);
    shown.truncate(0x3F);
    let mut out = [0u8; 0x40];
    out[..shown.len()].copy_from_slice(&shown);
    out
}

pub(super) fn sessions_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let link = &o.net.link;
    // The logic is in `rc2k_core::net::screens::sessions_title`.
    let title = rc2k_core::net::screens::sessions_title(link.sessions().get());
    let mark = crate::strings::append_string_by_id::own_text(
        o.strings,
        rc2k_core::strings::own::id::LOCK_MARK,
    );
    list_frame(o, title, link.sessions(), |n| {
        crate::net::tables::with_session(link, n, |s| listed_name(s.name, s.locked, &mark))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_locked_name_has_the_mark_and_fits() {
        let mut name = [0u8; 0x40];
        name[..4].copy_from_slice(b"Rall");
        assert_eq!(listed_name(name, false, b"[x] "), name);
        let shown = listed_name(name, true, b"[x] ");
        assert_eq!(&shown[..9], b"[x] Rall\0");
        let mut long = [b'a'; 0x40];
        long[0x3F] = 0;
        let shown = listed_name(long, true, b"[x] ");
        assert_eq!(shown[0x3F], 0);
        assert_eq!(&shown[..4], b"[x] ");
    }
}
