#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![cfg_attr(any(miri, NO_ALLOC_FAIL_TESTS, target_os = "macos"), allow(dead_code, unused_imports))]

#[cfg(feature = "alloc")]
use core::alloc::AllocError;

use pinned_init::*;
use std::sync::Arc;

#[expect(unused_attributes)]
#[path = "./ring_buf.rs"]
mod ring_buf;
use ring_buf::*;

#[cfg(all(
    feature = "alloc",
    not(miri),
    not(NO_ALLOC_FAIL_TESTS),
    not(target_os = "macos")
))]
#[test]
fn too_big_pinned() {
    // should be too big with current hardware.
    assert!(matches!(
        Box::pin_init(RingBuffer::<u8, { 1024 * 1024 * 1024 * 1024 }>::new()),
        Err(AllocError)
    ));
    // should be too big with current hardware.
    assert!(matches!(
        Arc::pin_init(RingBuffer::<u8, { 1024 * 1024 * 1024 * 1024 }>::new()),
        Err(AllocError)
    ));
}

#[cfg(all(
    feature = "alloc",
    not(miri),
    not(NO_ALLOC_FAIL_TESTS),
    not(target_os = "macos")
))]
#[test]
fn too_big_in_place() {
    // should be too big with current hardware.
    assert!(matches!(
        Box::init(zeroed::<[u8; 1024 * 1024 * 1024 * 1024]>()),
        Err(AllocError)
    ));
    // should be too big with current hardware.
    assert!(matches!(
        Arc::init(zeroed::<[u8; 1024 * 1024 * 1024 * 1024]>()),
        Err(AllocError)
    ));
}
