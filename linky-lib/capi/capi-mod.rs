/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIFNS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitaTIFns under the License.
 */

use ::std::os::raw;
use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;

use afbv4::prelude::*;

const MAX_ERROR_LEN: usize = 256;
pub mod cglue {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!("_capi-map.rs");
}

pub fn get_perror() -> String {
    let mut buffer = [0 as raw::c_char; MAX_ERROR_LEN];
    unsafe {
        cglue::strerror_r(
            *cglue::__errno_location(),
            &mut buffer as *mut raw::c_char,
            MAX_ERROR_LEN,
        )
    };
    let cstring = unsafe { CStr::from_ptr(&mut buffer as *const raw::c_char) };
    let slice: &str = cstring.to_str().unwrap();
    slice.to_owned()
}

pub struct SerialRaw {
    pub(crate)raw_fd: Cell<raw::c_int>,
    pub(crate)devname: CString,
    pub(crate)speed: SerialSpeed,
    pub(crate)pflags: raw::c_int,  // device open flags
    pub(crate)iflags: cglue::tcflag_t, // input stream mask
    pub(crate)cflags: cglue::tcflag_t, // control stream mask
    pub(crate)lflags: cglue::tcflag_t, // local control mask
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum SerialSpeed {
    B1200 = cglue::TIO_B1200,
    B9600 = cglue::TIO_B9600,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum SerialCflag {
    CS7 = cglue::TCF_CS7,
    CS8 = cglue::TCF_CS8,
    PARENB = cglue::TCF_PARENB,
    PARODD = cglue::TCF_PARODD,
    CSTOPB = cglue::TCF_CSTOPB,
    CRTSCTS = cglue::TCF_CRTSCTS,
    CLOCAL = cglue::TCF_CLOCAL,
    PAREVN = 0, // C default value
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum SerialIflag {
    IGNBRK = cglue::TIF_IGNBRK,
    IGNPAR = cglue::TIF_IGNPAR,
    INLCR = cglue::TIF_INLCR,
    IGNCR = cglue::TIF_IGNCR,
    IUCLC = cglue::TIF_IUCLC,
    IUTF8 = cglue::TIF_IUTF8,
    ICRNL = cglue::TIF_ICRNL,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum SerialLflag {
    ICANON = cglue::TIO_ICANON,
    XCASE = cglue::TIO_XCASE,
    ISIG= cglue::TIO_ISIG,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PortFlag {
    NOCTTY = cglue::TTY_O_NOCTTY,
    NDELAY = cglue::TTY_O_NDELAY,
    RDWRITE = cglue::TTY_O_RDWR,
    RDONLY = cglue::TTY_O_RDONLY,
    OSYNC = cglue::TTY_O_SYNC,
}

impl SerialRaw {

    // prepare handle for open operaTIFn
    pub fn new(
        device: &'static str,
        speed: SerialSpeed,
        pflags: &[PortFlag],
        iflags: &[SerialIflag],
        cflags: &[SerialCflag],
        lflags: &[SerialLflag],
    ) -> Result<SerialRaw, AfbError> {
        let devname = match CString::new(device) {
            Err(_) => {
                return Err(AfbError::new("serial-invalid-devname", "fail to convert name to UTF8"))
            }
            Ok(value) => value,
        };

        let mut tty_pflags = 0;
        for value in pflags {
            tty_pflags = tty_pflags | *value as i32;
        }
        let mut tty_iflags= 0;
        for value in iflags {
            tty_iflags = tty_iflags | *value as u32;
        }

        let mut tty_cflags= 0;
        for value in cflags {
            tty_cflags = tty_cflags | *value as u32;
        }

        let mut tty_lflags= 0;
        for lflag in lflags {
            tty_lflags = tty_lflags | *lflag as u32;
        }

        let handle= SerialRaw {
            devname,
            raw_fd:Cell::new(0),
            speed,
            pflags: tty_pflags,
            iflags: tty_iflags,
            lflags: tty_lflags,
            cflags: tty_cflags,
        };

        // open the line before returning the handle
        let _ = &handle.open() ?;

        Ok(handle)
    }

    pub fn open(&self) -> Result<(), AfbError> {
        // open tty device
        let raw_fd = unsafe { cglue::open(self.devname.as_ptr(), self.pflags, 0) };
        if raw_fd < 0 {
            return Err(AfbError::new("serial-open-fail", get_perror()));
        }

        // set attributes useless but ttyios.c_cc[6]= 1 require
        let mut termios: cglue::termios = unsafe { mem::zeroed() };
        termios.c_cc[cglue::TIO_VMIN as usize]=1; // read at least one charracter when not in cannonical mode

        // Fulup warning cfsetspeed does not seems working as expected with ICANON
        if unsafe { cglue::cfsetispeed(&mut termios, self.speed as u32) } < 0 {
            return Err(AfbError::new("serial-speed-setting", get_perror()));
        }
        if unsafe { cglue::cfsetospeed(&mut termios, self.speed as u32) } < 0 {
            return Err(AfbError::new("serial-speed-setting", get_perror()));
        }

        termios.c_cflag= termios.c_cflag| self.cflags;
        termios.c_lflag= termios.c_lflag| self.lflags;
        termios.c_iflag= termios.c_iflag| self.iflags;

        if unsafe { cglue::tcsetattr(raw_fd, cglue::TIO_TCSANOW as i32, &mut termios) } < 0 {
            return Err(AfbError::new("serial-flags-setting", get_perror()));
        }

        // update fd cell within immutable handle
        self.raw_fd.set(raw_fd);

        afb_log_msg!(Debug, None, "Open port={:?} speed={:?}", self.devname, self.speed);

        Ok(())
    }

    pub fn get_raw_fd(&self) -> raw::c_int {
        self.raw_fd.get()
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, AfbError> {
        let count = unsafe {
            cglue::read(
                self.raw_fd.get(),
                buffer as *const _ as *mut raw::c_void,
                buffer.len(),
            )
        };

        if count <= 0 {
            Err(AfbError::new("SerialRaw-read-fail", get_perror()))
        } else {
            Ok(count as usize)
        }
    }

    #[allow(dead_code)]
    pub fn flush(&self) {
        unsafe{cglue::tcflush(self.raw_fd.get(), cglue::TIO_TCIOFLUSH)};
    }

    pub fn close(&self) {
        unsafe{cglue::close(self.raw_fd.get())};
    }
}
