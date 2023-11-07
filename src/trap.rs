use crate::hart_id;
use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause, sepc, sie,
    sstatus::{self, Sstatus},
    stval, stvec, uip,
};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

global_asm!(include_str!("trap.asm"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
        // enable supervisor interrupt
        sstatus::set_sie();
        // enable external interrupt
        sie::set_sext();
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    // disable supervisor interrupt
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        scause::Trap::Interrupt(scause::Interrupt::SupervisorExternal) => {
            // debug!("SEI");
            crate::plic::handle_external_interrupt(hart_id(), 'S');
        }
        scause::Trap::Exception(scause::Exception::Breakpoint) => {
            log::debug!("ebreak");
            unsafe { uip::set_usoft(); }
        }
        _ => {
            error!(
                "Unsupported trap {:?}, stval = {:#x}, sepc = {:#x}!",
                scause.cause(),
                stval,
                sepc::read()
            );
            panic!("not surpport");
        }
    }
    cx
}
