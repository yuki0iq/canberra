#![feature(negative_impls)]

use anyhow::{ensure, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{
    ffi::CStr,
    fmt::{Display, Error as FmtError, Formatter},
    ptr,
};
use thiserror::Error;

pub mod ffi;

#[derive(Copy, Clone, FromPrimitive, Error, Debug)]
pub enum Error {
    NotSupported = -1,
    Invalid = -2,
    State = -3,
    Oom = -4,
    NoDriver = -5,
    System = -6,
    Corrupt = -7,
    TooBig = -8,
    NotFound = -9,
    Destroyed = -10,
    Canceled = -11,
    NotAvailable = -12,
    Access = -13,
    Io = -14,
    Internal = -15,
    Disabled = -16,
    Forked = -17,
    Disconnected = -18,
}

impl Error {
    fn c(val: i32) -> Option<Self> {
        FromPrimitive::from_i32(val)
    }
}

#[allow(non_snake_case)]
fn Y(val: i32) -> Result<()> {
    match Error::c(val) {
        None => Ok(()),
        Some(e) => Err(e)?,
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let ptr = unsafe { ffi::ca_strerror(*self as i32) };
        if ptr.is_null() {
            return Ok(());
        }
        let cstr = unsafe { CStr::from_ptr(ptr) };
        write!(f, "{:?}", cstr)
    }
}

pub struct Context {
    ctx: *mut ffi::ca_context,
}

// TODO use more concise thread unsafety bounds (ca_*** functions WILL fail if called from thread that differs from the first caller's one)
impl !Send for Context {}
impl !Sync for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        let err = Error::c(unsafe { ffi::ca_context_destroy(self.ctx) });
        assert!(err.is_none());
    }
}

impl Context {
    fn try_create() -> Result<Self> {
        let mut ptr = ptr::null_mut();
        Y(unsafe { ffi::ca_context_create(&mut ptr) })?;
        ensure!(!ptr.is_null(), "Context is a null pointer");
        Ok(Self { ctx: ptr })
    }

    fn open(&mut self) -> Result<()> {
        Y(unsafe { ffi::ca_context_open(self.ctx) })
    }

    fn set_driver(&mut self, driver: &CStr) -> Result<()> {
        Y(unsafe { ffi::ca_context_set_driver(self.ctx, driver.as_ptr()) })
    }

    fn change_device(&mut self, device: &CStr) -> Result<()> {
        Y(unsafe { ffi::ca_context_change_device(self.ctx, device.as_ptr()) })
    }

    // fn change_props(&mut self, props);
    // fn play(&mut self, id: u32, props, cb, userdata);
    // fn cache(&mut self, props);

    fn cancel(&mut self, id: u32) -> Result<()> {
        Y(unsafe { ffi::ca_context_cancel(self.ctx, id) })
    }

    fn is_playing(&mut self, id: u32) -> Result<bool> {
        let mut playing = 0;
        Y(unsafe { ffi::ca_context_playing(self.ctx, id, &mut playing) })?;
        Ok(playing != 0)
    }
}
