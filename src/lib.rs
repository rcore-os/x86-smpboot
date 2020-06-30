#![no_std]
#![feature(global_asm)]
#![deny(warnings)]

use x86::apic::{xapic::XAPIC, ApicControl, ApicId};

global_asm!(include_str!("boot_ap.S"));

/// Startup all application processors.
///
/// # Arguments
///
/// - `entry`: The entry function of application processors.
/// - `stack_fn`: A function given the processor ID and outputs its stack top.
/// - `phys_to_virt`: A function that convert physical address to virtual address.
///
/// # Safety
///
/// This function will write to physical page at 0x6000, and control local APIC
/// through memory region at page of 0xfee0_0000.
pub unsafe fn start_application_processors(
    entry: fn(),
    stack_fn: impl Fn(usize) -> usize,
    phys_to_virt: impl Fn(usize) -> usize,
) {
    (phys_to_virt(0x6ff8) as *mut u32).write(x86::controlregs::cr3() as u32);
    (phys_to_virt(0x6ff0) as *mut usize).write(entry as usize);

    extern "C" {
        fn ap_start();
        fn ap_end();
    }
    // copy boot_ap code to 0x6000
    const START_PAGE: u8 = 6;
    let count = ap_end as usize - ap_start as usize;
    core::ptr::copy_nonoverlapping(
        ap_start as *const u8,
        phys_to_virt(START_PAGE as usize * 0x1000) as _,
        count,
    );
    // startup
    let apic_region = core::slice::from_raw_parts_mut(phys_to_virt(0xfee0_0000) as _, 0x1000 / 4);
    let mut lapic = XAPIC::new(apic_region);
    for apic_id in 1..64 {
        // set stack
        (phys_to_virt(0x6fe8) as *mut usize).write(stack_fn(apic_id as usize));

        // send IPIs
        let apic_id = ApicId::XApic(apic_id);
        lapic.ipi_init(apic_id);
        delay_us(200);
        lapic.ipi_init_deassert();
        delay_us(10000);
        lapic.ipi_startup(apic_id, START_PAGE);
        delay_us(200);
        lapic.ipi_startup(apic_id, START_PAGE);
        delay_us(200);

        // wait for startup
        delay_us(10000);
    }
}

/// Spinning delay for specified amount of time on microseconds.
fn delay_us(us: u64) {
    use core::arch::x86_64::_rdtsc;
    let start = unsafe { _rdtsc() };
    let freq = 3_000_000_000u64; // assume 3GHz
    let end = start + freq / 1_000_000 * us;
    while unsafe { _rdtsc() } < end {
        core::sync::atomic::spin_loop_hint();
    }
}
