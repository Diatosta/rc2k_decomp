//! Creates DirectPlay's object in-process ("NET: Create dplay"). Returns true when made.

use crate::net::lobby::{co_create, log_bytes};
use crate::net::store::NetShared;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn create_dplay(vars: &NetShared, lines: &NetTexts) -> bool {
    log_bytes(&lines.text(texts::NET_CREATE_DPLAY), true, 2);
    co_create(
        &crate::net::store::CLSID_DPLAY,
        &crate::net::store::IID_DPLAY4A,
        &vars.dplay,
    ) == 0
}
