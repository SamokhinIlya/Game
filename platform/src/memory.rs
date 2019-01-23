use core::{
    mem::{size_of, align_of},
    marker::Sized,
};
use super::*;
use winapi::{
    um::heapapi::*,
    um::winnt::*
};

pub unsafe fn allocate_array<T>(size: usize) -> *mut T {
    //TODO: it is also possible to call CreateHeap to create private heap
    //TODO: check what is faster: calling this every time or caching result and checking every time
    let heap = win_assert_non_null!(
        GetProcessHeap()
    );
    //TODO: it is also possible to deal with allocations in different threads - check msdn
    let ptr = win_assert_non_null!(
        HeapAlloc(
            heap,
            HEAP_ZERO_MEMORY, //TODO: check other flags
            size * size_of::<T>() + align_of::<T>(),
        )
    );

    let raw = ptr as usize;
    let align_mask = (1 << log2(align_of::<T>())) - 1;
    if raw & align_mask == 0 {
        ptr as *mut T
    } else {
        // find first next aligned adress by adding alignment to our ptr
        // and then masking away least significant part, that is not divisible by alignment
        // NOTE: on on my machine (amd64 win10) all HeapAllocs are 16 byte aligned
        // so this thing may or may not work
        ((raw + align_of::<T>()) & !align_mask) as *mut T
    }
}

pub unsafe fn deallocate<T>(ptr: *mut T)
    where T: Sized,
{
    let heap = win_assert_non_null!(
        GetProcessHeap()
    );
    win_assert_non_zero!(
        HeapFree(
            heap,
            0, // ignored for process heap
            ptr as *mut _,
        )
    );
}

// assumes that alignment cannot be more that 8
#[inline(always)]
unsafe fn log2(n: usize) -> usize {
    match n {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        _ => std::unreachable!(),
    }
}