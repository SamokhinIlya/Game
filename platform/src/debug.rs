use core::{
    mem,
    ptr,
};
use winapi::{
    shared::ntdef::LPSTR,
    um::errhandlingapi::GetLastError
};

pub fn panic_with_last_error_message(fn_name: &str) {
    use std::{
        ffi::CStr,
        os::raw::c_char,
    };
    use winapi::um::winbase::{
        FormatMessageA,
        FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
        FORMAT_MESSAGE_IGNORE_INSERTS,
    };

    let mut message_ptr: *mut c_char = unsafe { mem::uninitialized() };
    let get_error_message_result = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null(),
            GetLastError(),
            0,
            &mut message_ptr as *mut _ as LPSTR,
            0,
            ptr::null_mut(),
        )
    };
    assert!(
        get_error_message_result != 0,
        "Error message retrieval failed. GetLastError -> {}",
        unsafe { GetLastError() }
    );

    //TODO: utf8_error handling
    let error_message = unsafe { CStr::from_ptr(message_ptr) }
        .to_str()
        .unwrap_or_else(|_utf8_error| {
            panic!("Error message from {} is not valid UTF-8", fn_name);
        });
    panic!("{}. Error: {}", fn_name, error_message);
}

#[macro_export]
macro_rules! win_assert_non_zero {
    (
        $fn_name:ident( $($arg:expr),* )
    ) => {
        unsafe {
            let result = $fn_name($($arg),*);
            if result == 0 {
                debug::panic_with_last_error_message(stringify!($fn_name));
            }

            result
        }
    };
    
    (
        $fn_name:ident( $($arg:expr,)* )
    ) => {
        win_assert_non_zero![$fn_name($($arg),*)]
    };
}

#[macro_export]
macro_rules! win_assert_non_null {
    (
        $fn_name:ident( $($arg:expr),* )
    ) => {
        unsafe {
            let result = $fn_name($($arg),*);
            if result.is_null() {
                debug::panic_with_last_error_message(stringify!($fn_name));
            }

            result
        }
    };
    
    (
        $fn_name:ident( $($arg:expr,)* )
    ) => {
        win_assert_non_null![$fn_name($($arg),*)]
    };
}