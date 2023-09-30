pub mod net;
pub mod dma;

pub use net::*;
pub use dma::*;


pub fn init() {
    dma::init();
    net::init();
}