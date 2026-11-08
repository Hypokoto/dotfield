use std::ffi::c_int;
use std::thread;
use std::time::Duration;

// glibc's mallopt settings: the size served by its own mapping, and how many pools threads share
const M_MMAP_THRESHOLD: c_int = -3;
const M_ARENA_MAX: c_int = -8;

const ONE_MEGABYTE: c_int = 1024 * 1024;

const POOLS: c_int = 2;

// how often freed memory inside the pools is handed back
const TRIM: Duration = Duration::from_secs(10);

unsafe extern "C" {
    fn mallopt(param: c_int, value: c_int) -> c_int;

    fn malloc_trim(pad: usize) -> c_int;
}

/*
 * glibc raises the mapping size after the first big block is freed, so
 * later decode buffers stay in a pool after they are freed; fixing it
 * keeps big blocks on their own mappings, which go back to the system as
 * soon as an image is done with them. each new thread also gets a pool of
 * its own, so every image decoded on its own thread left freed memory
 * behind in yet another pool; two pools let that memory be used again
 */
pub fn limit() {
    let mapped = unsafe { mallopt(M_MMAP_THRESHOLD, ONE_MEGABYTE) };
    let pooled = unsafe { mallopt(M_ARENA_MAX, POOLS) };

    if mapped != 1 || pooled != 1 {
        eprintln!("amane: could not set the allocator's limits, using defaults");
    }

    thread::spawn(trim);
}

/*
 * a pool only shrinks from its end, so the gaps decoding, d-bus and niri's
 * events leave in the middle stay with the process; trimming gives them back
 */
pub fn trim() {
    loop {
        thread::sleep(TRIM);

        unsafe { malloc_trim(0) };
    }
}

/// Trims freed allocator pages back to the operating system once without looping.
pub fn trim_once() {
    unsafe { malloc_trim(0) };
}
