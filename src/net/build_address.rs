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

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::dplay::{CREATE_COMPOUND_ADDRESS, call};
use crate::net::lobby::{
    ADDRESS, ADDRESS_SIZE, ALLOCATE, ELEMENT_TABLE, ELEMENTS, FLAGS, HOST_ADDRESS, LOBBY,
    LOBBY_METHOD, LOG_L2_NL, NET_MEMORY, PORT, POSTING, Part, SAME_GUID, Saved, TEXT_LENGTH,
    TestObject, a, answer, cmp_flags, com_answer, dispatch_lobby, dword, g, log_text, p, region,
    run_both, test_eax,
};
use crate::net::types::NetConnection;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::build_address", rva(0x41882C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2EBD; // "NET: DPCA"
const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry
const ADD_ELEMENT: u32 = 0x41896E; // net::add_address_element
pub const IPX: usize = 0x5AB350; // DPSPGUID_IPX
pub const TCPIP: usize = 0x5AB360; // DPSPGUID_TCPIP
const SERVICE_PROVIDER: usize = 0x5AB3A0; // DPAID_ServiceProvider
const INET: usize = 0x5AB3E0; // DPAID_INet
const INET_PORT: usize = 0x5AB3F0; // DPAID_INetPort
const BUFFER_TOO_SMALL: u32 = 0x8877_001E;

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
    r.ebx = a(IPX);
    part(SAME_GUID, &mut r)?;
    let element = |r: &mut Regs, kind: usize, size: u32, part: Part| -> Result<(), Refused> {
        r.eax = a(kind);
        r.ecx = size;
        part(ADD_ELEMENT, r)
    };
    if r.eflags & ZERO_FLAG != 0 {
        element(&mut r, SERVICE_PROVIDER, 0x10, part)?;
    } else {
        r.ebx = a(TCPIP);
        part(SAME_GUID, &mut r)?;
        if r.eflags & ZERO_FLAG != 0 {
            element(&mut r, SERVICE_PROVIDER, 0x10, part)?;
            r.eax = a(INET);
            r.ebx = a(HOST_ADDRESS);
            part(TEXT_LENGTH, &mut r)?;
            r.ecx = r.ecx.wrapping_add(1);
            part(ADD_ELEMENT, &mut r)?;
            if g(PORT) != 0 {
                r.eax = a(INET_PORT);
                r.ebx = a(PORT);
                r.ecx = 2;
                part(ADD_ELEMENT, &mut r)?;
            }
        } else {
            r.ebx = r.eax;
            r.ecx = 0x10;
            r.eax = a(SERVICE_PROVIDER);
            part(ADD_ELEMENT, &mut r)?;
            p(POSTING, 1);
        }
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
    cmp_flags(&mut r, v, BUFFER_TOO_SMALL);
    if r.eax != BUFFER_TOO_SMALL {
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, build_address);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2_NL,
        CONNECTION_ENTRY,
        SAME_GUID,
        ADD_ELEMENT,
        TEXT_LENGTH,
        ALLOCATE,
    ];
    // pusha; pusha; lea eax, aNetDpca
    let copy = match crate::net::lobby::copy(
        0x41882C,
        0x142,
        &[0x60, 0x60, 0x8D, 0x05, 0xBD, 0x2E, 0x5C, 0x00],
        &[
            0x8, 0x1B, 0x51, 0x63, 0x70, 0x82, 0x93, 0x99, 0xB8, 0xCC, 0x10A,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let lobby = TestObject::lobby();
    let lo = lobby.addr();
    let mut record = vec![0u8; 0x60];
    let rec = record.as_mut_ptr() as u32;
    let all = [
        dword(LOBBY),
        region(PORT, 4),
        region(ELEMENTS, 0xC),
        dword(POSTING),
    ];
    let saved = Saved::new(&all);
    // (what, a lobby, the provider (0 IPX, 1 TCP/IP, 2 another), the port, what the size
    // asked gives, the memory given, what the address gives)
    type Case = (&'static str, bool, u32, u32, u32, bool, u32);
    let cases: [Case; 9] = [
        ("no lobby", false, 0, 0, BUFFER_TOO_SMALL, true, 0),
        ("IPX", true, 0, 0, BUFFER_TOO_SMALL, true, 0),
        ("TCP/IP", true, 1, 0, BUFFER_TOO_SMALL, true, 0),
        (
            "TCP/IP with a port",
            true,
            1,
            2300,
            BUFFER_TOO_SMALL,
            true,
            0,
        ),
        ("a modem", true, 2, 0, BUFFER_TOO_SMALL, true, 0),
        ("the size not asked", true, 0, 0, 0, true, 0),
        ("no memory", true, 0, 0, BUFFER_TOO_SMALL, false, 0),
        (
            "the address fails",
            true,
            1,
            0,
            BUFFER_TOO_SMALL,
            true,
            0x8877_0005,
        ),
        ("the size fails", true, 2, 0, 0x8877_0005, true, 0),
    ];
    for (what, has_lobby, provider, port, first, given, second) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                eax: 4,
                ..Regs::SENTINEL
            },
            &all,
            FLAGS,
            &[],
            &|| unsafe {
                p(LOBBY, if has_lobby { lo } else { 0 });
                p(PORT, port);
                p(ELEMENTS, 9);
                p(ADDRESS, 9);
                p(ADDRESS_SIZE, 9);
                p(POSTING, 9);
                for n in 0..0x60u32 {
                    ((rec + n) as *mut u8).write(n as u8);
                }
                match provider {
                    0 => crate::net::lobby::movsb(
                        rec + field!(NetConnection, service_provider),
                        a(IPX),
                        0x10,
                    ),
                    1 => crate::net::lobby::movsb(
                        rec + field!(NetConnection, service_provider),
                        a(TCPIP),
                        0x10,
                    ),
                    _ => {}
                }
                answer(Box::new(move |place, r| match place {
                    CONNECTION_ENTRY => r.eax = rec,
                    SAME_GUID => r.set_zero(crate::system::same_guid::same_guid(r.eax, r.ebx)),
                    ADD_ELEMENT => {
                        r.edx = 0x18 * g(ELEMENTS);
                        p(ELEMENTS, g(ELEMENTS).wrapping_add(1));
                    }
                    TEXT_LENGTH => {
                        r.ecx = 9;
                        r.set_zero(true);
                    }
                    ALLOCATE => {
                        r.eax = 0x00AD_D000;
                        r.set_carry(!given);
                    }
                    _ => {}
                }));
                let mut asks = 0;
                com_answer(Box::new(move |place, args| {
                    if place == LOBBY_METHOD | CREATE_COMPOUND_ADDRESS {
                        asks += 1;
                        (args[4] as *mut u32).write_unaligned(0x40);
                        if asks == 1 { first } else { second }
                    } else {
                        0
                    }
                }));
            },
            &build_address,
        );
    }
    drop(saved);
    let _ = (&mut record, lobby);
    t.finish()
}
