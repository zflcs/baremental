pub mod net;

pub use net::*;


pub fn init() {
    net::init();
}

#[cfg(feature = "sync")]
pub fn eth_test() {
    use crate::config::AXI_NET_CONFIG;
    use alloc::boxed::Box;
    use axi_ethernet::LinkStatus;
    use crate::read_time_reg;
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

    let mut eth = AXI_ETH.lock();
    eth.reset();
    let options = eth.get_options();
    use axi_ethernet::XAE_JUMBO_OPTION;
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
    eth.enable_tx_cmplt();
    eth.clear_tx_cmplt();

    drop(eth);
    // submit buffer
    let rx_frame = Box::pin([1u8; 20000]);
    let start = read_time_reg();
    let _buf = AXI_DMA.tx_submit(rx_frame).unwrap().wait();
    use crate::{START_TIME, DMA_TX_DURATION};
    unsafe { START_TIME = read_time_reg() };

    let end = read_time_reg();
    use crate::config::{CLOCK_FREQ, USEC_PER_SEC};
    let duration = (end - start) * USEC_PER_SEC / CLOCK_FREQ;
    unsafe { DMA_TX_DURATION = duration; }
    AXI_ETH.lock().start();
}

#[cfg(feature = "async")]
pub fn async_eth_test() {
    use crate::read_time_reg;
    net::init();

    let task = Arc::new(Task {
        state: AtomicU32::new(0),
        poll_fn: Some(poll),
        fut: AtomicCell::new(Box::new(send_data())),
    });
    let start = read_time_reg();
    loop {
        if task.state.load(core::sync::atomic::Ordering::Relaxed) != 1 {
            log::debug!("run task");
            let poll_fn = task.poll_fn.as_ref().unwrap();
            unsafe { (*poll_fn)(task.clone()) }
        }
    }
}

#[cfg(feature = "async")]
async fn send_data() -> i32 {
    let rx_frame = Box::pin([1u8; 20000]);
    log::debug!("send data 1");
    let _buf = AXI_DMA.tx_submit(rx_frame).unwrap().await;
    0
}

use core::{sync::atomic::AtomicU32,  task::RawWaker};
use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::future::Future;
use core::pin::Pin;
use crossbeam::atomic::AtomicCell;
#[cfg(feature = "async")]
#[repr(C)]
pub struct Task {
    pub(crate) state: AtomicU32,
    pub(crate) poll_fn: Option<unsafe fn(Arc<Task>)>,
    pub fut: AtomicCell<Box<dyn Future<Output = i32> + 'static + Send + Sync>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.state.store(2, core::sync::atomic::Ordering::Relaxed);
        log::debug!("wake_by_ref {}", self.state.load(core::sync::atomic::Ordering::Relaxed));
    }
}

unsafe fn poll(task: Arc<Task>) {
    use core::task::{Waker, Context, Poll};
    let fut = &mut *task.fut.as_ptr();
    let mut future = Pin::new_unchecked(fut.as_mut());
    let waker = Waker::from_raw(RawWaker::from(task.clone()));
    let mut cx = Context::from_waker(&waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(_) => { log::debug!("Ready ok"); },
        Poll::Pending => { 
            if task.state.load(core::sync::atomic::Ordering::Relaxed) == 0 {
                task.state.store(1, core::sync::atomic::Ordering::Relaxed); 
            }
            log::debug!("pending {}", task.state.load(core::sync::atomic::Ordering::Relaxed));
        }
    };
}