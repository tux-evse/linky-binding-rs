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
use afbv4::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::anychar,
    character::complete::{char, i32, line_ending, not_line_ending},
    number::complete::hex_u32,
    IResult,
};
use serde::{Deserialize, Serialize};

macro_rules! _ignore_data {
    ($label:ident) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, ()> {
            let (s, _) = label_to_ignore(s, stringify!($label))?;
            Ok((s, ()))
        }
    };
}

macro_rules! _numeric_data {
    ($label:ident) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicValue> {
            let (s, value) = label_to_int(s, stringify!($label))?;
            Ok((s, TicValue::$label(value)))
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum TicUnit {
    Ampere,
    Volt,
    Watt,
    VoltAmpere,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(untagged)]
enum RegisterCut {
    CLOSE,
    SURTENTION,
    DELESTING,
    ONCPL,
    HOTHIGH,
    HOTLOW,
    OVERPOWER,
    UNKNOWN,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(untagged)]
enum RegisterMod {
    PROVIDER,
    CONSUMER,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum RegisterEnergy {
    POSITIVE,
    NEGATIVE,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct RegisterStatus {
    #[serde(skip_serializing)]
    pub raw: u32,
    relay_open: bool,
    cut: RegisterCut,
    door_open: bool,
    over_tension: bool,
    over_power: bool,
    mode: RegisterMod,
    energy: RegisterEnergy,
}

AfbDataConverter!(tic_value, TicValue);
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum TicValue {
    IMAX(i32),
    IMAX1(i32),
    IMAX2(i32),
    IMAX3(i32),

    // instant current
    IINST(i32),
    IINST1(i32),
    IINST2(i32),
    IINST3(i32),

    // instant active power
    SINSTS(i32),
    SINSTS1(i32),
    SINSTS2(i32),
    SINSTS3(i32),

    //misc
    ADSC(RegisterStatus),
    ADPS(i32),  // over consumption
    ADIR1(i32), // over consumption ph1
    ADIR2(i32), // over consumption ph2
    ADIR3(i32), // over consumption ph3
    UNSET,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TicObject {
    label: &'static str,
    info: &'static str,
    unit: TicUnit,
    count: usize,
}

impl TicObject {
    pub const ADSC: TicObject = TicObject {
        label: "ADSC",
        info: "Linky status register",
        unit: TicUnit::None,
        count: 1,
    };

    pub const ADPS: TicObject = TicObject {
        label: "ADPS",
        info: "Over consumption",
        unit: TicUnit::Ampere,
        count: 4,
    };

    pub const IMAX: TicObject = TicObject {
        label: "IMAX",
        info: "Max called intensity (A)",
        unit: TicUnit::Ampere,
        count: 4,
    };

    pub const IINST: TicObject = TicObject {
        label: "IINST",
        info: "Current intensity (A)",
        unit: TicUnit::Ampere,
        count: 4,
    };

    pub const SINSTS: TicObject = TicObject {
        label: "SINSTS",
        info: "Current Power (VA)",
        unit: TicUnit::VoltAmpere,
        count: 4,
    };

    pub const LTARF: TicObject = TicObject {
        label: "LTARF",
        info: "Current Tariff",
        unit: TicUnit::Ampere,
        count: 1,
    };

    pub const IGNORED: TicObject = TicObject {
        label: "IGNORED",
        info: "Ignored Label",
        unit: TicUnit::None,
        count: 0,
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

    pub fn get_count(&self) -> usize {
        self.count
    }
}

impl TicValue {
    pub fn metadata(&self) -> &TicObject {
        match self {
            TicValue::IMAX(_) => &TicObject::IMAX,
            TicValue::IMAX1(_) => &TicObject::IMAX,
            TicValue::IMAX2(_) => &TicObject::IMAX,
            TicValue::IMAX3(_) => &TicObject::IMAX,

            TicValue::IINST(_) => &TicObject::IINST,
            TicValue::SINSTS(_) => &TicObject::IINST,
            TicValue::SINSTS1(_) => &TicObject::IINST,
            TicValue::SINSTS2(_) => &TicObject::IINST,
            TicValue::SINSTS3(_) => &TicObject::IINST,

            TicValue::ADPS(_) => &TicObject::ADPS,
            TicValue::ADIR1(_) => &TicObject::ADPS,
            TicValue::ADIR2(_) => &TicObject::ADPS,
            TicValue::ADIR3(_) => &TicObject::ADPS,

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

// this method is not available from &str
fn hexa_to_value<'a>(s: &'a str) -> IResult<&'a str, u32> {
    match hex_u32::<nom::error::Error<&[u8]>>(s.as_bytes()) {
        Ok((_v, value)) => return Ok((s, value)),
        _ => {
            let err = nom::error::Error {
                input: s,
                code: nom::error::ErrorKind::Alpha,
            };
            return Err(nom::Err::Error(err));
        }
    }
}

fn label_to_register<'a>(s: &'a str, label: &str) -> IResult<&'a str, RegisterStatus> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, value) = hexa_to_value(s)?;
    let (s, _) = take_while(not_separator)(s)?;
    let (s, _) = checksum(s)?;

    let relay = value & 0x01 == 1;
    let cut = match value >> 1 & 0x111 {
        0 => RegisterCut::CLOSE,
        1 => RegisterCut::OVERPOWER,
        2 => RegisterCut::SURTENTION,
        3 => RegisterCut::DELESTING,
        4 => RegisterCut::ONCPL,
        5 => RegisterCut::HOTHIGH,
        6 => RegisterCut::HOTLOW,
        _ => RegisterCut::UNKNOWN,
    };

    let door = value >> 4 & 0x01 == 1;
    let tension = value >> 6 & 0x01 == 1;
    let power = value >> 7 & 0x01 == 1;

    let mode = match value >> 8 & 0x01 == 1 {
        true => RegisterMod::PROVIDER,
        false => RegisterMod::CONSUMER,
    };

    let active = match value >> 9 & 0x01 == 1 {
        true => RegisterEnergy::POSITIVE,
        false => RegisterEnergy::NEGATIVE,
    };

    let register = RegisterStatus {
        raw: value,
        relay_open: relay,
        cut: cut,
        door_open: door,
        over_tension: tension,
        over_power: power,
        mode: mode,
        energy: active,
    };
    Ok((s, register))
}

fn label_to_int<'a>(s: &'a str, label: &str) -> IResult<&'a str, i32> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, value) = i32(s)?;
    let (s, _) = checksum(s)?;
    Ok((s, value))
}

fn _label_to_str<'a>(s: &'a str, label: &str) -> IResult<&'a str, &'a str> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, value) = take_while(not_separator)(s)?;
    let (s, _) = checksum(s)?;
    Ok((s, value))
}

