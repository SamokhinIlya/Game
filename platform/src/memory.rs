use core::{
    mem::size_of,
    marker::Sized,
};
use super::*;
use winapi::{
    ctypes::*,
    um::heapapi::*,
    um::winnt::*
};

pub unsafe fn allocate<T>(value: T) -> *mut T
    where T: Sized,
{
    let memory = allocate_bytes(size_of::<T>()) as *mut T;
    *memory = value;
    memory
}

pub unsafe fn allocate_bytes(nbytes: usize) -> *mut u8 {
    //TODO: it is also possible to call CreateHeap to create private heap
    //TODO: check what is faster: calling this every time or caching result and checking every time
    let heap = win_assert_non_null!( GetProcessHeap() );
    //TODO: it is also possible to deal with allocations in different threads - check msdn
    let memory = win_assert_non_null!(
        HeapAlloc(
            heap,
            HEAP_ZERO_MEMORY, //TODO: check other flags
            nbytes,
        )
    );

    memory as *mut u8
}

pub unsafe fn deallocate<T>(ptr: *mut T)
    where T: Sized,
{
    let heap = win_assert_non_null!( GetProcessHeap() );
    win_assert_non_zero!(
        HeapFree(
            heap,
            0, //NOTE: ignored for process heap
            ptr as *mut c_void,
        )
    );
}
