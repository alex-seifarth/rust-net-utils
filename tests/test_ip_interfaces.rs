use net_utils::IpInterface;

#[test]
fn test_interface_retrieval() {
    let ipifs = IpInterface::retrieve_ip_interfaces();
    assert!(ipifs.is_ok());
}
