
use alloc::{sync::Arc, boxed::Box, vec};
use axi_dma::{AxiDmaIntr, AxiDma};
use axi_ethernet::{AxiEthernet, XAE_JUMBO_OPTION, LinkStatus};
use kernel_sync::SpinLock;
use spin::Lazy;
use crate::config::{AXI_DMA_CONFIG, AXI_NET_CONFIG};
use smoltcp::phy::{Device, RxToken, TxToken, DeviceCapabilities, Medium};

#[derive(Clone)]
pub struct AxiNet {
    pub dma: Arc<AxiDma>,
    pub dma_intr: Arc<AxiDmaIntr>,
    pub eth: Arc<SpinLock<AxiEthernet>>,
}

impl AxiNet {
    pub const fn new(
        dma: Arc<AxiDma>,
        dma_intr: Arc<AxiDmaIntr>,
        eth: Arc<SpinLock<AxiEthernet>>
    ) -> Self {
        Self { dma, dma_intr, eth }
    }
}

impl Default for AxiNet {
    fn default() -> Self {
        AxiNet::new(AXI_DMA.clone(), AXI_DMA_INTR.clone(), AXI_ETH.clone())
    }
}

pub static AXI_NET: Lazy<AxiNet> = Lazy::new(|| AxiNet::default());

pub static AXI_ETH: Lazy<Arc<SpinLock<AxiEthernet>>> = Lazy::new(||  Arc::new(SpinLock::new(AxiEthernet::new(
    AXI_NET_CONFIG.eth_baseaddr, AXI_NET_CONFIG.dma_baseaddr
))));

pub static AXI_DMA_INTR: Lazy<Arc<AxiDmaIntr>> = Lazy::new(|| AxiDmaIntr::new(AXI_DMA_CONFIG.base_address));

pub static AXI_DMA: Lazy<Arc<AxiDma>> = Lazy::new(|| AxiDma::new(AXI_DMA_CONFIG));


pub fn init() {
    dma_init();
    eth_init();
}

pub fn dma_init() {
    AXI_DMA.reset();
    // enable cyclic mode
    AXI_DMA.tx_cyclic_enable();
    AXI_DMA.rx_cyclic_enable();

    // init cyclic block descriptor
    AXI_DMA.tx_bd_create(AXI_NET_CONFIG.tx_bd_cnt);
    AXI_DMA.rx_bd_create(AXI_NET_CONFIG.rx_bd_cnt);

    // enable tx & rx intr
    AXI_DMA.tx_intr_enable();
    AXI_DMA.rx_intr_enable();
}

pub fn eth_init() {
    let mut eth = AXI_ETH.lock();
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
    eth.set_mac_address(&AXI_NET_CONFIG.mac_addr);
    debug!("link_status: {:?}", eth.link_status);
    eth.enable_rx_memovr();
    eth.enable_rx_rject();
    eth.enable_rx_cmplt();
    eth.start();
}

#[cfg(feature = "sync")]
impl RxToken for AxiNet {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R {
        let rx_frame = Box::pin([0u8; AXI_NET_CONFIG.mtu]);
        let mut buf = self.dma.rx_submit(rx_frame).unwrap().wait();
        log::trace!("receive buf {:x?}", &buf[0..14]);
        f((*buf).as_mut())
    }
}

#[cfg(feature = "sync")]
impl TxToken for AxiNet {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R {
        let mut tx_frame = Box::pin(vec![0u8; len]);
        let res = f((*tx_frame).as_mut());
        self.dma.tx_submit(tx_frame).unwrap().wait();
        log::trace!("transmit buf");
        res
    }
}

#[cfg(feature = "sync")]
impl Device for AxiNet {
    type RxToken<'a> = Self;
    type TxToken<'a> = Self;

    fn receive(&mut self, _timestamp: smoltcp::time::Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.eth.lock().is_rx_cmplt() {
            self.eth.lock().clear_rx_cmplt();
        }
        if self.eth.lock().can_receive() {
            Some((self.clone(), self.clone()))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        Some(self.clone())
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = AXI_NET_CONFIG.mtu;
        caps.max_burst_size = Some(1);
        caps
    }
}
