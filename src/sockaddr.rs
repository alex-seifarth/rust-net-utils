use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

/// Creates a new SocketAddr from a libc::sockaddr for IPv4 or IPv6 addresses.
pub fn socket_address_from(sockad_raw: *const libc::sockaddr) -> std::io::Result<std::net::SocketAddr> {
    let sockad = unsafe{ *sockad_raw };
    match sockad.sa_family as i32 {
        libc::AF_INET   => {
            let addr4 = unsafe{ *(sockad_raw as *const libc::sockaddr_in) };
            Ok( SocketAddr::V4( std::net::SocketAddrV4::new(
                Ipv4Addr::from(u32::from_be(addr4.sin_addr.s_addr)), u16::from_be(addr4.sin_port)
            ) ) )
        },
        libc::AF_INET6  => {
            let addr6 = unsafe{ *(sockad_raw as *const libc::sockaddr_in6) };
            Ok( SocketAddr::V6( std::net::SocketAddrV6::new(
                Ipv6Addr::from(addr6.sin6_addr.s6_addr), u16::from_be(addr6.sin6_port as u16),
                u32::from_be(addr6.sin6_flowinfo), u32::from_be(addr6.sin6_scope_id)
            ) ) )
        },
        _ => { Err(std::io::Error::new(std::io::ErrorKind::Other, "not an IP or IP6 address")) },
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::net::{SocketAddrV4, SocketAddrV6};

    #[test]
    fn test_ipv4() {
        let data = [
            (4711 as u16, 0x11223344 as u32),
            ( 501 as u16, 0x01000080 as u32)
        ];

        for d in data.iter() {
            let ad = libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: d.0.to_be(),
                sin_addr: libc::in_addr { s_addr: d.1.to_be() },
                sin_zero: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            };
            let address_result = socket_address_from(std::ptr::addr_of!(ad) as *const libc::sockaddr);
            assert!(address_result.is_ok());
            if let Ok(address) = address_result {
                assert_eq!(address, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(d.1), d.0)));
            };
        }
    }

    #[test]
    fn test_ipv6() {
        let data = [
            (5433 as u16, 0 as u32, 12 as u32, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x00]),
            (80 as u16, 1230 as u32, 98400 as u32, [0x21, 0x22, 0x23, 0x34, 0x35, 0x36, 0x47, 0x48, 0x49, 0x5a, 0x5b, 0x5c, 0x6d, 0x6e, 0x7f, 0x80]),
        ];
        for d in data.iter() {
            let ad = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as u16,
                sin6_port: d.0.to_be(),
                sin6_flowinfo: d.1.to_be(),
                sin6_addr: libc::in6_addr{ s6_addr: d.3 },
                sin6_scope_id: d.2.to_be(),
            };
            let address_result = socket_address_from(std::ptr::addr_of!(ad) as *const libc::sockaddr);
            assert!(address_result.is_ok());
            if let Ok(address) = address_result {
                assert_eq!(address, SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(d.3), d.0, d.1, d.2)));
            };
        }
    }
}
