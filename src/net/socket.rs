
use alloc::{sync::Arc, vec};
use kernel_sync::SpinLock;
use spin::Lazy;

use smoltcp::iface::SocketSet;


pub static SOCKET_SET: Lazy<Arc<SpinLock<SocketSet>>> = Lazy::new(|| Arc::new(SpinLock::new(
    SocketSet::new(vec![])
)));





