
mod reply;

use alloc::boxed::Box;
use axi_ethernet::XAE_MAX_FRAME_SIZE;
use spin::Lazy;

use crate::drivers::{net::NetDevice, dma::AXI_DMA_INTR, AXI_DMA};

use smoltcp::wire::*;

pub struct NetStack {
    mac_addr: EthernetAddress,
    ipv4_addr: Ipv4Address,
}

impl Default for NetStack {
    fn default() -> Self {
        Self { 
            mac_addr: EthernetAddress::from_bytes(&[0x00, 0x0A, 0x35, 0x01, 0x02, 0x03]),
            ipv4_addr: Ipv4Address::new(172, 16, 1, 2),
        }
    }
}

pub static NET_STACK: Lazy<NetStack> = Lazy::new(|| NetStack::default());


pub fn net_interrupt_handler(irq: u16) {
    use crate::{net::reply::{build_arp_repr, build_eth_repr, build_eth_frame}, drivers::{net::ETHERNET, dma}};
    if irq == 2 {
        log::debug!("new mac_irq");
    } else if irq == 3 {            // maybe need to wait a moment
        log::debug!("new interrupt {:b}", ETHERNET.lock().get_intr_status());
        if ETHERNET.lock().is_rx_cmplt() {
            ETHERNET.lock().clear_rx_cmplt();
            let rx_frame = Box::pin([0u8; XAE_MAX_FRAME_SIZE]);
            let buf = AXI_DMA.rx_submit(rx_frame).unwrap().wait();
            if let Ok(eth_packet) = EthernetFrame::new_checked((*buf).as_ref()) {
                match eth_packet.ethertype() {
                    EthernetProtocol::Arp => {
                        if let Ok(arp_packet) = ArpPacket::new_checked(eth_packet.payload()) {
                            if arp_packet.operation() == ArpOperation::Request {
                                let dst_mac_addr = EthernetAddress::from_bytes(arp_packet.source_hardware_addr());
                                let arp_repr = build_arp_repr(
                                    NET_STACK.mac_addr, 
                                    NET_STACK.ipv4_addr, 
                                    dst_mac_addr,
                                    Ipv4Address::from_bytes(arp_packet.source_protocol_addr())    
                                );
                                let eth_repr = build_eth_repr(
                                    NET_STACK.mac_addr, 
                                    dst_mac_addr, 
                                    EthernetProtocol::Arp
                                );
                                for _ in 0..400 {
                                    if let Some(eth_frame) = build_eth_frame(eth_repr, Some(arp_repr), None) {
                                        NetDevice.transmit(eth_frame.into_inner());
                                    }
                                }
                            } else {
                                log::trace!("don't need to reply")
                            }
                        } else {
                            log::trace!("Cannot analysis Arp protocol");
                        }
                    },
                    _ => { log::trace!("Protocol is not supported"); }
                }
            } else {
                log::trace!("do nothing");
            }
        } else {
            log::warn!("other interrupt happend");
        }
    } else if irq == 4 {
        log::debug!("new mm2s intr");
        if !AXI_DMA_INTR.tx_intr_handler() {
            dma::init();
        }
        AXI_DMA.tx_from_hw();
    } else if irq == 5 {
        log::debug!("new s2mm intr");
        if !AXI_DMA_INTR.rx_intr_handler() {
            dma::init();
        }
        AXI_DMA.rx_from_hw();
    }
}