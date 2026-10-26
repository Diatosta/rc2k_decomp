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

use crate::errors::Refused;
use crate::net::add_address_element::add_address_element;
use crate::net::lobby::{connection, log, net_allocate};
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use crate::system::same_guid::same_guid;
use rc2k_core::net::address::{self, Provider};

/// The Rust version.
pub fn build_address(connection_number: u32) -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_DPCA), true, 2);
    if NET.lobby.get() == 0 {
        return Ok(false);
    }
    let guid = connection(connection_number)?.wrapping_add(0x40);
    NET.elements.set(0);
    NET.address.set(0);
    NET.address_size.set(0);
    NET.posting.set(0);
    // Converted: `rc2k_core::net::address` (`provider`, the elements' sizes, `has_port`,
    // `posts`, `address::BUFFER_TOO_SMALL`).
    let provider = address::provider(|which| -> Result<bool, Refused> {
        Ok(same_guid(
            guid,
            (if which == Provider::Ipx {
                &NET.sp_ipx
            } else {
                &NET.sp_tcpip
            })
            .addr(),
        ))
    })?;
    match provider {
        Provider::Ipx => add_address_element(
            NET.aid_service_provider.addr(),
            address::PROVIDER_SIZE,
            NET.sp_ipx.addr(),
        ),
        Provider::TcpIp => {
            add_address_element(
                NET.aid_service_provider.addr(),
                address::PROVIDER_SIZE,
                NET.sp_tcpip.addr(),
            );
            let length = {
                crate::strings::text_length::text_length(crate::pointers::host(
                    NET.host_address.addr(),
                ) as *const u8)
            }?;
            add_address_element(
                NET.aid_inet.addr(),
                address::host_address_size(length),
                NET.host_address.addr(),
            );
            if address::has_port(NET.port.get()) {
                add_address_element(
                    NET.aid_inet_port.addr(),
                    address::PORT_SIZE,
                    NET.port.addr(),
                );
            }
        }
        Provider::Other => add_address_element(
            NET.aid_service_provider.addr(),
            address::PROVIDER_SIZE,
            guid,
        ),
    }
    if address::posts(provider) {
        NET.posting.set(1);
    }
    let make = |at: u32| {
        crate::net::dp_objects::create_compound_address(
            NET.lobby.get(),
            NET.element_table.addr(),
            NET.elements.get(),
            at,
            NET.address_size.addr(),
        )
    };
    if make(0) != address::BUFFER_TOO_SMALL {
        return Ok(false);
    }
    NET.address.set(net_allocate(NET.address_size.get())?);
    Ok(make(NET.address.get()) == 0)
}
