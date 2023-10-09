
mod interface;
mod socket;

use alloc::{vec::Vec, vec, borrow::ToOwned};
use interface::*;
use socket::*;
use smoltcp::socket::{udp, tcp};
use alloc::str;



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
    let udp_rx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; 5],
        vec![0; 65535],
    );
    let udp_tx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; 5],
        vec![0; 65535],
    );
    let mut udp_socket = udp::Socket::new(udp_rx_buffer, udp_tx_buffer);
    udp_socket.bind(80).unwrap();
    let udp_handle = SOCKET_SET.lock().add(udp_socket);
    loop {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(udp_handle);
        if let Ok((data, endpoint)) = socket.recv() {
            debug!("udp:80 recv data: {:#x?} from {}", data, endpoint);
        }
        drop(socket);
        drop(binding);
    }
}

pub fn tcp_test() {
    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 9000]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 9000]);
    let mut tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
    tcp_socket.listen(80).unwrap();
    let tcp_handle = SOCKET_SET.lock().add(tcp_socket);
    loop {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(tcp_handle);
        if socket.may_recv() {
            let data = socket
                .recv(|data| {
                    let mut data = data.to_owned();
                    if !data.is_empty() {
                        debug!(
                            "recv data: {:?}",
                            str::from_utf8(data.as_ref()).unwrap_or("(invalid utf8)")
                        );
                        data = data.split(|&b| b == b'\n').collect::<Vec<_>>().concat();
                        data.reverse();
                    }
                    (data.len(), data)
                })
                .unwrap();
            if socket.can_send() && !data.is_empty() {
                debug!(
                    "send data: {:?}",
                    str::from_utf8(data.as_ref()).unwrap_or("(invalid utf8)")
                );
                socket.send_slice(&data[..]).unwrap();
            }
        }
        drop(socket);
        drop(binding);
    }
}