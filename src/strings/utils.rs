use std::ffi::{CStr, c_char};

pub fn show(src: *const u8) -> String {
    let bytes = unsafe { CStr::from_ptr(src as *const c_char) }.to_bytes();
    let shown = &bytes[..bytes.len().min(200)];
    let more = if bytes.len() > shown.len() { "..." } else { "" };
    format!("\"{}\"{more}", shown.escape_ascii())
}

/// What a function wrote: the bytes from the start of `buf` up to `end`.
pub fn written(buf: &[u8], end: *const u8) -> String {
    let len = (end as usize).wrapping_sub(buf.as_ptr() as usize);
    match buf.get(..len) {
        Some(b) => format!("\"{}\"", b.escape_ascii()),
        None => format!("<end pointer outside buffer: offset {len}>"),
    }
}

/// Printable form of a byte slice, truncated to 80 bytes.
pub fn bytes_str(b: &[u8]) -> String {
    let n = b.len().min(80);
    format!(
        "\"{}\"{}",
        b[..n].escape_ascii(),
        if b.len() > n { "..." } else { "" }
    )
}
