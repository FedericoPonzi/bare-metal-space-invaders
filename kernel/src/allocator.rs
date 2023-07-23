use core::alloc::{GlobalAlloc, Layout};

use crate::uart_pl011::{MutexTrait, NullLock};
use crate::{debug, info};
use core::ops::Range;

/// Gets the memory size
pub fn memory_size() -> usize {
    return 1 << 30; // 1 GB by default
}

/// Returns the heap memory
pub fn heap_memory() -> Range<*mut u8> {
    extern "C" {
        static mut __text_end: usize;
    }
    unsafe {
        let start = &__text_end as *const usize as *mut u8;
        let end = memory_size() as *const usize as *mut u8;

        Range { start, end }
    }
}

pub type AllocImpl = BumpAllocator;

#[global_allocator]
pub(crate) static ALLOCATOR: Allocator<AllocImpl> = Allocator::uninitialized();

pub struct Allocator<T> {
    //TODO: Remove inner member and just wrap the nulllock.
    allocator: NullLock<Option<T>>,
}

impl Allocator<AllocImpl> {
    const fn uninitialized() -> Allocator<AllocImpl> {
        Allocator {
            allocator: NullLock::new(None),
        }
    }
    pub fn initialize(&self) {
        let heap = heap_memory();
        let (start, end) = (heap.start, heap.end);
        let mut r = &self.allocator;
        r.lock(|alloc| *alloc = Some(BumpAllocator::new(start, end)));
    }
}

/// Allocate memory as described by the given `layout`.
///
/// Returns a pointer to newly-allocated memory,
/// or null to indicate allocation failure.
unsafe impl<T: ImplAllocator + core::fmt::Debug> GlobalAlloc for Allocator<T> {
    /// Align up!
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debug!("Requested allocation: {:?}", layout);
        let mut r = &self.allocator;
        r.lock(|inner| match inner {
            Some(allocator) => allocator.alloc(layout),
            None => core::ptr::null_mut(),
        })
    }

    /// Align down
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut r = &self.allocator;
        debug!("Requested dellocation: {:?}", layout);
        r.lock(|inner| {
            if let Some(allocator) = inner {
                allocator.dealloc(ptr, layout);
            }
        });
    }
}

#[alloc_error_handler]
pub fn oom(layout: Layout) -> ! {
    panic!(
        "Out Of Memory / alloc error handler: Allocation Failed. Requested layout: {:?}",
        layout
    );
}
pub unsafe trait ImplAllocator {
    fn new(start_memory_free: *mut u8, end_memory: *mut u8) -> Self;
    /// Allocate memory as described by the given `layout`.
    ///
    /// Returns a pointer to newly-allocated memory,
    /// or null to indicate allocation failure.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure that `layout` has non-zero size.
    ///
    /// (Extension subtraits might provide more specific bounds on
    /// behavior, e.g., guarantee a sentinel address or a null pointer
    /// in response to a zero-size allocation request.)
    ///
    /// The allocated block of memory may or may not be initialized.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    /// Deallocate the block of memory at the given `ptr` pointer with the given `layout`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all of the following:
    ///
    /// * `ptr` must denote a block of memory currently allocated via
    ///   this allocator,
    ///
    /// * `layout` must be the same layout that was used
    ///   to allocate that block of memory,
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}

#[derive(Debug)]
pub struct BumpAllocator {
    start_memory_free: *mut u8,
    end_memory: *mut u8,
    current: *mut u8,
    times: usize,
}
unsafe impl ImplAllocator for BumpAllocator {
    fn new(start_memory_free: *mut u8, end_memory: *mut u8) -> Self {
        Self {
            start_memory_free,
            end_memory,
            current: start_memory_free,
            times: 30,
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if self.times == 0 {
            panic!("Requested allocation: {:?}", layout);
        }
        self.times -= 1;
        let aligned = align_up(self.current as usize, layout.align());
        self.current = aligned
            .checked_add(layout.size())
            .expect("Alloc: memory allocation sum overflowed.") as *mut u8;
        let to_ret = aligned as *mut u8;
        info!(
            "Old heap position: {:?}, new position: {:?}",
            to_ret, self.current
        );
        to_ret
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator does nothing in dealloc.
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    if !align.is_power_of_two() {
        panic!("align_up: alignment must be a power of 2")
    }
    let delta = align - addr % align;
    // If addr is already aligned to addr, `delta` will be set to align.
    // In that case, to zero-fy delta % align is used:
    let actual_delta = delta % align;
    addr.checked_add(actual_delta)
        .expect("Checked add in align up overflowed.")
}
