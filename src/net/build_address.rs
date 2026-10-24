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
use crate::net::dplay::{CREATE_COMPOUND_ADDRESS, call};
use crate::net::lobby::{
    ADDRESS, ADDRESS_SIZE, ELEMENT_TABLE, ELEMENTS, HOST_ADDRESS, LOBBY, PORT, POSTING, a,
    connection, g, log, net_allocate, p,
};
use crate::system::same_guid::same_guid;
use rc2k_core::net::address::{self, Provider};

const TEXT: usize = 0x5C2EBD; // "NET: DPCA"
pub const IPX: usize = 0x5AB350; // DPSPGUID_IPX
pub const TCPIP: usize = 0x5AB360; // DPSPGUID_TCPIP
const SERVICE_PROVIDER: usize = 0x5AB3A0; // DPAID_ServiceProvider
const INET: usize = 0x5AB3E0; // DPAID_INet
const INET_PORT: usize = 0x5AB3F0; // DPAID_INetPort

/// The Rust version.
pub fn build_address(connection_number: u32) -> Result<bool, Refused> {
    log(a(TEXT), true, 2);
    if g(LOBBY) == 0 {
        return Ok(false);
    }
    let guid = connection(connection_number)?.wrapping_add(0x40);
    p(ELEMENTS, 0);
    p(ADDRESS, 0);
    p(ADDRESS_SIZE, 0);
    p(POSTING, 0);
    // Converted: `rc2k_core::net::address` (`provider`, the elements' sizes, `has_port`,
    // `posts`, `address::BUFFER_TOO_SMALL`).
    let provider = address::provider(|which| -> Result<bool, Refused> {
        Ok(same_guid(
            guid,
            a(if which == Provider::Ipx { IPX } else { TCPIP }),
        ))
    })?;
    match provider {
        Provider::Ipx => add_address_element(a(SERVICE_PROVIDER), address::PROVIDER_SIZE, a(IPX)),
        Provider::TcpIp => {
            add_address_element(a(SERVICE_PROVIDER), address::PROVIDER_SIZE, a(TCPIP));
            let length =
                unsafe { crate::strings::text_length::text_length(a(HOST_ADDRESS) as *const u8) }?;
            add_address_element(a(INET), address::host_address_size(length), a(HOST_ADDRESS));
            if address::has_port(g(PORT)) {
                add_address_element(a(INET_PORT), address::PORT_SIZE, a(PORT));
            }
        }
        Provider::Other => add_address_element(a(SERVICE_PROVIDER), address::PROVIDER_SIZE, guid),
    }
    if address::posts(provider) {
        p(POSTING, 1);
    }
    let make = |at: u32| unsafe {
        call(
            LOBBY,
            CREATE_COMPOUND_ADDRESS,
            &[a(ELEMENT_TABLE), g(ELEMENTS), at, a(ADDRESS_SIZE)],
        )
    };
    if make(0) != address::BUFFER_TOO_SMALL {
        return Ok(false);
    }
    p(ADDRESS, net_allocate(g(ADDRESS_SIZE))?);
    Ok(make(g(ADDRESS)) == 0)
}
