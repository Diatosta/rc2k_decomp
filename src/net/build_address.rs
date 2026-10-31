//! `net::build_address` (0x41882C): "NET: DPCA", and connection eax's DirectPlay address
//! built with the lobby object (`CreateCompoundAddress`): the address elements cleared
//! (`dword_52AC3C`, `dword_52AC40`, the size `dword_52AC44`) and `g_netPosting` 0; for IPX
//! (`unk_5AB350`) the service provider alone; for TCP/IP (`unk_5AB360`) the service
//! provider, the host's address (`unk_52ABE4`, its length + 1) and, if not 0, the port
//! (`dword_52AC24`, 2 bytes); for any other the record's service provider and
//! `g_netPosting` 1 (DirectPlay's calls then made on the receiving thread: a modem's or a
//! serial link's dialogs, it seems). The size asked (DPERR_BUFFERTOOSMALL), that much memory
//! taken on the network's list into `dword_52AC40`, the address made there.
//!
//! In:  `connection` (eax: the connection's number)
//! Out: true when built (the original: carry clear and the flags of `test eax, eax`; every
//!      register kept); false for the carry: no lobby object, the size not asked as
//!      expected, or `CreateCompoundAddress` failing
//! Caller: `net::init_connection`.

use crate::errors::{BadInput, Refused};
use crate::net::add_address_element::{self as elements, add_address_element};
use crate::net::lobby::log;
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use rc2k_core::net::address::{self, Provider};

/// The Rust version (see the header).
pub fn build_address(connection_number: u32) -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_DPCA), true, 2);
    if NET.lobby.get() == 0 {
        return Ok(false);
    }
    let guid = crate::net::tables::with_connection(connection_number, |c| c.provider)?;
    elements::clear();
    NET.address.set(0);
    NET.address_size.set(0);
    NET.posting.set(0);
    let provider = address::provider(|which| -> Result<bool, Refused> {
        let known = if which == Provider::Ipx {
            &NET.sp_ipx
        } else {
            &NET.sp_tcpip
        };
        Ok(known.read() == guid)
    })?;
    match provider {
        Provider::Ipx => add_address_element(&NET.aid_service_provider, &NET.sp_ipx.read()),
        Provider::TcpIp => {
            add_address_element(&NET.aid_service_provider, &NET.sp_tcpip.read());
            let host = NET.host_address.read();
            let length = host
                .iter()
                .position(|&c| c == 0)
                .ok_or(BadInput("a host address without its end"))? as u32;
            let size = address::host_address_size(length) as usize;
            add_address_element(&NET.aid_inet, &host[..size]);
            if address::has_port(NET.port.get()) {
                let port = NET.port.get().to_le_bytes();
                add_address_element(&NET.aid_inet_port, &port[..address::PORT_SIZE as usize]);
            }
        }
        Provider::Other => {
            let data = &guid[..address::PROVIDER_SIZE as usize];
            add_address_element(&NET.aid_service_provider, data);
        }
    }
    if address::posts(provider) {
        NET.posting.set(1);
    }
    let parts = elements::elements();
    let (result, size) =
        crate::net::dp_objects::create_compound_address(&parts, NET.address_size.get());
    NET.address_size.set(size);
    if result != address::BUFFER_TOO_SMALL {
        return Ok(false);
    }
    crate::net::tables::count(NET.address_size.get())?;
    let made = crate::net::dp_objects::compound_address(&parts);
    let fits = made.len() as u32 <= NET.address_size.get();
    crate::net::tables::NET_TABLES.with(|t| t.address = made)?;
    Ok(fits)
}
