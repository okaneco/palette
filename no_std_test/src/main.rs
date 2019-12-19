#![feature(lang_items, start, alloc_error_handler)]
#![no_std]

extern crate libc;
extern crate alloc;

extern crate palette;

use core::panic::PanicInfo;

//from https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { null_mut() }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static A: MyAllocator = MyAllocator;

//from https://github.com/rust-lang/rust/issues/51540
#[alloc_error_handler]
fn handle_alloc_error(_layout: Layout) -> ! {
    // example implementation based on libc
    extern "C" { fn abort() -> !; }
    unsafe { abort() }
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    let _magenta = palette::Srgb::new(255u8, 0, 255);
    let mut v = alloc::vec::Vec::with_capacity(2);
    v.push(palette::LinSrgb::new(1.0, 0.1, 0.1));
    v.push(palette::LinSrgb::new(0.1, 1.0, 1.0));
    let _grad = palette::Gradient::new(v);

    0
}

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern fn rust_eh_unwind_resume() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
