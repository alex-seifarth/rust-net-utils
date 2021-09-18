use net_utils::*;
use std::net::{Ipv4Addr, Ipv6Addr};

#[test]
fn test_mc_socket_ip4() {
    let socket = create_std_multicast_socket_ipv4(&"239.255.255.250:1900".parse().unwrap(),
                                                  &Ipv4Addr::UNSPECIFIED);
    assert!(socket.is_ok());
    drop(socket);
}

#[test]
fn test_mc_socket_ip6() {
    let socket = create_std_multicast_socket_ipv6(&"[ff02::c]:1900".parse().unwrap(),
                                                  &Ipv6Addr::UNSPECIFIED);
    assert!(socket.is_ok());
    drop(socket);
}