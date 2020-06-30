#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![deny(warnings)]

extern crate alloc;
extern crate rlibc;

use log::*;
use uefi::prelude::*;

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> uefi::Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    unsafe {
        fn stack_fn(_pid: usize) -> usize {
            let layout = alloc::alloc::Layout::from_size_align(0x1000, 16).unwrap();
            unsafe { alloc::alloc::alloc(layout) as usize + 0x1000 }
        }
        x86_smpboot::start_application_processors(ap_main, stack_fn, |x| x);
    }
    panic!("finished")
}

fn ap_main() {
    let apic_id = raw_cpuid::CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize;
    info!("processor {} started", apic_id);
}
