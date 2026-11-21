//! Apple's DNS service discovery (`dns_sd.h`, in the system library): a service announced and the
//! services of a type heard, with their IPv4 addresses. iOS lets an app use Bonjour through this
//! without the multicast permission that raw broadcast and multicast sockets need (the service
//! type is listed in the app's `NSBonjourServices`).

#![allow(unsafe_code)]

use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Mutex;

type ServiceRef = *mut c_void;

/// `kDNSServiceFlagsAdd`: the reply adds an answer (without it, it removes one).
const FLAGS_ADD: u32 = 0x2;
const FLAGS_SHARE_CONNECTION: u32 = 0x4000;
const PROTOCOL_IPV4: u32 = 0x1;

type BrowseReply = unsafe extern "C" fn(
    ServiceRef,
    u32,
    u32,
    i32,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_void,
);
type ResolveReply = unsafe extern "C" fn(
    ServiceRef,
    u32,
    u32,
    i32,
    *const c_char,
    *const c_char,
    u16,
    u16,
    *const u8,
    *mut c_void,
);
type AddrReply = unsafe extern "C" fn(
    ServiceRef,
    u32,
    u32,
    i32,
    *const c_char,
    *const libc::sockaddr,
    u32,
    *mut c_void,
);
type RegisterReply = unsafe extern "C" fn(
    ServiceRef,
    u32,
    i32,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_void,
);

unsafe extern "C" {
    fn DNSServiceRegister(
        sd: *mut ServiceRef,
        flags: u32,
        interface: u32,
        name: *const c_char,
        regtype: *const c_char,
        domain: *const c_char,
        host: *const c_char,
        port_network_order: u16,
        txt_len: u16,
        txt: *const c_void,
        reply: Option<RegisterReply>,
        context: *mut c_void,
    ) -> i32;
    fn DNSServiceCreateConnection(sd: *mut ServiceRef) -> i32;
    fn DNSServiceBrowse(
        sd: *mut ServiceRef,
        flags: u32,
        interface: u32,
        regtype: *const c_char,
        domain: *const c_char,
        reply: BrowseReply,
        context: *mut c_void,
    ) -> i32;
    fn DNSServiceResolve(
        sd: *mut ServiceRef,
        flags: u32,
        interface: u32,
        name: *const c_char,
        regtype: *const c_char,
        domain: *const c_char,
        reply: ResolveReply,
        context: *mut c_void,
    ) -> i32;
    fn DNSServiceGetAddrInfo(
        sd: *mut ServiceRef,
        flags: u32,
        interface: u32,
        protocol: u32,
        hostname: *const c_char,
        reply: AddrReply,
        context: *mut c_void,
    ) -> i32;
    fn DNSServiceRefSockFD(sd: ServiceRef) -> i32;
    fn DNSServiceProcessResult(sd: ServiceRef) -> i32;
    fn DNSServiceRefDeallocate(sd: ServiceRef);
}

/// A service announced until this is dropped.
pub struct Announcement(ServiceRef);

// SAFETY: the reference is only deallocated (once, in `drop`); the daemon's connection it
// stands for may be closed from any thread.
unsafe impl Send for Announcement {}

impl Drop for Announcement {
    fn drop(&mut self) {
        // SAFETY: a reference `announce` made, deallocated once here.
        unsafe { DNSServiceRefDeallocate(self.0) };
    }
}

/// The service `kind` (`_name._udp`) announced at `port` under the device's own name; none (the
/// error code) when the daemon refuses it.
pub fn announce(kind: &CStr, port: u16) -> Result<Announcement, i32> {
    let mut sd: ServiceRef = std::ptr::null_mut();
    // SAFETY: a NUL-ended type that lives over the call; null name, domain, host and text (the
    // defaults); no reply wanted.
    let error = unsafe {
        DNSServiceRegister(
            &raw mut sd,
            0,
            0,
            std::ptr::null(),
            kind.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            port.to_be(),
            0,
            std::ptr::null(),
            None,
            std::ptr::null_mut(),
        )
    };
    if error != 0 || sd.is_null() {
        return Err(error);
    }
    Ok(Announcement(sd))
}

/// The services heard: their names and where each answers.
static HEARD: Mutex<Option<HashMap<String, SocketAddr>>> = Mutex::new(None);

/// A service resolved to its host and port, waiting for the host's address.
struct Pending {
    name: String,
    port: u16,
}

/// The services of `kind` heard so far (their addresses); the first call starts a thread that
/// listens for them for the rest of the program.
pub fn heard(kind: &CStr) -> Vec<SocketAddr> {
    let mut heard = HEARD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if heard.is_none() {
        *heard = Some(HashMap::new());
        let kind = kind.to_owned();
        std::thread::spawn(move || browse(&kind));
    }
    heard
        .as_ref()
        .map(|h| h.values().copied().collect())
        .unwrap_or_default()
}

fn note(name: &str, at: Option<SocketAddr>) {
    let mut heard = HEARD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(heard) = heard.as_mut() else {
        return;
    };
    match at {
        Some(at) => {
            if heard.insert(name.to_owned(), at) != Some(at) {
                crate::program_log::log(&format!("net: Bonjour: {name} heard at {at}"));
            }
        }
        None => {
            heard.remove(name);
        }
    }
}

