//! Builds the connection's DirectPlay address with the lobby object ("NET: DPCA"). IPX gets the
//! service provider alone; TCP/IP the provider, the host's address and the port if set; any other
//! provider the record's service provider and `posting` 1 (its calls then run on the receiving
//! thread, for modem and serial dialogs). Returns false when there is no lobby object, the size
//! query is unexpected or `CreateCompoundAddress` fails.

use crate::errors::{BadInput, Refused};
use crate::net::add_address_element::{self as elements, add_address_element};
use crate::net::lobby::log_bytes;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{NetTexts, texts};
use rc2k_core::net::address::{self, Provider};
pub fn build_address(net: &Net, lines: &NetTexts, connection_number: u32) -> Result<bool, Refused> {
    let (vars, link) = (&net.vars, &net.link);
    log_bytes(&lines.text(texts::NET_DPCA), true, 2);
    if link.lobby() == 0 {
        return Ok(false);
    }
    let guid = crate::net::tables::with_connection(link, connection_number, |c| c.provider)?;
    elements::clear(net);
    net.address().address.set(0);
    net.address().address_size.set(0);
    vars.receiver.posting.set(0);
    let provider = address::provider(|_| -> Result<bool, Refused> {
        Ok(crate::net::store::SP_TCPIP == guid)
    })?;
    match provider {
        Provider::TcpIp => {
            add_address_element(
                net,
                &crate::net::store::AID_SERVICE_PROVIDER,
                &crate::net::store::SP_TCPIP,
            );
            let host = net.setup().host_address.read();
            let length = host
                .iter()
                .position(|&c| c == 0)
                .ok_or(BadInput("a host address without its end"))? as u32;
            let size = address::host_address_size(length) as usize;
            add_address_element(net, &crate::net::store::AID_INET, &host[..size]);
            if address::has_port(net.setup().port.get()) {
                let port = net.setup().port.get().to_le_bytes();
                add_address_element(
                    net,
                    &crate::net::store::AID_INET_PORT,
                    &port[..address::PORT_SIZE as usize],
                );
            }
        }
        Provider::Other => {
            let data = &guid[..address::PROVIDER_SIZE as usize];
            add_address_element(net, &crate::net::store::AID_SERVICE_PROVIDER, data);
        }
    }
    if address::posts(provider) {
        vars.receiver.posting.set(1);
    }
    let parts = elements::elements(net);
    let (result, size) =
        crate::net::dp_objects::create_compound_address(&parts, net.address().address_size.get());
    net.address().address_size.set(size);
    if result != address::BUFFER_TOO_SMALL {
        return Ok(false);
    }
    crate::net::tables::count(link, net.address().address_size.get())?;
    let made = crate::net::dp_objects::compound_address(&parts);
    let fits = made.len() as u32 <= net.address().address_size.get();
    link.tables().with(|t| t.address = made)?;
    Ok(fits)
}
