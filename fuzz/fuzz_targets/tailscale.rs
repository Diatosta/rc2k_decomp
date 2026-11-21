#![no_main]

// The text `tailscale status --json` prints, read for the peers' addresses.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = rc2k_core::net::tailscale::peer_addresses(text);
    }
});
