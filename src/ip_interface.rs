use std;
use std::ptr::null_mut;
use super::*;

/// Struct describing a single IPv4 or IPv6 capable network interface configuration.
/// Note that in a typical system a single interface (identified by its name) can have multiple
/// configurations simultaneously.
pub struct IpInterface {
    /// interface index
    pub index: u32,

    /// name of the interface
    pub name: std::string::String,

    /// flags
    pub flags: libc::c_uint,

    /// assigned IP address
    pub address: std::net::SocketAddr,

    /// network mask
    pub net_mask: std::net::SocketAddr,

    /// broadcast address
    pub broadcast_address: Option<std::net::SocketAddr>,

    /// P2P address
    pub p2p_address: Option<std::net::SocketAddr>,
}

impl IpInterface {

    /// Get the actual list of network IP interfaces and their properties from the system.
    /// The method retrieves the list of all network interfaces of the host. It then scans the
    /// interfaces for the ones which have IPv4 or IPv6 addresses and assembles them in a vector
    /// of IpInterface structs.
    /// Note that there can and will be multiple IpInterface elements in the returned list with
    /// the same interface name. This is because a single interface can have multiple configurations
    /// running simultaneously.
    pub fn retrieve_ip_interfaces() -> std::io::Result<std::vec::Vec<IpInterface>> {
        let mut p =  null_mut() as *mut libc::ifaddrs;
        let result = unsafe { libc::getifaddrs(std::ptr::addr_of_mut!(p) as *mut *mut libc::ifaddrs) };
        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut p_next = p;
        let mut vec = std::vec::Vec::new();

        while !p_next.is_null() {
            let if_info = unsafe{ *p_next };
            if let Ok(netif) = IpInterface::new_from(&if_info) {
                vec.push(netif);
            }
            p_next = if_info.ifa_next;
        }
        unsafe { libc::freeifaddrs(p) };
        Ok(vec)
    }

    /// Creates a new IpInterface from a C-struct ifaddrs.
    pub fn new_from(if_addr: &libc::ifaddrs) -> std::io::Result<IpInterface> {
        let name = match unsafe { std::ffi::CStr::from_ptr(if_addr.ifa_name.clone()) }.to_str() {
            Ok(str) => String::from(str),
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other,
                                                     "interface name seems to be invalid UTF8")),
        };

        if if_addr.ifa_addr.is_null() {
            return  Err(std::io::Error::new(std::io::ErrorKind::Other, "no address for interface"))
        }
        let address = socket_address_from(if_addr.ifa_addr)?;
        if if_addr.ifa_netmask.is_null() {
            return  Err(std::io::Error::new(std::io::ErrorKind::Other, "no netmask for interface"))
        }
        let net_mask = socket_address_from(if_addr.ifa_netmask)?;

        let broadcast_address =
            if (if_addr.ifa_flags & (libc::IFF_BROADCAST as u32)) != 0 && !if_addr.ifa_ifu.is_null() {
                 Some(socket_address_from(if_addr.ifa_ifu)?)
            } else {
                None
            };

        let p2p_address =
            if (if_addr.ifa_flags & (libc::IFF_POINTOPOINT as u32)) != 0  && !if_addr.ifa_ifu.is_null() {
                Some(socket_address_from(if_addr.ifa_ifu)?)
            } else {
                None
            };

        let index = unsafe{ libc::if_nametoindex(if_addr.ifa_name) } as u32;

        Ok( IpInterface {index, name, flags: if_addr.ifa_flags, address, net_mask, broadcast_address, p2p_address} )
    }

    /// Returns whether the interface is enabled or not. (e.g. administrative on/off of the interface).
    pub fn is_up(&self) -> bool {
        (self.flags & (libc::IFF_UP as u32)) != 0
    }

    /// Returns whether the interface has detected a physical link (layer 1) signal.
    pub fn is_l1_up(&self) -> bool {
        (self.flags & (libc::IFF_LOWER_UP as u32)) != 0
    }

    /// Returns whether this interface is a loopback/virtual interface.
    pub fn is_loopback(&self) -> bool {
        (self.flags & (libc::IFF_LOOPBACK as u32)) != 0
    }

    /// Returns whether the interface is a point-to-point link.
    pub fn is_p2p(&self) -> bool {
        (self.flags & (libc::IFF_POINTOPOINT as u32)) != 0
    }

    /// Returns whether the interface supports multicast transmission and reception.
    pub fn supports_multicast(&self) -> bool {
        (self.flags & (libc::IFF_MULTICAST as u32)) != 0
    }

    /// Returns whether the network interface address (l2-address) is dynamic and lost when the
    /// interface shuts down.
    /// @note: This is not about the IP address!
    pub fn has_dynamic_address(&self) -> bool {
        (self.flags & (libc::IFF_DYNAMIC as u32)) != 0
    }
}

