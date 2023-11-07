use rv_plic::Priority;
use rv_plic::PLIC;


pub const PLIC_BASE: usize = 0xc00_0000;

pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

pub fn get_context(hartid: usize, mode: char) -> usize {
    const MODE_PER_HART: usize = 3;
    hartid * MODE_PER_HART
        + match mode {
            'M' => 0,
            'S' => 1,
            'U' => 2,
            _ => panic!("Wrong Mode"),
        }
}



pub fn init() {
    for intr in 1..=6 {
        Plic::set_priority(intr, Priority::lowest());
    }
}


pub fn init_hart(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    Plic::clear_enable(context, 0);
    Plic::set_threshold(context, Priority::any());
    #[cfg(feature = "sync")]
    for irq in 1..=3 {
        Plic::enable(context, irq);
        Plic::claim(context);
        Plic::complete(context, irq);
    }
    #[cfg(feature = "async")]
    for irq in 1..=5 {
        Plic::enable(context, irq);
        Plic::claim(context);
        Plic::complete(context, irq);
    }
    Plic::clear_enable(get_context(hart_id, 'U'), 0);
    Plic::set_threshold(get_context(hart_id, 'M'), Priority::never());
}    

pub fn handle_external_interrupt(hart_id: usize, mode: char) {
    let context = get_context(hart_id, mode);
    if let Some(irq) = Plic::claim(context) {
        log::trace!("[PLIC] ctx: {}, IRQ: {:?}", context, irq);
        match irq {
            2 | 3 | 4 | 5 => crate::net::net_interrupt_handler(irq),
            _ => panic!("not surpport"),
        };
        Plic::complete(context, irq);
    }
}
