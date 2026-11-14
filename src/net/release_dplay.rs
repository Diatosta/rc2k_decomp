//! Releases DirectPlay's object, if any ("Destroy dplay" logged).

use crate::net::link::Link;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn release_dplay(link: &Link, lines: &NetTexts) {
    if link.has_dplay() {
        crate::net::lobby::log_bytes(&lines.text(texts::NET_DESTROY_DPLAY), true, 2);
        let _ = link.release_dplay();
    }
}
