//! `uefi-alloc` implements Rust's global allocator interface using UEFI's memory allocation functions.
//!
//! Linking this crate in your app will allow you to use Rust's higher-level data structures,
//! like boxes, vectors, hash maps, linked lists and so on.
//!
//! # Usage
//!
//! Call the `init` function with a reference to the boot services table.
//! Failure to do so before calling a memory allocating function will panic.

// Enable additional lints.
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]

#![no_std]

// Custom allocators are currently unstable.
#![feature(allocator_api)]
#![feature(global_allocator)]

use core::alloc::{GlobalAlloc, Layout, Opaque};

extern crate uefi;
use uefi::table::boot::{BootServices, MemoryType};

/// Reference to the boot services table, used to call the pool memory allocation functions.
static mut BOOT_SERVICES: Option<&BootServices> = None;

/// Initializes the allocator.
pub fn init(boot_services: &'static BootServices) {
    unsafe {
        BOOT_SERVICES = Some(boot_services);
    }
}

fn boot_services() -> &'static BootServices {
    unsafe {
        BOOT_SERVICES.unwrap()
    }
}

/// Allocator which uses the UEFI pool allocation functions.
///
/// Only valid for as long as the UEFI runtime services are available.
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        let mem_ty = MemoryType::LoaderData;
        let size = layout.size();
        let align = layout.align();

        // TODO: add support for other alignments.
        if align > 8 {
            // Unsupported alignment for allocation, UEFI can only allocate 8-byte aligned addresses
            // BUG: use `Opaque::null_mut()` once it works. See https://github.com/rust-lang/rust/issues/46665
            0 as *mut _
        } else {
            boot_services()
                .allocate_pool(mem_ty, size)
                .map(|addr| addr as *mut _)
                // BUG: use `Opaque::null_mut()` once it works. See https://github.com/rust-lang/rust/issues/46665
                .unwrap_or(0 as *mut _)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, _layout: Layout) {
        let addr = ptr as usize;
        boot_services()
            .free_pool(addr)
            .unwrap();
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;