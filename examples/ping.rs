//! IPv4 ping example.
//!
//! Much of this code was borrowed from [the smoltcp ping example](https://github.com/smoltcp-rs/smoltcp/blob/main/examples/ping.rs).

#![no_main]
#![no_std]

use core::str::FromStr;

use log::info;
use smoltcp::{
    iface::{Config, Interface, SocketSet, SocketStorage},
    phy::Device,
    socket::icmp,
    wire::{
        HardwareAddress, Icmpv4Packet, Icmpv4Repr, IpAddress, IpCidr, Ipv4Address, Ipv6Address,
    },
};
use smoltcp_uefi::{device::SnpDevice, time::shitty_now_from_processor_clock};
use uefi::{boot::ScopedProtocol, prelude::*, proto::network::snp::SimpleNetwork};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("hello world from ping example!");

    let (h, snp) = init_network().unwrap();
    let snp = snp.get().unwrap();
    print_snp_status(&h, snp);

    send_loop(snp);

    Status::SUCCESS
}

fn send_loop(snp: &SimpleNetwork) {
    let mut device = SnpDevice::<1536>::new(snp).unwrap();
    let mut iface = Interface::new(
        Config::new(HardwareAddress::Ethernet(device.permanent_address())),
        &mut device,
        shitty_now_from_processor_clock(),
    );

    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(192, 168, 122, 69), 24))
            .unwrap();
        ip_addrs
            .push(IpCidr::new(
                IpAddress::v6(0xfe80, 0, 0, 0, 0, 0, 0x12, 0x34),
                64,
            ))
            .unwrap();
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(192, 168, 122, 0))
        .unwrap();
    iface
        .routes_mut()
        .add_default_ipv6_route(Ipv6Address::new(0xfe80, 0, 0, 0, 0, 0, 0, 0x100))
        .unwrap();

    macro_rules! make_buffer {
        ($name:ident) => {
            let mut metadata = [icmp::PacketMetadata::EMPTY; 256];
            let mut payload = [0u8; 256];
            let $name =
                icmp::PacketBuffer::new(&mut metadata as &mut [_], &mut payload as &mut [_]);
        };
    }

    make_buffer!(icmp_rx_buffer);
    make_buffer!(icmp_tx_buffer);
    let sockets = &mut [SocketStorage::EMPTY] as &mut [SocketStorage<'_>];
    let mut sockets = SocketSet::new(sockets);
    let handle = sockets.add(icmp::Socket::new(icmp_rx_buffer, icmp_tx_buffer));
    let caps = device.capabilities();

    let ident = 0x1234;
    let mut sent_packets = 0u64;
    loop {
        iface.poll(shitty_now_from_processor_clock(), &mut device, &mut sockets);

        let icmp_socket = sockets.get_mut::<icmp::Socket>(handle);
        if !icmp_socket.is_open() {
            icmp_socket.bind(icmp::Endpoint::Ident(ident)).unwrap();
        }

        if icmp_socket.can_send() {
            let icmp_repr = Icmpv4Repr::EchoRequest {
                ident,
                seq_no: sent_packets as u16,
                data: b"OwO",
            };
            let payload = icmp_socket
                .send(
                    icmp_repr.buffer_len(),
                    IpAddress::from_str("192.168.122.1").unwrap(),
                )
                .unwrap();
            let mut packet = Icmpv4Packet::new_unchecked(payload);
            icmp_repr.emit(&mut packet, &caps.checksum);
        };

        info!("tx {}", sent_packets);

        // poll repeatedly
        loop {
            let result = iface.poll(shitty_now_from_processor_clock(), &mut device, &mut sockets);

            match result {
                smoltcp::iface::PollResult::None => continue,
                smoltcp::iface::PollResult::SocketStateChanged => break,
            }
        }

        sent_packets += 1;

        boot::stall(1_000_000);
    }
}

fn init_network() -> uefi::Result<(Handle, ScopedProtocol<SimpleNetwork>)> {
    let handle = boot::get_handle_for_protocol::<SimpleNetwork>()?;
    let snp = boot::open_protocol_exclusive::<SimpleNetwork>(handle)?;
    snp.start()?;
    snp.initialize(0, 0)?;
    Ok((handle, snp))
}

fn print_snp_status(h: &Handle, snp: &SimpleNetwork) {
    let m = snp.mode();
    info!(
        "found snp on handle {:?} with mac {:x?}",
        h.as_ptr(),
        &m.current_address.0[..6]
    );
    info!("  snp is in mode {:?}", m.state);
    info!("  media header size {:?}", m.media_header_size);
    info!("  max packet size {:?}", m.max_packet_size);
    info!(
        "  media_present_supported? {}; media_present? {}",
        m.media_present_supported, m.media_present,
    );
    info!("  interface type: {}", m.if_type);
}
