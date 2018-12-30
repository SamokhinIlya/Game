extern crate core;
extern crate winapi;

pub mod file;
pub mod graphics;
pub mod input;
pub mod memory;
pub mod time;
pub mod debug;


pub type RawPtr = *mut Mem;
pub enum Mem {}