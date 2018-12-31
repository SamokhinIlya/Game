use core::{
    mem,
    ptr,
    result::Result,
    convert::From,
};
use super::*;
use winapi::{
    ctypes::*,
    um::{
        fileapi::{
            CreateFileA, GetFileSizeEx, ReadFile, WriteFile,
            CREATE_ALWAYS, OPEN_EXISTING,
        },
        handleapi::{
            CloseHandle, INVALID_HANDLE_VALUE
        },
        winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE},
    },
};

pub struct File {
    pub data: *mut u8,
    pub size: usize,
}

impl core::ops::Drop for File {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                memory::deallocate(self.data);
            }
        }
    }
}

impl File {
    pub fn read(filepath: &str) -> Result<Self, FileErr> {
        use std::io::Write;
        assert!(filepath.len() <= 256);
        let mut str_buffer: [u8; 256] = unsafe { mem::uninitialized() };
        write!(&mut str_buffer as &mut [u8], "{}\0", filepath).unwrap();
        //TODO: check if W version is better
        let file_handle = unsafe {
            CreateFileA(
                str_buffer.as_ptr() as *const i8,
                GENERIC_READ,    //TODO: check others
                0,               //TODO: prevent other processess from opening file, check others
                ptr::null_mut(), //TODO: check others
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL, //TODO: check others
                ptr::null_mut(),       //NOTE: ignored when opening existing file
            )
        };
        if file_handle == INVALID_HANDLE_VALUE {
            return Err(FileErr::SomeError);
        }

        let bytes_to_read = unsafe {
            let mut size = mem::uninitialized();
            if GetFileSizeEx(file_handle, &mut size) == 0 {
                debug::panic_with_last_error_message("GetFileSizeEx");
            }
            *size.QuadPart() as usize
        };
        let file_buffer_ptr = unsafe { memory::allocate_bytes(bytes_to_read) };
        let mut bytes_read = unsafe { mem::uninitialized() };
        win_assert_non_zero!(
            ReadFile(
                file_handle,
                file_buffer_ptr as *mut c_void,
                bytes_to_read as u32,
                &mut bytes_read,
                ptr::null_mut(),
            )
        );
        assert!(bytes_to_read == bytes_read as usize);

        win_assert_non_zero!( CloseHandle(file_handle) );

        Ok(
            Self {
                data: file_buffer_ptr,
                size: bytes_to_read,
            }
        )
    }

    pub fn write<T>(filepath: &str, value: &T) -> Result<(), FileErr>
    where
        T: core::marker::Sized,
    {
        assert!(filepath.len() <= 256);
        use std::io::Write;

        let mut str_buffer: [u8; 256] = unsafe { mem::uninitialized() };
        write!(&mut str_buffer as &mut [u8], "{}\0", filepath).unwrap();
        //TODO: check if W version is better
        let file_handle = unsafe {
            CreateFileA(
                str_buffer.as_ptr() as *const i8,
                GENERIC_WRITE,         //TODO: check others
                0,               //TODO: prevent other processess from opening file, check others
                ptr::null_mut(), //TODO: check others
                CREATE_ALWAYS,   //TODO: if file exists - overwrites, check error code
                FILE_ATTRIBUTE_NORMAL, //TODO: check others
                ptr::null_mut(), //TODO: template with file attributes, check
            )
        };
        if file_handle == INVALID_HANDLE_VALUE {
            return Err(FileErr::SomeError);
        }

        let mut bytes_written = unsafe { mem::uninitialized() };
        win_assert_non_zero!(
            WriteFile(
                file_handle,
                value as *const T as *const c_void,
                mem::size_of::<T>() as u32,
                &mut bytes_written,
                ptr::null_mut(),
            )
        );
        win_assert_non_zero!( CloseHandle(file_handle) );

        Ok(())
    }
}

//TODO: expand to catch all errors
#[derive(Debug)]
pub enum FileErr {
    SomeError,
}

pub trait Load
    where Self: Sized + From<File>,
{
    fn load(filepath: &str) -> Result<Self, LoadErr>;
}

#[derive(Debug)]
pub enum LoadErr {
    NotValid,
    FileErr(FileErr),
}
