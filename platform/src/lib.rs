#![allow(unused_unsafe)]
// for debug macros,
// when attributes on expressions gets stabilized this will be removed

extern crate core;
extern crate winapi;

pub mod file;
pub mod graphics;
pub mod input;
pub mod memory;
pub mod time;
#[macro_use]
pub mod debug;


pub type RawPtr = *mut Mem;
pub enum Mem {}