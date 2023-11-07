#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]

#[macro_use]
extern crate log;
extern crate alloc;
use core::{arch::asm, sync::atomic::{AtomicUsize, Ordering::Relaxed}};

use crate::config::{CLOCK_FREQ, CPU_NUM};
use config::{TOTAL_BOOT_STACK_SIZE, BOOT_STACK_SIZE};
use riscv::register::{time, utvec, ustatus, uip, sideleg, uie, sstatus, sepc, sip};


#[macro_use]
mod console;
mod config;
mod lang_items;
mod logger;
mod mm;
mod net;
mod plic;
mod trap;
mod drivers;

static BOOT_HART: AtomicUsize = AtomicUsize::new(1);
static mut START_TIME: usize = 0;
static mut END_TIME: usize = 0;
static mut DMA_TX_DURATION: usize = 0;

/// Initialize kernel stack in .bss section.
#[link_section = ".bss.stack"]
static mut STACK: [u8; TOTAL_BOOT_STACK_SIZE] = [0u8; TOTAL_BOOT_STACK_SIZE];

/// Entry for the first kernel.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn __entry(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "
        la a1, {stack}
        li t0, {total_stack_size}
        li t1, {stack_size}
        mul sp, a0, t1
        sub sp, t0, sp
        add sp, a1, sp
        ",        // Jump to the main function.
        "j  {main}",
        total_stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack_size       = const BOOT_STACK_SIZE,
        stack            =   sym STACK,
        main             =   sym rust_main_init,
        options(noreturn),
    )
}

/// Entry for other kernels.
#[naked]
#[no_mangle]
pub unsafe extern "C" fn __entry_others(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "
        la a1, {stack}
        li t0, {total_stack_size}
        li t1, {stack_size}
        mul sp, a0, t1
        sub sp, t0, sp
        add sp, a1, sp
        ",
        // Jump to the main function.
        "j  {main}",
        total_stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack_size       = const BOOT_STACK_SIZE,
        stack            =   sym STACK,
        main             =   sym rust_main_init_other,
        options(noreturn),
    )
}

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

fn clear_bss() {
    extern "C" {
        fn s_bss();
        fn e_bss();
        fn e_bss_ma();
    }
    (s_bss as usize..e_bss_ma as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    println!(
        "s_bss: {:#x?}, e_bss: {:#x?}, e_bss_ma: {:#x?}",
        s_bss as usize, e_bss as usize, e_bss_ma as usize
    );
}

#[no_mangle]
pub fn rust_main_init(hart_id: usize) {
    clear_bss();
    logger::init();
    mm::init_heap();
    plic::init();
    net::init();
    // drivers::init();
    if CPU_NUM > 1 {
        for i in 0..CPU_NUM {
            if i != hart_id {
                // Starts other harts.
                let ret = sbi_rt::hart_start(i, __entry_others as _, 0);
                assert!(ret.is_ok(), "Failed to shart hart {}", i);
            }
        }
        while BOOT_HART.load(Relaxed) != CPU_NUM {}
    }
    rust_main(hart_id)
}

#[no_mangle]
pub fn rust_main_init_other(hart_id: usize) {
    info!("Hart {} booted", hart_id);
    BOOT_HART.fetch_add(1, Relaxed);
    rust_main(hart_id)
}

#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {
    trap::init();
    plic::init_hart(hart_id);
    info!("Tests begin!");

    // net::udp_test();
    // net::tcp_test();
    #[cfg(feature = "sync")]
    drivers::eth_test();
    #[cfg(feature = "async")]
    drivers::async_eth_test();
    loop {}
}


pub fn delay(ms: usize) {
    let start = time::read();
    while time::read() - start < CLOCK_FREQ * ms / 1000 {}
}

pub fn read_time_reg() -> usize {
    time::read()
}