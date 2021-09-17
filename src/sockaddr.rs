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
