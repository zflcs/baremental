#[cfg(feature = "sync")]
mod interface;
mod socket;

use alloc::{vec::Vec, vec, borrow::ToOwned};
#[cfg(feature = "sync")]
use interface::*;
use socket::*;
use smoltcp::socket::{udp, tcp};
use alloc::str;



use crate::{drivers::net::*, read_time_reg, END_TIME, START_TIME};

use smoltcp::time::Instant;

pub fn init() {
    #[cfg(feature = "sync")]
    interface::set_up();
}


#[cfg(feature = "sync")]
pub fn net_interrupt_handler(irq: u16) {
    if irq == 2 {
        log::debug!("new mac_irq");
    } else if irq == 3 {
        if AXI_NET.eth.lock().is_rx_cmplt() {
            AXI_NET.eth.lock().clear_rx_cmplt();
        } else if AXI_NET.eth.lock().is_tx_cmplt() {
            unsafe { END_TIME = read_time_reg() };
            use crate::config::{CLOCK_FREQ, USEC_PER_SEC};
            let duration = unsafe { (END_TIME - START_TIME) * USEC_PER_SEC / CLOCK_FREQ };
            log::debug!("start {}, end {}", unsafe{START_TIME}, unsafe{END_TIME});
            log::debug!("eth tx total time {} us", duration);
            log::debug!("dma tx total time {} us", unsafe { crate::DMA_TX_DURATION });
            AXI_NET.eth.lock().clear_tx_cmplt();
        } else {
            log::warn!("other interrupt happend {:b}", AXI_NET.eth.lock().get_intr_status());
        }
    }
}

#[cfg(feature = "async")]
pub fn net_interrupt_handler(irq: u16) {
    if irq == 2 {
        log::debug!("new mac_irq");
    } else if irq == 3 {
        if AXI_NET.eth.lock().is_rx_cmplt() {
            AXI_NET.eth.lock().clear_rx_cmplt();
        } else if AXI_NET.eth.lock().is_tx_cmplt() {
            unsafe { END_TIME = read_time_reg() };
            use crate::config::{CLOCK_FREQ, USEC_PER_SEC};
            let duration = unsafe { (END_TIME - START_TIME) * USEC_PER_SEC / CLOCK_FREQ };
            log::debug!("start {}, end {}", unsafe{START_TIME}, unsafe{END_TIME});
            log::debug!("eth tx total time {} us", duration);
            log::debug!("dma tx total time {} us", unsafe { crate::DMA_TX_DURATION });
            AXI_NET.eth.lock().clear_tx_cmplt();
        } else {
            log::warn!("other interrupt happend {:b}", AXI_NET.eth.lock().get_intr_status());
        }
    } else if irq == 4 {
        log::debug!("net irq 4");
        if let Some(waker) = AXI_DMA.tx_wakers.lock().pop_front() {
            waker.wake();
        }
        AXI_DMA_INTR.tx_intr_handler();
    } else if irq == 5 {
        log::debug!("net irq 5");
        if let Some(waker) = AXI_DMA.rx_wakers.lock().pop_front() {
            waker.wake();
        }
        AXI_DMA_INTR.rx_intr_handler();
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