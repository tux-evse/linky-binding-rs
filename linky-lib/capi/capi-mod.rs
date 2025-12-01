/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 * Provide Interface abstraction for Linky acquisition sources
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
 *
 */

use ::std::os::raw;
use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::os::fd::AsRawFd;
use std::str::FromStr;

use afbv4::prelude::*;

pub mod cglue {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!("_capi-map.rs");
}

pub trait SourceHandle {
    fn get_uid(&self) -> &str;
    fn open(&self) -> Result<(), AfbError>;
    fn close(&self);
    fn get_raw_fd(&self) -> raw::c_int;
    fn get_msgs(&self, buffer: &mut [u8]) -> Result<(usize, bool), AfbError>;
}

pub struct SerialHandle {
    uid: &'static str,
    raw_fd: Cell<raw::c_int>,
    devname: CString,
    speed: SerialSpeed,
    pflags: raw::c_int,      // device open flags
    iflags: cglue::tcflag_t, // input stream mask
    cflags: cglue::tcflag_t, // control stream mask
    lflags: cglue::tcflag_t, // local control mask
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
    ISIG = cglue::TIO_ISIG,
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

const RING_BUFFER_SZ:usize = 512;
pub struct BufferRing {
    empty: bool,
    start: usize,
    stop: usize,
    data: [u8; RING_BUFFER_SZ],
}

impl BufferRing {
    pub fn new() -> RefCell<Self> {
        RefCell::new(Self {
            empty: true,
            start: 0,
            stop: 0,
            data: [0; RING_BUFFER_SZ],
        })
    }

    pub fn _print_buffer(&self) {
        println!("### ------------------------");
        println!("### Buffer: start:{} stop:{}", self.start, self.stop);
        println!("### Buffer: hexa {:02x?}", &self.data[0..self.stop]);
        match str::from_utf8(&self.data[0..self.stop]) {
            Ok(value) => println!("### Buffer text ->{}<-", value),
            Err(_) => println!("### Buffer hexa ->{:x?}", &self.data[0..self.stop]),
        }
        println!("### ------------------------");
    }

    pub fn _print_one_line(&self, buffer_out: &mut [u8], count: usize) {
        if count > 3 {
            match str::from_utf8(&buffer_out[0..count - 3]) {
                Ok(value) => println!("*** Line text:[..{}] ->{}<-", count - 2, value),
                Err(_) => println!(
                    "*** Line hexa:[..{}] ->{:02x?}",
                    count,
                    &buffer_out[0..count]
                ),
            }
        }
    }

    pub fn get_one_line(&mut self, buffer_out: &mut [u8]) -> (usize, bool) {
        // we have some remaining data in our buffer
        let mut idx = 0;
        for jdx in self.start..self.stop {
            if self.data[jdx] == 0x0D {
                continue;
            }; // ignore \r

            buffer_out[idx] = self.data[jdx];
            idx = idx + 1;

            // found end of line
            if self.data[jdx] == 0x0A {
                // ignore any line shorter than 3chars
                if idx <= 3 {
                    self.start += idx + 1;
                    continue;
                }
                //self._print_one_line(buffer_out, idx);
                self.start += idx + 1; // ignore \r
                return (idx, false);
            }
        }

        // no full line shift buffer
        if idx + 1 + self.start >= self.stop {
            idx = 0;
            for jdx in self.start..self.stop {
                self.data[idx] = self.data[jdx];
                idx = idx + 1;
            }
            self.start = idx;
            self.stop = idx;
            self.empty = true;
        }
        //self._print_one_line(buffer_out, idx);
        (idx, true)
    }
}

impl SerialHandle {
    // prepare handle for open operaTIFn
    #[track_caller]
    pub fn new(
        device: &'static str,
        speed: SerialSpeed,
        pflags: &[PortFlag],
        iflags: &[SerialIflag],
        cflags: &[SerialCflag],
        lflags: &[SerialLflag],
    ) -> Result<Box<dyn SourceHandle>, AfbError> {
        let devname = match CString::new(device) {
            Err(_) => return afb_error!("serial-invalid-devname", "fail to convert name to UTF8"),
            Ok(value) => value,
        };

        let mut tty_pflags = 0;
        for value in pflags {
            tty_pflags = tty_pflags | *value as i32;
        }
        let mut tty_iflags = 0;
        for value in iflags {
            tty_iflags = tty_iflags | *value as u32;
        }

        let mut tty_cflags = 0;
        for value in cflags {
            tty_cflags = tty_cflags | *value as u32;
        }

        let mut tty_lflags = 0;
        for lflag in lflags {
            tty_lflags = tty_lflags | *lflag as u32;
        }

        let handle = SerialHandle {
            uid: device,
            devname,
            raw_fd: Cell::new(0),
            speed,
            pflags: tty_pflags,
            iflags: tty_iflags,
            lflags: tty_lflags,
            cflags: tty_cflags,
        };

        // open the line before returning the handle
        let _ = &handle.open()?;

        Ok(Box::new(handle))
    }

    #[allow(dead_code)]
    pub fn flush(&self) {
        unsafe { cglue::tcflush(self.raw_fd.get(), cglue::TIO_TCIOFLUSH) };
    }
}

impl SourceHandle for SerialHandle {
    fn get_uid(&self) -> &str {
        self.uid
    }

