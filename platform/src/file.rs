use core::{
    convert::From,
    marker::Sized,
    mem,
    ops::Deref,
    ptr,
    result::Result,
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

pub struct File(Box<[u8]>);

impl Deref for File {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl File {
    pub fn size(&self) -> usize { self.0.len() }
    pub unsafe fn as_ptr(&self) -> *const u8 { self.0.as_ptr() }
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 { self.0.as_mut_ptr() }

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
                ptr::null_mut(),       // ignored when opening existing file
            )
        };
        if file_handle == INVALID_HANDLE_VALUE {
            return Err(FileErr::SomeError);
        }

        let file_size = unsafe {
            let mut size = mem::uninitialized();
            win_assert_non_zero!(
                GetFileSizeEx(file_handle, &mut size)
            );
            *size.QuadPart() as usize
        };
        let mut file_buffer = Vec::<u8>::with_capacity(file_size);
        let mut bytes_read = unsafe { mem::uninitialized() };
        win_assert_non_zero!(
            ReadFile(
                file_handle,
                file_buffer.as_mut_ptr() as *mut c_void,
                file_size as u32,
                &mut bytes_read,
                ptr::null_mut(),
            )
        );
        assert_eq!(file_size, bytes_read as usize);
        unsafe {
            file_buffer.set_len(bytes_read as usize);
        }

        win_assert_non_zero!(
            CloseHandle(file_handle)
        );

        Ok(Self(file_buffer.into_boxed_slice()))
    }

    pub fn write<T>(filepath: &str, value: &T) -> Result<(), FileErr>
        where T: Sized,
    {
        use std::io::Write;

        assert!(filepath.len() <= 256);
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
        win_assert_non_zero!(
            CloseHandle(file_handle)
        );

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
    //TODO: use std's Path
    fn load(filepath: &str) -> Result<Self, LoadErr>;
}

#[derive(Debug)]
pub enum LoadErr {
    NotValid,
    FileErr(FileErr),
}
