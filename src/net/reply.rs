

pub(crate) const ARP_LEN: usize = 28;

use alloc::{vec, vec::Vec};
use smoltcp::wire::*;
use smoltcp::phy::ChecksumCapabilities;


pub fn build_eth_repr(src_mac_addr: EthernetAddress, dst_mac_addr: EthernetAddress, ethertype: EthernetProtocol) -> EthernetRepr {
    EthernetRepr {
        src_addr: src_mac_addr,
        dst_addr: dst_mac_addr,
        ethertype,
    }
}

pub fn build_arp_repr(src_mac_addr: EthernetAddress, src_ip: Ipv4Address, dst_mac_addr: EthernetAddress, dst_ip: Ipv4Address) -> ArpRepr {
    ArpRepr::EthernetIpv4 { 
        operation: ArpOperation::Reply, 
        source_hardware_addr: src_mac_addr, 
        source_protocol_addr: src_ip, 
        target_hardware_addr: dst_mac_addr, 
        target_protocol_addr: dst_ip, 
    }
}

pub fn build_ipv4_repr(src_ip: Ipv4Address, dst_ip: Ipv4Address, protocol: IpProtocol, payload_len: usize) -> Ipv4Repr {
    Ipv4Repr {
        src_addr: src_ip,
        dst_addr: dst_ip,
        next_header: protocol,
        payload_len,
        hop_limit: 128,
    }
}

// generate the tcp repr from the receive TcpPacket
pub fn build_tcp_ack_repr<'a>(packet: &TcpPacket<&'a [u8]>) -> TcpRepr<'a> {
    pub(crate) const TCP_EMPTY_DATA: &[u8] = &[];
    let src_port = packet.dst_port();
    let dst_port = packet.src_port();
    let control = if packet.syn() {
        TcpControl::Syn
    } else if packet.fin() {
        TcpControl::Fin
    } else {
        TcpControl::None
    };
    let ack_number = packet.seq_number() + packet.segment_len();
    let seq_number = if packet.ack() {
        packet.ack_number()
    } else {
        TcpSeqNumber::default()
    };
    TcpRepr {
        src_port,
        dst_port,
        control,
        seq_number,
        ack_number: Some(ack_number),
        window_len: packet.window_len(),
        window_scale: Some(8),
        max_seg_size: Some(1460),
        sack_permitted: false,
        sack_ranges: [None; 3],
        payload: TCP_EMPTY_DATA,
    }
}

pub fn build_tcp_repr<'a>(
    src_port: u16, 
    dst_port: u16, 
    control: TcpControl, 
    seq_number: TcpSeqNumber, 
    ack_number: Option<TcpSeqNumber>,
    payload: &'a [u8]
) -> TcpRepr<'a> {
    TcpRepr {
        src_port,
        dst_port,
        control,
        seq_number,
        ack_number,
        window_len: 1460,
        window_scale: None,
        max_seg_size: Some(1460),
        sack_permitted: false,
        sack_ranges: [None; 3],
        payload,
    }
}

pub fn build_eth_frame(eth_repr: EthernetRepr, arp_repr: Option<ArpRepr>, ipv4_repr: Option<(Ipv4Repr, TcpRepr)>) -> Option<EthernetFrame<Vec<u8>>> {
    if let Some(arp_repr) = arp_repr {
        let mut buf = vec![0u8; ETHERNET_HEADER_LEN + ARP_LEN];
        let mut arp_packet = ArpPacket::new_checked(&mut buf[ETHERNET_HEADER_LEN..]).unwrap();
        arp_repr.emit(&mut arp_packet);
        let mut eth_frame = EthernetFrame::new_checked(buf).unwrap();
        eth_repr.emit(&mut eth_frame);
        Some(eth_frame)

    } else {
        if let Some((ipv4_repr, tcp_repr)) = ipv4_repr {
            let checksum_ability = ChecksumCapabilities::default();
            let mut buf = vec![0u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + tcp_repr.buffer_len()];

            let mut tcp_packet = TcpPacket::new_unchecked(&mut buf[ETHERNET_HEADER_LEN + IPV4_HEADER_LEN..]);
            tcp_repr.emit(&mut tcp_packet, &ipv4_repr.src_addr.into_address(), &ipv4_repr.dst_addr.into_address(), &checksum_ability);

            let mut ipv4_packet = Ipv4Packet::new_checked(&mut buf[ETHERNET_HEADER_LEN..]).unwrap();
            ipv4_repr.emit(&mut ipv4_packet, &checksum_ability);

            let mut eth_frame = EthernetFrame::new_checked(buf).unwrap();
            eth_repr.emit(&mut eth_frame);
            Some(eth_frame)
        } else {
            None
        }
    }
}