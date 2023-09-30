/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x4_0000;

/// Total boot kernel size.
pub const TOTAL_BOOT_STACK_SIZE: usize = BOOT_STACK_SIZE * CPU_NUM;


pub const KERNEL_HEAP_SIZE: usize = 0x20_000;


pub const CLOCK_FREQ: usize = 10_000_000;

pub const CPU_NUM: usize = 1;

