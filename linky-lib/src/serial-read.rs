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
 */

use crate::prelude::*;
use afbv4::prelude::*;
use std::str;

#[derive(Debug)]
pub enum LinkyError {
    RetryLater,
    ReopenDev,
    FatalError,
    TooLong(String),
    ParsingError(String),
    InvalidEncoding,
    SerialError(String),
    ChecksumError,
}

pub struct LinkyHandle {
    portname: &'static str,
    handle: SerialRaw,
}

impl LinkyHandle {
    pub fn new(
        portname: &'static str,
        speed: u32,
        parity: &'static str,
    ) -> Result<LinkyHandle, AfbError> {
        let parity = match parity {
            "even" => SerialCflag::PAREVN,
            "odd" => SerialCflag::PARODD,
            _ => {
                return Err(AfbError::new(
                    "tty-parity-invalid",
                    "Linky only support even|odd",
                ))
            }
        };

        let speed = match speed {
            1200 => SerialSpeed::B1200,
            9600 => SerialSpeed::B9600,
            _ => {
                return Err(AfbError::new(
                    "tty-speed-invalid",
                    "Linky only support 1200|9600",
                ))
            }
        };

        let pflags = [PortFlag::NOCTTY, PortFlag::RDONLY];
        let iflags= [SerialIflag::IGNBRK];
        let cflags = [SerialCflag::CS7, SerialCflag::CLOCAL, SerialCflag::PARENB, parity /*dlt=even*/];
        let lflags = [SerialLflag::ICANON];

        let handle = SerialRaw::new(portname, speed, &pflags, &iflags, &cflags, &lflags)?;

        Ok(LinkyHandle { portname, handle })
    }

    pub fn reopen(&self) -> Result<(), AfbError> {
        self.handle.close();
        self.handle.open()
    }

    pub fn get_fd(&self) -> i32 {
        self.handle.get_raw_fd()
    }

    pub fn get_name(&self) -> &'static str {
        self.portname
    }

    fn checksum<'a> (&self, buffer:&'a [u8], count: usize) -> Result<&'a str, LinkyError> {
        // verify checksum take all data from 'etiquette" to last 'delimiteur'
        let mut sum:u64=0;
        for idx in 0 .. count -3 {
            sum= sum + buffer[idx] as u64;
        }

        let checksum= (sum & 0x3f) as u8 + 0x20;
        if checksum != buffer [count-3] {
           return Err(LinkyError::ChecksumError)
        }

        match str::from_utf8(buffer) {
            Err(_) =>  Err(LinkyError::ChecksumError),
            Ok(data) => Ok(data),
        }
    }

    pub fn decode(&self, buffer: &mut [u8]) -> Result<TicValue, LinkyError> {

        let count= match self.handle.read(buffer) {
            Err(error) => return Err(LinkyError::SerialError(error.to_string())),
            Ok(count) => {count}
        };

        let data= self.checksum(buffer, count)?;

        println! ("data={}", data);

        let value= tic_from_str(data)?;

        Ok(value)
    }
}
