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
 * Reference: Enedis-NOI-CPT_54E https://www.enedis.fr/media/2035/download
 *
 */

use crate::prelude::*;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::anychar,
    character::complete::{char, i32, line_ending, not_line_ending},
    IResult,
};

macro_rules! _ignore_data {
    ($funcname:ident, $label:literal) => {
        fn $funcname(s: &str) -> IResult<&str, ()> {
            let (s, _) = label_to_ignore(s, $label)?;
            Ok((s, ()))
        }
    };
}

macro_rules! _numeric_data {
    ($funcname:ident, $label:literal) => {
        fn $funcname(s: &str) -> IResult<&str, TicValue> {
            let (s, value) = label_to_int(s, "INNST")?;
            Ok((s, TicValue::IINST(value)))
        }
    };
}

#[derive(Debug)]
pub enum TicUnit {
    Ampere,
    Volt,
    Watt,
    VoltAmpere,
    None,
}

#[derive(Debug)]
enum RegisterCut {
    CLOSE,
    OPEN,
    SURTENTION,
    DELESTING,
    ONCPL,
    HOTHIGH,
    HOTLOW,
}

#[derive(Debug)]
enum RegisterMod {
    PROVIDER,
    CONSUMER,
}

pub struct RegisterStatus {
    relay: bool,
    cut: RegisterCut,
    door_open: bool,
    surtension: bool,
    mode: RegisterMod,
    ener_avtiv: RegisterMod,
}

pub enum TicValue {
    IMAX(i32),
    IINST(i32),
    LTARF(String),
    IGNORE,
}

pub struct TicObject {
    label: &'static str,
    info: &'static str,
    unit: TicUnit,
}

impl TicObject {
    pub const IMAX: TicObject = TicObject {
        label: "IMAX",
        info: "Max called intensity (A)",
        unit: TicUnit::Ampere,
    };

    pub const IINST: TicObject = TicObject {
        label: "IINST",
        info: "Current intensity (A)",
        unit: TicUnit::Ampere,
    };

    pub const LTARF: TicObject = TicObject {
        label: "LTARF",
        info: "Current Tariff",
        unit: TicUnit::Ampere,
    };

    pub const IGNORED: TicObject = TicObject {
        label: "IGNORED",
        info: "Ignored Label",
        unit: TicUnit::None,
    };

    pub fn get_label(&self) -> &'static str {
        self.label
    }

    pub fn get_unit(&self) -> &TicUnit {
        &self.unit
    }

    pub fn get_info(&self) -> &'static str {
        self.info
    }
}

impl TicValue {
    pub fn metadata(&self) -> &TicObject {
        match self {
            TicValue::IMAX(_) => &TicObject::IMAX,
            TicValue::IINST(_) => &TicObject::IINST,
            TicValue::LTARF(_) => &TicObject::LTARF,

            _ => &TicObject::IGNORED,
        }
    }
}

fn separator(input: &str) -> IResult<&str, char> {
    char(0x09 as char)(input)
}

fn not_separator(chr: char) -> bool {
    chr != 0x09 as char
}

fn checksum(s: &str) -> IResult<&str, ()> {
    let (s, _) = separator(s)?;
    let (s, _) = anychar(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, ()))
}

fn label_to_int<'a>(s: &'a str, label: &str) -> IResult<&'a str, i32> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, value) = i32(s)?;
    let (s, _) = checksum(s)?;
    Ok((s, value))
}

fn label_to_str<'a>(s: &'a str, label: &str) -> IResult<&'a str, &'a str> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, value) = take_while(not_separator)(s)?;
    let (s, _) = checksum(s)?;
    Ok((s, value))
}

fn label_to_ignore<'a>(s: &'a str, label: &str) -> IResult<&'a str, ()> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = not_line_ending(s)?;
    let (s, value) = line_ending(s)?;
    Ok((s, ()))
}

// Max Intensity
fn tarf_data(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = label_to_str(s, "LTARF")?;
    Ok((s, TicValue::LTARF(value.to_string())))
}

// numeric message data
_numeric_data!(relay_msg, "RELAY");
_numeric_data!(innst_msg, "INNST");
_numeric_data!(imax_msg, "IMAX");
fn numeric_data(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = alt((
        relay_msg, innst_msg, imax_msg,
    ))(s)?;
    Ok((s, value))
}

// --- ignored messages ----------
_ignore_data!(ngtf_msg, "LTARF");
_ignore_data!(ltarf_msg, "NGTF");
_ignore_data!(vtic_msg, "VTIC");
_ignore_data!(date_msg, "DATE");
_ignore_data!(stge_msg, "STGE");
_ignore_data!(msg1_msg, "MSG1");
_ignore_data!(msg2_msg, "MSG2");
_ignore_data!(prm_msg, "PRM");
_ignore_data!(ntarf_msg, "NTARF");
_ignore_data!(njourf_msg, "NJOURF");
_ignore_data!(njour1_msg, "PJOURF+1");
_ignore_data!(pointe_msg, "PPOINTE");
fn ignore_data(s: &str) -> IResult<&str, TicValue> {
    let (s, _) = alt((
        ngtf_msg, ltarf_msg, vtic_msg, date_msg, stge_msg, stge_msg, msg1_msg, msg2_msg, prm_msg,
        ntarf_msg, njourf_msg, njour1_msg, pointe_msg,
    ))(s)?;
    Ok((s, TicValue::IGNORE))
}

fn tic_data(s: &str) -> IResult<&str, TicValue> {
    let (s, data) = alt((ignore_data, numeric_data, tarf_data))(s)?;
    Ok((s, data))
}

pub fn tic_from_str(tic_str: &str) -> Result<TicValue, LinkyError> {
    match tic_data(tic_str) {
        Ok((remaining, data)) => {
            if remaining.len() > 3 {
                return Err(LinkyError::ParsingError(remaining.to_string()));
            }
            Ok(data)
        }
        Err(error) => Err(LinkyError::ParsingError(error.to_string())),
    }
}
