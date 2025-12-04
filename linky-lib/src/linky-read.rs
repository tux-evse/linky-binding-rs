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
    ChecksumError(String),
}

pub struct SerialConfig {
    pub device: &'static str,
    pub parity: &'static str,
    pub speed: u32,
}

pub struct NetworkConfig {
    pub ip_bind: &'static str,
    pub udp_port: u16,
}

pub enum LinkyConfig {
    Serial(SerialConfig),
    Network(NetworkConfig),
}

pub struct LinkyHandle {
    handle: Box<dyn SourceHandle>,
}

impl LinkyHandle {
    pub fn new(source: &LinkyConfig) -> Result<LinkyHandle, AfbError> {
        let handle = match source {
            LinkyConfig::Serial(config) => {
                let par = match config.parity {
                    "even" => SerialCflag::PAREVN,
                    "odd" => SerialCflag::PARODD,
                    _ => return afb_error!("tty-parity-invalid", "Linky only support even|odd",),
                };

                let baud = match config.speed {
                    1200 => SerialSpeed::B1200,
                    9600 => SerialSpeed::B9600,
                    _ => return afb_error!("tty-speed-invalid", "Linky only support 1200|9600",),
                };

                let pflags = [PortFlag::NOCTTY, PortFlag::RDONLY];
                let iflags = [SerialIflag::IGNBRK];
                let cflags = [
                    SerialCflag::CS7,
                    SerialCflag::CLOCAL,
                    SerialCflag::PARENB,
                    par, /*dlt=even*/
                ];
                let lflags = [SerialLflag::ICANON];

                SerialHandle::new(config.device, baud, &pflags, &iflags, &cflags, &lflags)?
            }

            LinkyConfig::Network(config) => NetworkHandle::new(config.ip_bind, config.udp_port)?,
        };

        Ok(LinkyHandle { handle })
    }

    pub fn reopen(&self) -> Result<(), AfbError> {
        self.handle.close();
        self.handle.open()
    }

    pub fn get_fd(&self) -> i32 {
        self.handle.get_raw_fd()
    }

    pub fn get_uid(&self) -> &str {
        self.handle.get_uid()
    }

    pub fn checksum<'a>(&self, buffer: &'a [u8], count: usize) -> Result<&'a str, LinkyError> {
        const CHECH_SUM_OFFSET: usize = 2;

        // verify checksum take all data from 'etiquette" to last 'delimiter'
        let mut sum: u64 = 0;
        for idx in 0..(count - CHECH_SUM_OFFSET) {
            sum = sum + buffer[idx] as u64;
        }

        // reduce line to effective size
        let data = match buffer.get(0..count) {
            Some(value) => value,
            None => b"invalid-count",
        };

        // move byte buffer to printable string
        let line = match std::str::from_utf8(data) {
            Err(_) => return Err(LinkyError::ChecksumError("not uft".to_string())),
            Ok(data) => data,
        };

        // finally check
        let checksum = (sum & 0x3f) as u8 + 0x20;
        if checksum != buffer[count - CHECH_SUM_OFFSET] {
            Err(LinkyError::ChecksumError(line.to_string()))
        } else {
            Ok(line)
        }
    }

    pub fn decode(&self, buffer: &mut [u8]) -> Result<(TicMsg, bool), LinkyError> {
        let result = match self.handle.get_msgs(buffer) {
            Err(error) => {
                afb_log_msg!(Error, None, "Fail to read error={}", (error.to_string()));
                return Err(LinkyError::SerialError(error.to_string()));
            }
            Ok((count, eob)) => {
                if eob {
                    return Ok((TicMsg::NODATA, true));
                }
                if count > 0 && count <= 3 {
                    afb_log_msg!(Warning, None, "Invalid buffer={:?}", buffer);
                    return Err(LinkyError::RetryLater);
                } else {
                    let data = self.checksum(buffer, count)?;
                    (tic_from_str(data)?, eob)
                }
            }
        };
        Ok(result)
    }
}
