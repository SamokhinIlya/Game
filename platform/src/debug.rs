pub fn panic_with_last_error_message(fn_name: &str) {
    use std::{
        ffi::CStr,
        mem::MaybeUninit,
        os::raw::c_char,
        ptr,
    };
    use winapi::um::{
        errhandlingapi::GetLastError,
        winbase::{
            FormatMessageA,
            FORMAT_MESSAGE_ALLOCATE_BUFFER,
            FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS,
        }
    };

    let error_message = {
        let mut message_ptr = MaybeUninit::<*const c_char>::uninit();
        let get_error_message_result = unsafe {
            FormatMessageA(
                FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS,
                ptr::null(),
                GetLastError(),
                0,
                message_ptr.as_mut_ptr() as *mut _,
                0,
                ptr::null_mut(),
            )
        };
        assert!(
            get_error_message_result != 0,
            "Error message retrieval failed. GetLastError -> {}",
            unsafe { GetLastError() }
        );
        // TODO: show message box before panicking
        unsafe { CStr::from_ptr(message_ptr.assume_init()) }.to_str()
            .unwrap_or_else(|utf8_error| {
                panic!("Error message from {} is not valid UTF-8. Error: {}", fn_name, utf8_error);
            })
    };

    panic!("{}. Error: {}", fn_name, error_message);
}

#[macro_export]
macro_rules! win_assert_non_zero {
    (
        $fn_name:ident( $($arg:expr),* $(,)? ) $(;)?
    ) => {
        {
            let result = unsafe { $fn_name($($arg),*) };
            if result == 0 {
                $crate::debug::panic_with_last_error_message(stringify!($fn_name));
            }
            result
        }
    };

    (
        $( $fn_name:ident( $($arg:expr),* $(,)? ) );+ $(;)?
    ) => {
        {
            let mut result;
            $(
                unsafe {
                    result = $fn_name($($arg),*);
                }
                if result == 0 {
                    $crate::debug::panic_with_last_error_message(stringify!($fn_name));
                }
            )+
            result
        }
    };
}

#[macro_export]
macro_rules! win_assert_non_null {
    (
        $fn_name:ident( $($arg:expr),* $(,)? ) $(;)?
    ) => {
        {
            let result = unsafe { $fn_name($($arg),*) };
            if result.is_null() {
                $crate::debug::panic_with_last_error_message(stringify!($fn_name));
            }
            result
        }
    };

    (
        $( $fn_name:ident( $($arg:expr),* $(,)? ) );+ $(;)?
    ) => {
        {
            let mut result;
            $(
                unsafe {
                    result = $fn_name($($arg),*);
                }
                if result.is_null() {
                    $crate::debug::panic_with_last_error_message(stringify!($fn_name));
                }
            )+
            result
        }
    };
}