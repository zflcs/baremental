

use alloc::{sync::Arc, vec::Vec, boxed::Box};
use axi_ethernet::*;
use kernel_sync::SpinLock;
use spin::Lazy;

pub const AXI_ETHERNET_BASE_ADDR: usize = 0x60140000;
pub const MAC_ADDR: [u8; 6] = [0x00, 0x0A, 0x35, 0x01, 0x02, 0x03];

use axi_dma::AXI_DMA_CONFIG;

use crate::drivers::AXI_DMA;
// use smoltcp::phy::{Device, TxToken, RxToken};

pub struct NetDevice;

impl NetDevice {

    pub fn transmit(&self, data: Vec<u8>) {
        log::trace!("net transmit");
        // reclaim tx descriptor block
        // AXI_DMA.lock().tx_from_hw();
        // 初始化填充发送帧
        AXI_DMA.tx_submit(Box::pin(data)).unwrap().wait();
    }

}


pub static ETHERNET: Lazy<Arc<SpinLock<AxiEthernet>>> = Lazy::new(|| Arc::new(SpinLock::new(AxiEthernet::new(AXI_ETHERNET_BASE_ADDR, AXI_DMA_CONFIG.base_address))));


pub fn init() {
    let mut eth = ETHERNET.lock();
    eth.reset();
    let options = eth.get_options();
    eth.set_options(options | XAE_JUMBO_OPTION);
    eth.detect_phy();
    let speed = eth.get_phy_speed_ksz9031();
    debug!("speed is: {}", speed);
    eth.set_operating_speed(speed as u16);
    if speed == 0 {
        eth.link_status = LinkStatus::EthLinkDown;
    } else {
        eth.link_status = LinkStatus::EthLinkUp;
    }
    eth.set_mac_address(&MAC_ADDR);
    debug!("link_status: {:?}", eth.link_status);
    eth.enable_rx_memovr();
    eth.enable_rx_rject();
    eth.enable_rx_cmplt();
    eth.start();
}