fn label_to_ignore<'a>(s: &'a str, label: &str) -> IResult<&'a str, ()> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = not_line_ending(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, ()))
}

// joour+1 is not a valid name
fn njour1_msg<'a>(s: &'a str) -> IResult<&'a str, ()> {
    let (s, _) = tag("PJOURF+1")(s)?;
    let (s, _) = not_line_ending(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, ()))
}

// register status
fn adsc(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = label_to_register(s, "ADSC")?;
    Ok((s, TicValue::ADSC(value)))
}

// numeric message data
_numeric_data!(ADPS);
_numeric_data!(IMAX);
_numeric_data!(IMAX1);
_numeric_data!(IMAX2);
_numeric_data!(IMAX3);
_numeric_data!(IINST);
_numeric_data!(IINST1);
_numeric_data!(IINST2);
_numeric_data!(IINST3);
_numeric_data!(SINSTS);
_numeric_data!(SINSTS1);
_numeric_data!(SINSTS2);
_numeric_data!(ADIR1);
_numeric_data!(ADIR2);
_numeric_data!(ADIR3);
_numeric_data!(SINSTS3);
fn numeric_data(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = alt((
        adsc, IMAX, IMAX1, IMAX2, IMAX3, IINST, IINST, IINST1, IINST2, IINST3, SINSTS, SINSTS1,
        SINSTS2, SINSTS3, ADPS, ADIR1, ADIR2, ADIR3,
    ))(s)?;
    Ok((s, value))
}

// --- ignored messages ---
_ignore_data!(LTARF);
_ignore_data!(NGTF);
_ignore_data!(VTIC);
_ignore_data!(DATE);
_ignore_data!(STGE);
_ignore_data!(MSG);
_ignore_data!(PRM);
_ignore_data!(NTARF);
_ignore_data!(NJOURF);
_ignore_data!(PPOINTE);
_ignore_data!(OPTARIF);
_ignore_data!(ISOUSC);
_ignore_data!(BASE);
_ignore_data!(HCHC);
_ignore_data!(HCHP);
_ignore_data!(EJPH);
_ignore_data!(BBRH);
_ignore_data!(PEJP);
_ignore_data!(PTEC);
_ignore_data!(DEMAIN);
_ignore_data!(HHPHC);
_ignore_data!(MOTDETAT);
_ignore_data!(PMAX);
_ignore_data!(PAPP);
_ignore_data!(PPOT);
_ignore_data!(EASF);
_ignore_data!(EAST);
_ignore_data!(CCAIN);
_ignore_data!(URMS);

fn ignore_data(s: &str) -> IResult<&str, TicValue> {
    let (s, _) = alt((
        LTARF, NGTF, VTIC, DATE, STGE, STGE, MSG, PRM, NTARF, NJOURF, njour1_msg, PPOINTE, OPTARIF,
        ISOUSC, BASE, HCHC, HCHP, EAST,URMS,CCAIN,
    ))(s)?;
    Ok((s, TicValue::UNSET))
}

fn ignore_data2(s: &str) -> IResult<&str, TicValue> {
    let (s, _) = alt((
        EJPH, BBRH, PEJP, PTEC, DEMAIN, HHPHC, MOTDETAT, PMAX, PAPP, PPOT, EASF,
    ))(s)?;
    Ok((s, TicValue::UNSET))
}

// Fulup note: size of nom 'alt' is limited which impose to split labels grammar
fn tic_data(s: &str) -> IResult<&str, TicValue> {
    let (s, data) = alt((ignore_data, ignore_data2, numeric_data))(s)?;
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

pub fn tic_register_type() -> Result<(), AfbError> {
    tic_value::register()?;
    Ok(())
}
