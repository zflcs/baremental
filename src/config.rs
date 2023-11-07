/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x4_0000;

/// Total boot kernel size.
pub const TOTAL_BOOT_STACK_SIZE: usize = BOOT_STACK_SIZE * CPU_NUM;


pub const KERNEL_HEAP_SIZE: usize = 0x100_000;


pub const CLOCK_FREQ: usize = 10_000_000;
// pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;


pub const CPU_NUM: usize = 1;

use axi_dma::AxiDmaConfig;
use axi_ethernet::XAE_MAX_JUMBO_FRAME_SIZE;

pub const AXI_DMA_CONFIG: AxiDmaConfig = AxiDmaConfig {
    device_id: 0,
    base_address: 0x6010_0000,
    has_sts_cntrl_strm: false,
    is_micro_dma: false,
    has_mm2s: true,
    has_mm2s_dre: false,
    mm2s_data_width: 64,
    mm2s_burst_size: 16,
    has_s2mm: true,
    has_s2mm_dre: false,
    s2mm_data_width: 64,
    s2mm_burst_size: 16,
    has_sg: true,
    sg_length_width: 16,
    addr_width: 64,
};

pub struct AxiNetConfig {
    pub tx_bd_cnt: usize,
    pub rx_bd_cnt: usize,
    pub eth_baseaddr: usize,
    pub dma_baseaddr: usize,
    pub mac_addr: [u8; 6],
    pub mtu: usize
}

pub const AXI_NET_CONFIG: AxiNetConfig = AxiNetConfig {
    tx_bd_cnt: 1024,
    rx_bd_cnt: 1024,
    eth_baseaddr: 0x60140000,
    dma_baseaddr: 0x6010_0000,
    mac_addr: [0x00, 0x0A, 0x35, 0x01, 0x02, 0x03],
    mtu: XAE_MAX_JUMBO_FRAME_SIZE,
};
