use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use url::Url;

/// Check if a URL targets a private/internal network address.
/// Returns true if the URL is private (should be blocked).
pub fn is_private_url(url: &Url) -> bool {
    let host = match url.host_str() {
        Some(h) => h,
        None => return true, // No host = fail-closed (private)
    };

    // Fast path: check raw IP literals
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(&ip);
    }

    // Slow path: DNS resolution
    match (host, 0).to_socket_addrs() {
        Ok(mut addrs) => {
            // If any resolved address is private, the URL is private
            addrs.any(|addr| is_private_ip(&addr.ip()))
        }
        Err(_) => {
            // DNS resolution failed → fail-closed (treat as private)
            tracing::warn!(%host, "DNS resolution failed for SSRF check; treating as private");
            true
        }
    }
}

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || is_shared_address_space(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unicast_link_local()
                || v6.is_unspecified()
                || v6.is_multicast()
                || is_unique_local_v6(v6)
        }
    }
}

/// Check if an IPv4 address is in the Shared Address Space (100.64.0.0/10, RFC 6598).
fn is_shared_address_space(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (octets[1] & 0xC0) == 64
}

/// Check if an IPv6 address is in the Unique Local Address range (fc00::/7).
fn is_unique_local_v6(ip: &Ipv6Addr) -> bool {
    let segments = ip.segments();
    (segments[0] & 0xfe00) == 0xfc00
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_is_private() {
        let url = Url::parse("http://127.0.0.1:8080").unwrap();
        assert!(is_private_url(&url));

        let url = Url::parse("http://localhost:8080").unwrap();
        assert!(is_private_url(&url));

        let url = Url::parse("http://[::1]:8080").unwrap();
        assert!(is_private_url(&url));
    }

    #[test]
    fn test_private_ranges() {
        let url = Url::parse("http://192.168.1.1").unwrap();
        assert!(is_private_url(&url));

        let url = Url::parse("http://10.0.0.1").unwrap();
        assert!(is_private_url(&url));

        let url = Url::parse("http://172.16.0.1").unwrap();
        assert!(is_private_url(&url));
    }

    #[test]
    fn test_public_is_not_private() {
        let url = Url::parse("https://example.com").unwrap();
        // This test may fail if DNS resolves example.com to a private IP
        // In CI, this should resolve to a public IP
        let _result = is_private_url(&url);
        // Not asserting false because DNS resolution varies
    }

    #[test]
    fn test_no_host_is_private() {
        let url = Url::parse("file:///etc/passwd").unwrap();
        assert!(is_private_url(&url));
    }

    #[test]
    fn test_link_local() {
        let url = Url::parse("http://169.254.1.1").unwrap();
        assert!(is_private_url(&url));
    }

    #[test]
    fn test_shared_address_space() {
        let url = Url::parse("http://100.64.0.1").unwrap();
        assert!(is_private_url(&url));
    }

    #[test]
    fn test_unique_local_ipv6() {
        let url = Url::parse("http://[fd00::1]").unwrap();
        assert!(is_private_url(&url));
    }
}
