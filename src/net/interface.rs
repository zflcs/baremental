use alloc::sync::Arc;
use kernel_sync::SpinLock;
use smoltcp::{
    iface::{Interface, Config},
    wire::{EthernetAddress, IpCidr, IpAddress}, 
    time::Instant,
};
use spin::Lazy;

use crate::{config::AXI_NET_CONFIG, drivers::AXI_NET};




pub static INTERFACE: Lazy<Arc<SpinLock<Interface>>> = Lazy::new(|| Arc::new(SpinLock::new(
    Interface::new(
            Config::new(EthernetAddress(AXI_NET_CONFIG.mac_addr).into()),
            unsafe { &mut *AXI_NET.as_mut_ptr() },
            Instant::ZERO
        )
)));

pub fn set_up() {
    INTERFACE.lock().update_ip_addrs(|ip_addrs|
        ip_addrs.push(IpCidr::new(IpAddress::v4(172, 16, 1, 2), 30)).unwrap()
    );
}




