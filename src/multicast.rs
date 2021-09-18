use std::{
    net::{SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr},
    io::{Result, Error, ErrorKind},
    os::unix::io::FromRawFd
};

use super::IpInterface;

/// Creates a std::net::UdpSocket for multicast reception with SO_REUSEADDR set for IPv4.
/// # Arguments
/// * mc_address    The multicast IPv4 address. The socket will only receive from this address/port.
/// * interface     The local address will determine the interface from which multicast messages
///                 can be received and this address will also be used as source for sent packets.
pub fn create_std_multicast_socket_ipv4(mc_address: &SocketAddrV4, interface: &Ipv4Addr)
                                        -> Result<std::net::UdpSocket> {
    if !mc_address.ip().is_multicast() {
        return Err(Error::new(ErrorKind::InvalidInput, "mc_address is not multicast"));
    }
    let socket_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    set_socket_reuseaddr(&socket_fd)?;

    let mc_addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: mc_address.port().to_be(),
        sin_addr: libc::in_addr { s_addr: u32::from(mc_address.ip().clone()).to_be() },
        sin_zero: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    };
    bind_socket(&socket_fd, &mc_addr)?;

    let socket = unsafe{ std::net::UdpSocket::from_raw_fd(socket_fd) };
    if let Err(e) = socket.join_multicast_v4(mc_address.ip(), interface) {
        return Err(e);
    }
    Ok(socket)
}

/// Creates a std::net::UdpSocket for multicast reception with SO_REUSEADDR set for IPv6.
/// # Arguments
/// * mc_address    The multicast IPv6 address. The socket will only receive from this address/port.
///                 Note that the function ignores the address' scope id and uses the second octet
///                 from the IP address instead.
/// * interface     The local address will determine the interface from which multicast messages
///                 can be received and this address will also be used as source for sent packets.
pub fn create_std_multicast_socket_ipv6(mc_address: &SocketAddrV6, interface: &Ipv6Addr)
                                        -> Result<std::net::UdpSocket> {
    if !mc_address.ip().is_multicast() {
        return Err(Error::new(ErrorKind::InvalidInput, "mc_address is not multicast"));
    }
    let socket_fd = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0) };
    set_socket_reuseaddr(&socket_fd)?;

    let mc_addr = libc::sockaddr_in6 {
        sin6_family: libc::AF_INET6 as u16,
        sin6_port: mc_address.port().to_be(),
        sin6_flowinfo: mc_address.flowinfo().to_be(),
        sin6_addr: libc::in6_addr { s6_addr: mc_address.ip().octets() },
        sin6_scope_id: mc_address.ip().octets()[1] as u32,
    };
    bind_socket(&socket_fd, &mc_addr)?;

    let socket = unsafe{ std::net::UdpSocket::from_raw_fd(socket_fd) };
    let intf_idx = find_interface_index(interface)?;
    if let Err(e) = socket.join_multicast_v6(mc_address.ip(), intf_idx) {
        return Err(e);
    }
    Ok(socket)
}

/// Creates a std::tokio::UdpSocket for multicast reception with SO_REUSEADDR set for IPv4.
/// Requires the feature 'tokio-net'.
/// # Arguments
/// * mc_address    The multicast IPv4 address. The socket will only receive from this address/port.
/// * interface     The local address will determine the interface from which multicast messages
///                 can be received and this address will also be used as source for sent packets.
#[cfg(feature = "tokio-net")]
pub fn create_tokio_multicast_socket_ipv4(mc_address: &SocketAddrV4, interface: &Ipv4Addr)
                                          -> Result<tokio::net::UdpSocket> {
    let std_socket = create_std_multicast_socket_ipv4(mc_address, interface)?;
    std_socket.set_nonblocking(true)?;
    tokio::net::UdpSocket::from_std(std_socket)
}

/// Creates a std::tokio::UdpSocket for multicast reception with SO_REUSEADDR set for IPv6.
/// Requires the feature 'tokio-net'.
/// # Arguments
/// * mc_address    The multicast IPv6 address. The socket will only receive from this address/port.
/// * interface     The local address will determine the interface from which multicast messages
///                 can be received and this address will also be used as source for sent packets.
#[cfg(feature = "tokio-net")]
pub fn create_tokio_multicast_socket_ipv6(mc_address: &SocketAddrV6, interface: &Ipv6Addr)
                                          -> Result<tokio::net::UdpSocket> {
    let std_socket = create_std_multicast_socket_ipv6(mc_address, interface)?;
    std_socket.set_nonblocking(true)?;
    tokio::net::UdpSocket::from_std(std_socket)
}

/// Sets the SO_REUSEADDR option on the raw socket
fn set_socket_reuseaddr(socket: &libc::c_int) -> Result<()> {
    let optval: libc::c_int = 1;
    if unsafe { libc::setsockopt(*socket, libc::SOL_SOCKET, libc::SO_REUSEADDR,
                                 &optval as *const _ as *const libc::c_void,
                                 std::mem::size_of_val(&optval) as libc::socklen_t) } != 0 {
        unsafe{ libc::close(*socket) };
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Bind the socket to the given address
fn bind_socket<T>(socket: &libc::c_int, addr: &T) -> Result<()> {
    if unsafe{ libc::bind(*socket, std::ptr::addr_of!(*addr) as *const libc::sockaddr,
                          std::mem::size_of_val(addr) as libc::socklen_t) } != 0 {
        unsafe{ libc::close(*socket) };
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Searches for an IP multicast capable interface with the given address and returns its index.
/// If no interface is found Ok(0) is returned, where 0 can be used as ANY_INTERFACE.
fn find_interface_index(addr: &Ipv6Addr) -> Result<u32> {
    let intfs = IpInterface::retrieve_ip_interfaces()?;
    for intf in intfs.iter() {
        if intf.address.ip() == *addr && intf.supports_multicast() {
            return Ok(intf.index)
        }
    }
    Ok(0)
}
