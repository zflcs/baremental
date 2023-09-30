
use lazy_static::lazy_static;
use alloc::sync::Arc;
use axi_dma::{AxiDma, AxiDmaIntr, AXI_DMA_CONFIG};
use axi_ethernet::XAE_MAX_JUMBO_FRAME_SIZE;

static mut RX_BUFFER: [u8; XAE_MAX_JUMBO_FRAME_SIZE] = [0u8; XAE_MAX_JUMBO_FRAME_SIZE];

lazy_static! {
    pub static ref AXI_DMA: Arc<AxiDma> = AxiDma::new(AXI_DMA_CONFIG);
    pub static ref AXI_DMA_INTR: Arc<AxiDmaIntr> = AxiDmaIntr::new(AXI_DMA_CONFIG.base_address);
}


const RX_BD_CNT: usize = 500;
const TX_BD_CNT: usize = 500;


pub fn init() {
    AXI_DMA.reset();
    AXI_DMA.tx_cyclic_enable();
    AXI_DMA.rx_cyclic_enable();

    // 初始化 BD
    AXI_DMA.tx_bd_create(TX_BD_CNT);
    AXI_DMA.rx_bd_create(RX_BD_CNT);
    // 中断使能
    AXI_DMA.tx_intr_enable();
    AXI_DMA.rx_intr_enable();
    // 提交接收的缓冲区
    // AXI_DMA.rx_submit();
}
