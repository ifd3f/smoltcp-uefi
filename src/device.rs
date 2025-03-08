use log::{error, trace};
use smoltcp::{
    phy::{Device, DeviceCapabilities, Medium},
    wire::EthernetAddress,
};
use uefi::proto::network::snp::{InterruptStatus, SimpleNetwork};

use crate::convert::u2s_mac_address;

/// A smoltcp [Device] based on a uefi [SimpleNetwork].
///
/// This device assumes that your [SimpleNetwork] has already been initialized.
pub struct SnpDevice<'a> {
    snp: &'a SimpleNetwork,
}

impl<'a> SnpDevice<'a> {
    pub fn new(snp: &'a SimpleNetwork) -> Self {
        Self { snp }
    }

    /// Get the current MAC address configured on the underlying [SimpleNetwork].
    pub fn current_address(&self) -> EthernetAddress {
        u2s_mac_address(&self.snp.mode().current_address)
    }

    /// Get the permanent MAC address configured on the underlying [SimpleNetwork].
    pub fn permanent_address(&self) -> EthernetAddress {
        u2s_mac_address(&self.snp.mode().permanent_address)
    }

    /// Get a reference to the underlying [SimpleNetwork].
    pub fn snp(&self) -> &SimpleNetwork {
        self.snp
    }
}

impl<'a> Device for SnpDevice<'a> {
    type RxToken<'b>
        = SnpRxToken
    where
        Self: 'b;

    type TxToken<'b>
        = SnpTxToken<'b>
    where
        Self: 'b;

    fn receive(
        &mut self,
        _timestamp: smoltcp::time::Instant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // TODO

        /*
        let mut rx = SnpRxToken {
            packet: [0; 1536],
            size: 0,
        };
        let tx = SnpRxToken { snp: &self.snp };
        match rx_mac_frame(self.snp, &mut rx.packet) {
            Ok(size) => {
                rx.size = size;
                Some((rx, tx))
            }
            Err(_) => None,
        } */

        None
    }

    fn transmit(&mut self, _timestamp: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        Some(SnpTxToken { snp: &self.snp })
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = self.snp.mode().max_packet_size as usize;
        caps
    }
}

pub struct SnpRxToken {
    packet: [u8; 1536],
    size: usize,
}

impl smoltcp::phy::RxToken for SnpRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.packet[..self.size])
    }
}

pub struct SnpTxToken<'a> {
    snp: &'a SimpleNetwork,
}

impl<'a> smoltcp::phy::TxToken for SnpTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = [0u8; 1536];
        let packet = &mut buf[..len];
        let result = f(packet);

        trace!("Transmitting {:x?}", packet);

        self.snp.get_interrupt_status().unwrap();

        self.snp
            .transmit(0, packet, None, None, None)
            .inspect_err(|e| {
                error!("error during tx! {e}");
            })
            .ok();

        result
    }
}
