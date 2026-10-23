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
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed (no lobby object
//!      among the ways), else the flags of `test eax, eax`
//! Caller: `net::init_connection`. Its calls through `part` (`net::add_address_element`
//! leaves edx); `CreateCompoundAddress` from Rust (`net::dplay`). Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins and whose lobby is a test
//! object, against the Rust version: each provider, a port or none, each failure.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::dplay::{CREATE_COMPOUND_ADDRESS, call};
use crate::net::lobby::{
    ADDRESS, ADDRESS_SIZE, ALLOCATE, ELEMENT_TABLE, ELEMENTS, HOST_ADDRESS, LOBBY, LOG_L2_NL,
    NET_MEMORY, PORT, POSTING, Part, SAME_GUID, TEXT_LENGTH, a, cmp_flags, g, lobby_entry,
    log_text, p, test_eax,
};
use rc2k_core::net::address::{self, Provider};

const TEXT: usize = 0x5C2EBD; // "NET: DPCA"
const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry
const ADD_ELEMENT: u32 = 0x41896E; // net::add_address_element
pub const IPX: usize = 0x5AB350; // DPSPGUID_IPX
pub const TCPIP: usize = 0x5AB360; // DPSPGUID_TCPIP
const SERVICE_PROVIDER: usize = 0x5AB3A0; // DPAID_ServiceProvider
const INET: usize = 0x5AB3E0; // DPAID_INet
const INET_PORT: usize = 0x5AB3F0; // DPAID_INetPort

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn build_address(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(LOBBY), 0);
    if g(LOBBY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(CONNECTION_ENTRY, &mut r)?;
    p(ELEMENTS, 0);
    p(ADDRESS, 0);
    p(ADDRESS_SIZE, 0);
    p(POSTING, 0);
    r.eax = r.eax.wrapping_add(0x40);
    // Converted: `rc2k_core::net::address` (`provider`, the elements' sizes, `has_port`,
    // `posts`, `address::BUFFER_TOO_SMALL`).
    let element = |r: &mut Regs, kind: usize, size: u32, part: Part| -> Result<(), Refused> {
        r.eax = a(kind);
        r.ecx = size;
        part(ADD_ELEMENT, r)
    };
    let provider = address::provider(|which| -> Result<bool, Refused> {
        r.ebx = a(if which == Provider::Ipx { IPX } else { TCPIP });
        part(SAME_GUID, &mut r)?;
        Ok(r.eflags & ZERO_FLAG != 0)
    })?;
    match provider {
        Provider::Ipx => element(&mut r, SERVICE_PROVIDER, address::PROVIDER_SIZE, part)?,
        Provider::TcpIp => {
            element(&mut r, SERVICE_PROVIDER, address::PROVIDER_SIZE, part)?;
            r.eax = a(INET);
            r.ebx = a(HOST_ADDRESS);
            part(TEXT_LENGTH, &mut r)?;
            r.ecx = address::host_address_size(r.ecx);
            part(ADD_ELEMENT, &mut r)?;
            if address::has_port(g(PORT)) {
                r.eax = a(INET_PORT);
                r.ebx = a(PORT);
                r.ecx = address::PORT_SIZE;
                part(ADD_ELEMENT, &mut r)?;
            }
        }
        Provider::Other => {
            r.ebx = r.eax;
            r.ecx = address::PROVIDER_SIZE;
            r.eax = a(SERVICE_PROVIDER);
            part(ADD_ELEMENT, &mut r)?;
        }
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
    r.eax = make(0);
    let v = r.eax;
    cmp_flags(&mut r, v, address::BUFFER_TOO_SMALL);
    if r.eax != address::BUFFER_TOO_SMALL {
        r.set_carry(true);
        return Ok(left(&r));
    }
    r.eax = g(ADDRESS_SIZE);
    r.ebx = a(NET_MEMORY);
    part(ALLOCATE, &mut r)?;
    if r.carry() {
        r.set_carry(true);
        return Ok(left(&r));
    }
    p(ADDRESS, r.eax);
    r.eax = make(g(ADDRESS));
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, build_address)
}
