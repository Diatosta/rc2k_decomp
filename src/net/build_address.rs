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
//! Takes `connection`. Returns true when built; false when there is no lobby object, the size
//! was not asked as expected, or `CreateCompoundAddress` failed.
//! Caller: `net::init_connection`.

use crate::errors::{BadInput, Refused};
use crate::net::add_address_element::{self as elements, add_address_element};
use crate::net::lobby::log;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use rc2k_core::net::address::{self, Provider};

/// The Rust version (see the header).
pub fn build_address(net: &Net, connection_number: u32) -> Result<bool, Refused> {
    let vars = &net.vars;
    log(VIEWER_PANEL.text_ptr(texts::NET_DPCA), true, 2);
    if vars.lobby.get() == 0 {
        return Ok(false);
    }
    let guid = crate::net::tables::with_connection(connection_number, |c| c.provider)?;
    elements::clear(vars);
    vars.address.set(0);
    vars.address_size.set(0);
    vars.posting.set(0);
    let provider = address::provider(|which| -> Result<bool, Refused> {
        let known = if which == Provider::Ipx {
            &vars.sp_ipx
        } else {
            &vars.sp_tcpip
        };
        Ok(known.read() == guid)
    })?;
    match provider {
        Provider::Ipx => add_address_element(net, &vars.aid_service_provider, &vars.sp_ipx.read()),
        Provider::TcpIp => {
            add_address_element(net, &vars.aid_service_provider, &vars.sp_tcpip.read());
            let host = vars.host_address.read();
            let length = host
                .iter()
                .position(|&c| c == 0)
                .ok_or(BadInput("a host address without its end"))? as u32;
            let size = address::host_address_size(length) as usize;
            add_address_element(net, &vars.aid_inet, &host[..size]);
            if address::has_port(vars.port.get()) {
                let port = vars.port.get().to_le_bytes();
                add_address_element(
                    net,
                    &vars.aid_inet_port,
                    &port[..address::PORT_SIZE as usize],
                );
            }
        }
        Provider::Other => {
            let data = &guid[..address::PROVIDER_SIZE as usize];
            add_address_element(net, &vars.aid_service_provider, data);
        }
    }
    if address::posts(provider) {
        vars.posting.set(1);
    }
    let parts = elements::elements();
    let (result, size) =
        crate::net::dp_objects::create_compound_address(&parts, vars.address_size.get());
    vars.address_size.set(size);
    if result != address::BUFFER_TOO_SMALL {
        return Ok(false);
    }
    crate::net::tables::count(vars.address_size.get())?;
    let made = crate::net::dp_objects::compound_address(&parts);
    let fits = made.len() as u32 <= vars.address_size.get();
    crate::net::tables::NET_TABLES.with(|t| t.address = made)?;
    Ok(fits)
}