    #[track_caller]
    fn open(&self) -> Result<(), AfbError> {
        // open tty device
        let raw_fd = unsafe { cglue::open(self.devname.as_ptr(), self.pflags, 0) };
        if raw_fd < 0 {
            return afb_error!(
                "serial-open-fail",
                "tty device={} err:{}",
                self.get_uid(),
                get_perror()
            );
        }

        // set attributes useless but ttyios.c_cc[6]= 1 require
        let mut termios: cglue::termios = unsafe { mem::zeroed() };
        termios.c_cc[cglue::TIO_VMIN as usize] = 1; // read at least one charracter when not in cannonical mode

        // Fulup warning cfsetspeed does not seems working as expected with ICANON
        if unsafe { cglue::cfsetispeed(&mut termios, self.speed as u32) } < 0 {
            return afb_error!(
                "serial-speed-setting",
                "tty device={} err:{}",
                self.get_uid(),
                get_perror()
            );
        }
        if unsafe { cglue::cfsetospeed(&mut termios, self.speed as u32) } < 0 {
            return afb_error!(
                "serial-speed-setting",
                "tty device={} err:{}",
                self.get_uid(),
                get_perror()
            );
        }

        termios.c_cflag = termios.c_cflag | self.cflags;
        termios.c_lflag = termios.c_lflag | self.lflags;
        termios.c_iflag = termios.c_iflag | self.iflags;

        if unsafe { cglue::tcsetattr(raw_fd, cglue::TIO_TCSANOW as i32, &mut termios) } < 0 {
            return afb_error!("serial-flags-setting", get_perror());
        }

        // update fd cell within immutable handle
        self.raw_fd.set(raw_fd);

        afb_log_msg!(
            Debug,
            None,
            "Open port={:?} speed={:?}",
            self.devname,
            self.speed
        );

        Ok(())
    }

    fn get_raw_fd(&self) -> raw::c_int {
        self.raw_fd.get()
    }

    #[track_caller]
    fn get_msgs(&self, out_buffer: &mut [u8]) -> Result<(usize, bool), AfbError> {
        let count = unsafe {
            cglue::read(
                self.raw_fd.get(),
                out_buffer as *const _ as *mut raw::c_void,
                out_buffer.len(),
            )
        };

        if count <= 0 {
            afb_error!(
                "SerialRaw-read-fail",
                "dev:{} err:{}",
                self.get_uid(),
                get_perror()
            )
        } else {
            // serial handler read tty buffer line/line
            Ok((count as usize, true))
        }
    }

    fn close(&self) {
        unsafe { cglue::close(self.raw_fd.get()) };
    }
}

pub struct NetworkHandle {
    uid: String,
    socket: UdpSocket,
    ring: RefCell<BufferRing>,
}

impl NetworkHandle {
    pub fn new(ip_bind: &'static str, udp_port: u16) -> Result<Box<dyn SourceHandle>, AfbError> {
        let ipaddr = match Ipv4Addr::from_str(ip_bind) {
            Ok(value) => value,
            Err(error) => {
                return afb_error!(
                    "network-bind-fail",
                    "invalid ipv4 addr:{} err:{}",
                    ip_bind,
                    error
                )
            }
        };

        let sockaddr = SocketAddrV4::new(ipaddr, udp_port);

        let socket = match UdpSocket::bind(sockaddr) {
            Ok(value) => value,
            Err(error) => {
                return afb_error!(
                    "network-bind-fail",
                    "invalid udp port:{} err:{}",
                    udp_port,
                    error
                )
            }
        };

        // UDP handle as an internal buffer to allow more than one line per udp read
        let handle = NetworkHandle {
            uid: format!("{}", sockaddr),
            socket,
            ring: BufferRing::new(),
        };

        Ok(Box::new(handle))
    }
}

impl SourceHandle for NetworkHandle {
    fn get_uid(&self) -> &str {
        &self.uid
    }

    fn open(&self) -> Result<(), AfbError> {
        Ok(())
    }
    fn close(&self) {}

    fn get_raw_fd(&self) -> raw::c_int {
        self.socket.as_raw_fd()
    }

    fn get_msgs(&self, out_buffer: &mut [u8]) -> Result<(usize, bool), AfbError> {
        let mut buffer_ring = match self.ring.try_borrow_mut() {
            Err(_) => {
                return afb_error!("network-getmsg-fail", "fail to access network ring buffer")
            }
            Ok(value) => value,
        };

        if !buffer_ring.empty {
            // we have remaining data pending in ring buffer
            let msg = buffer_ring.get_one_line(out_buffer);
            return Ok(msg);
        }

        let idx_start = buffer_ring.start;
        match self.socket.recv(&mut buffer_ring.data[idx_start..]) {
            Ok(count) => {
                buffer_ring.empty = false;
                buffer_ring.start = 0;
                buffer_ring.stop += count;
            }
            Err(error) => return afb_error!("network-getmsg-fail", error.to_string()),
        }

        // buffer_ring._print_buffer();
        let msg = buffer_ring.get_one_line(out_buffer);
        Ok(msg)
    }
}
