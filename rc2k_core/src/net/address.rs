//! A connection's DirectPlay address (from `net::add_address_element`
//! 0x41896E and `net::build_address` 0x41882C): a compound address made of elements
//! (`DPCOMPOUNDADDRESSELEMENT`: the data type's GUID, the data's size, the data's
//! address), which by the connection's service provider: IPX the provider alone, TCP/IP
//! the provider, the host's address and a port if one is set, any other the provider and
//! DirectPlay's calls made on the receiving thread.

/// An element's bytes.
pub const ELEMENT_LEN: u32 = 0x18;

/// Where element `count` (the count so far) goes in the table: no limit, as the original.
pub fn element_at(count: u32) -> u32 {
    count.wrapping_mul(ELEMENT_LEN)
}

/// An element's bytes: the data type's GUID, the data's size, the data's address.
pub fn element(data_type: &[u8; 16], size: u32, data: u32) -> [u8; ELEMENT_LEN as usize] {
    let mut out = [0u8; ELEMENT_LEN as usize];
    out[..16].copy_from_slice(data_type);
    out[16..20].copy_from_slice(&size.to_le_bytes());
    out[20..].copy_from_slice(&data.to_le_bytes());
    out
}

/// A connection's service provider, as the address is built for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Ipx,
    TcpIp,
    Other,
}

/// The provider: IPX (`is` asked of `Ipx` first), else TCP/IP (asked of `TcpIp` only
/// then), else another.
pub fn provider<E>(mut is: impl FnMut(Provider) -> Result<bool, E>) -> Result<Provider, E> {
    Ok(if is(Provider::Ipx)? {
        Provider::Ipx
    } else if is(Provider::TcpIp)? {
        Provider::TcpIp
    } else {
        Provider::Other
    })
}

/// The service provider's element's size (a GUID).
pub const PROVIDER_SIZE: u32 = 0x10;
/// The port's element's size (a word).
pub const PORT_SIZE: u32 = 2;

/// The host's address's element's size: its text and the NUL.
pub fn host_address_size(length: u32) -> u32 {
    length.wrapping_add(1)
}

/// Whether TCP/IP's address has a port element: a port set (not 0).
pub fn has_port(port: u32) -> bool {
    port != 0
}

/// Whether DirectPlay's calls are made on the receiving thread (`g_netPosting` 1): for a
/// provider not IPX nor TCP/IP (a modem's or a serial link's dialogs, it seems).
pub fn posts(provider: Provider) -> bool {
    provider == Provider::Other
}

/// `CreateCompoundAddress`'s answer to the size asked: DPERR_BUFFERTOOSMALL, the one that
/// goes on.
pub const BUFFER_TOO_SMALL: u32 = 0x8877_001E;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements() {
        assert_eq!(element_at(2), 0x30);
        let e = element(&[7; 16], 0x10, 0x1234_5678);
        assert_eq!(&e[..16], &[7; 16]);
        assert_eq!(&e[16..], &[0x10, 0, 0, 0, 0x78, 0x56, 0x34, 0x12]);
        let mut asked = vec![];
        let tcp = provider(|p| -> Result<bool, ()> {
            asked.push(p);
            Ok(p == Provider::TcpIp)
        });
        assert_eq!(tcp, Ok(Provider::TcpIp));
        assert_eq!(asked, [Provider::Ipx, Provider::TcpIp]);
        assert_eq!(provider(|_| Ok::<_, ()>(true)), Ok(Provider::Ipx));
        assert_eq!(provider(|_| Ok::<_, ()>(false)), Ok(Provider::Other));
        assert!(posts(Provider::Other) && !posts(Provider::TcpIp));
        assert_eq!(host_address_size(9), 10);
        assert!(has_port(1) && !has_port(0));
    }
}
