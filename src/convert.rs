//! Helpers for conversions between smoltcp and uefi types.

/// Convert a [`uefi::proto::network::MacAddress`] into a [`smoltcp::wire::EthernetAddress`].
pub fn u2s_mac_address(a: &uefi::proto::network::MacAddress) -> smoltcp::wire::EthernetAddress {
    let mut out = smoltcp::wire::EthernetAddress([0; 6]);
    out.0.copy_from_slice(&a.0[..6]);
    out
}
