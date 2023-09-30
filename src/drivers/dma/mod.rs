

pub mod axi_dma;

pub use self::axi_dma::*;


pub fn init() {
    axi_dma::init();
}