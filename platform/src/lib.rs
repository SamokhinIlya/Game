#![allow(unused_unsafe)]
// for debug macros,
// when attributes on expressions gets stabilized this will be removed

extern crate core;
extern crate lazy_static;
extern crate winapi;

#[macro_use] pub mod debug;
pub mod graphics;
pub mod input;
pub mod time;

pub type RawPtr = *mut Mem;
pub enum Mem {}