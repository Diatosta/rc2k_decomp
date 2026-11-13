//! Releases DirectPlay's object, if any ("Destroy dplay" logged).

use crate::net::store::NetShared;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn release_dplay(vars: &NetShared, lines: &NetTexts) {
    if vars.dplay.get() != 0 {
        crate::net::lobby::log_bytes(&lines.text(texts::NET_DESTROY_DPLAY), true, 2);
        crate::net::dp_objects::release(vars.dplay.get());
        vars.dplay.set(0);
    }
}
