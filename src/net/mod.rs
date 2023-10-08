
mod interface;
mod socket;

use alloc::vec;
use interface::*;
use socket::*;
use smoltcp::socket::udp::{Socket, PacketBuffer, PacketMetadata};



use crate::drivers::net::*;

use smoltcp::time::Instant;

pub fn init() {
    interface::set_up();
}


pub fn net_interrupt_handler(irq: u16) {
    if irq == 2 {
        log::debug!("new mac_irq");
    } else if irq == 3 {
        log::debug!("new interrupt {:b}", AXI_NET.eth.lock().get_intr_status());
        if AXI_NET.eth.lock().is_rx_cmplt() {
            INTERFACE.lock().poll(
                Instant::ZERO, 
                unsafe { &mut *AXI_NET.as_mut_ptr() },
                &mut SOCKET_SET.lock()
            );
        } else {
            log::warn!("other interrupt happend");
        }
    }
}

pub fn udp_test() {
    let udp_rx_buffer = PacketBuffer::new(
        vec![PacketMetadata::EMPTY; 5],
        vec![0; 65535],
    );
    let udp_tx_buffer = PacketBuffer::new(
        vec![PacketMetadata::EMPTY; 5],
        vec![0; 65535],
    );
    let mut udp_socket = Socket::new(udp_rx_buffer, udp_tx_buffer);
    udp_socket.bind(80).unwrap();
    let udp_handle = SOCKET_SET.lock().add(udp_socket);
    loop {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<Socket>(udp_handle);
        if let Ok((data, endpoint)) = socket.recv() {
            debug!("udp:80 recv data: {:#x?} from {}", data, endpoint);
        }
        drop(socket);
        drop(binding);
    }
}