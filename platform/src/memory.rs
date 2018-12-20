use core::{
    mem::size_of,
    marker::Sized,
};
use winapi::ctypes::*;
use crate::debug;

use winapi::{um::heapapi::*, um::winnt::*};

#[inline]
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
    let heap = GetProcessHeap();
    if heap.is_null() {
        debug::panic_with_last_error_message("GetProcessHeap");
    }
    //TODO: it is also possible to deal with allocations in different threads - check msdn
    let memory = HeapAlloc(
        heap,
        HEAP_ZERO_MEMORY, //TODO: check other flags
        nbytes,
    );
    if memory.is_null() {
        debug::panic_with_last_error_message("HeapAlloc");
    }

    memory as *mut u8
}

pub unsafe fn deallocate<T>(ptr: *mut T)
    where T: Sized,
{
    let heap = GetProcessHeap();
    if heap.is_null() {
        debug::panic_with_last_error_message("GetProcessHeap");
    }

    let free_result = HeapFree(
        heap,
        0, //NOTE: ignored for process heap
        ptr as *mut c_void,
    );
    if free_result == 0 {
        debug::panic_with_last_error_message("HeapFree");
    }
}