/// Text from the daemon (empty when null).
///
/// # Safety
/// `text` is null or NUL-ended, alive over the call.
unsafe fn text(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: by the caller's promise.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// The one connection to the daemon every browse, resolve and address query shares; its
/// answers are read on the browsing thread only.
static CONNECTION: Mutex<usize> = Mutex::new(0);

fn connection() -> ServiceRef {
    *CONNECTION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) as ServiceRef
}

/// The browsing thread: the shared connection made, the browse begun, and its answers read for
/// the rest of the program (a failure is logged and ends it).
fn browse(kind: &CStr) {
    let mut main: ServiceRef = std::ptr::null_mut();
    // SAFETY: the daemon's calls on references made here; the browse shares the connection, its
    // type is NUL-ended and lives over the call; the reply is a function of this file.
    let error = unsafe {
        let made = DNSServiceCreateConnection(&raw mut main);
        if made != 0 {
            made
        } else {
            *CONNECTION
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = main as usize;
            let mut browse = main;
            DNSServiceBrowse(
                &raw mut browse,
                FLAGS_SHARE_CONNECTION,
                0,
                kind.as_ptr(),
                std::ptr::null(),
                on_browse,
                std::ptr::null_mut(),
            )
        }
    };
    if error != 0 {
        crate::program_log::log(&format!(
            "net: Bonjour: no search (error {error}; the app lists its service type?)"
        ));
        return;
    }
    // SAFETY: the connection just made.
    let fd = unsafe { DNSServiceRefSockFD(main) };
    loop {
        let mut wait = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one pollfd of this frame.
        let ready = unsafe { libc::poll(&raw mut wait, 1, 1000) };
        if ready > 0 {
            // SAFETY: the connection, read on this thread only; the replies run inside.
            let error = unsafe { DNSServiceProcessResult(main) };
            if error != 0 {
                crate::program_log::log(&format!("net: Bonjour: the search ended (error {error})"));
                return;
            }
        }
    }
}

/// A service found or lost: a found one is resolved to its host and port.
unsafe extern "C" fn on_browse(
    _sd: ServiceRef,
    flags: u32,
    interface: u32,
    error: i32,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    _context: *mut c_void,
) {
    if error != 0 {
        return;
    }
    if flags & FLAGS_ADD == 0 {
        // SAFETY: the daemon's NUL-ended name, alive over the reply.
        note(&unsafe { text(name) }, None);
        return;
    }
    let mut resolve = connection();
    // SAFETY: the daemon's NUL-ended texts, alive over this reply (the call copies them); the
    // resolve shares the connection; the reply is a function of this file.
    unsafe {
        DNSServiceResolve(
            &raw mut resolve,
            FLAGS_SHARE_CONNECTION,
            interface,
            name,
            kind,
            domain,
            on_resolve,
            std::ptr::null_mut(),
        );
    }
}

/// A service's host and port: the host's IPv4 address is asked for, the resolve ended.
unsafe extern "C" fn on_resolve(
    sd: ServiceRef,
    _flags: u32,
    interface: u32,
    error: i32,
    full_name: *const c_char,
    host: *const c_char,
    port_network_order: u16,
    _txt_len: u16,
    _txt: *const u8,
    _context: *mut c_void,
) {
    // SAFETY: the resolve this reply is for, ended once (allowed inside its own reply).
    unsafe { DNSServiceRefDeallocate(sd) };
    if error != 0 {
        return;
    }
    let pending = Box::new(Pending {
        // SAFETY: the daemon's NUL-ended name, alive over the reply.
        name: unsafe { text(full_name) },
        port: u16::from_be(port_network_order),
    });
    let context = Box::into_raw(pending).cast::<c_void>();
    let mut query = connection();
    // SAFETY: the daemon's NUL-ended host name, alive over this reply (the call copies it); the
    // query shares the connection; its context is the box just made, freed by its reply.
    let made = unsafe {
        DNSServiceGetAddrInfo(
            &raw mut query,
            FLAGS_SHARE_CONNECTION,
            interface,
            PROTOCOL_IPV4,
            host,
            on_address,
            context,
        )
    };
    if made != 0 {
        // SAFETY: the box made above, which no query took.
        drop(unsafe { Box::from_raw(context.cast::<Pending>()) });
    }
}

/// A host's address: the service noted there, the query ended and its context freed.
unsafe extern "C" fn on_address(
    sd: ServiceRef,
    flags: u32,
    _interface: u32,
    error: i32,
    _host: *const c_char,
    address: *const libc::sockaddr,
    _ttl: u32,
    context: *mut c_void,
) {
    // A reply that says an address went (no "add" flag) is not one to note: the query goes on
    // for the one that comes.
    if error == 0 && flags & FLAGS_ADD == 0 {
        return;
    }
    // SAFETY: the query this reply is for, ended once (allowed inside its own reply), and the
    // box `on_resolve` gave it, taken back once here.
    let pending = unsafe {
        DNSServiceRefDeallocate(sd);
        Box::from_raw(context.cast::<Pending>())
    };
    if error != 0 || address.is_null() {
        return;
    }
    // SAFETY: a socket address of the daemon, alive over the reply; an IPv4 one (asked for, and
    // its family checked) is a `sockaddr_in`.
    let ip = unsafe {
        if i32::from((*address).sa_family) != libc::AF_INET {
            return;
        }
        let v4 = &*address.cast::<libc::sockaddr_in>();
        Ipv4Addr::from(u32::from_be(v4.sin_addr.s_addr))
    };
    note(&pending.name, Some(SocketAddr::from((ip, pending.port))));
}
