//! The peers of a Tailscale network, read from the text `tailscale status --json` prints. A
//! tailnet carries no broadcast, so a search for sessions probes each peer's address instead.

use std::net::Ipv4Addr;

/// The IPv4 addresses of the peers listed in `json`: every `TailscaleIPs` array after the
/// `Peer` key (the ones before it are this machine's own).
pub fn peer_addresses(json: &str) -> Vec<Ipv4Addr> {
    let Some(start) = json.find("\"Peer\"") else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut rest = &json[start..];
    while let Some(at) = rest.find("\"TailscaleIPs\"") {
        rest = &rest[at + "\"TailscaleIPs\"".len()..];
        let Some(open) = rest.find('[') else { break };
        let Some(close) = rest[open..].find(']') else {
            break;
        };
        for item in rest[open + 1..open + close].split(',') {
            let item = item.trim().trim_matches('"');
            if let Ok(ip) = item.parse::<Ipv4Addr>()
                && !found.contains(&ip)
            {
                found.push(ip);
            }
        }
        rest = &rest[open + close..];
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_peers_addresses_are_taken() {
        let json = r#"{"Version":"1.2","TailscaleIPs":["100.64.0.1","fd7a::1"],
            "Self":{"TailscaleIPs":["100.64.0.1","fd7a::1"]},
            "Peer":{"nodekey:a":{"HostName":"a","TailscaleIPs":["100.64.0.2","fd7a::2"],"Online":true},
                    "nodekey:b":{"HostName":"b","TailscaleIPs":["100.64.0.3"],"Online":false}}}"#;
        assert_eq!(
            peer_addresses(json),
            vec![Ipv4Addr::new(100, 64, 0, 2), Ipv4Addr::new(100, 64, 0, 3)]
        );
        assert!(peer_addresses(r#"{"Peer":null}"#).is_empty());
        assert!(peer_addresses("not json").is_empty());
    }
}
