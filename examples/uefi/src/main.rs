#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(const_in_array_repeat_expressions)]
#![deny(warnings)]

extern crate alloc;
extern crate rlibc;

use alloc::vec::Vec;
use core::sync::atomic::*;
use log::*;
use uefi::prelude::*;

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> uefi::Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    info!("test booting multi-processors");
    unsafe {
        fn stack_fn(_pid: usize) -> usize {
            let layout = alloc::alloc::Layout::from_size_align(0x1000, 16).unwrap();
            unsafe { alloc::alloc::alloc(layout) as usize + 0x1000 }
        }
        x86_smpboot::start_application_processors(ap_main, stack_fn, |x| x);
    }
    ap_main();
    let pids: Vec<_> = STARTED
        .iter()
        .enumerate()
        .filter(|(_, started)| started.load(Ordering::SeqCst))
        .map(|(i, _)| i)
        .collect();
    info!("started {} processors: {:?}", pids.len(), pids);
    panic!("finished")
}

static STARTED: [AtomicBool; 64] = [AtomicBool::new(false); 64];

fn ap_main() {
    let apic_id = raw_cpuid::CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize;
    STARTED[apic_id].store(true, Ordering::SeqCst);
}
