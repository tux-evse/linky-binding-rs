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

    // courrant efficace
    IRMS1(i32),
    IRMS2(i32),
    IRMS3(i32),

    // tension efficace
    URMS1(i32),
    URMS2(i32),
    URMS3(i32),

    // over current
    ADPS(i32),  // over consumption
    ADIR1(i32), // over consumption ph1
    ADIR2(i32), // over consumption ph2
    ADIR3(i32), // over consumption ph3

    // allowed power
    PREF(i32), // preference power
    PCOUP(i32), // cutting power

    //misc
    ADSC(RegisterStatus),
    RELAIS(i32),
    NTARF(i32), // index tarrification

    UNSET,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TicObject {
    uid: &'static str,
    name: &'static str,
    info: &'static str,
    unit: TicUnit,
    count: usize,
}

impl TicObject {
    pub const NTARF: TicObject = TicObject {
        uid: "NTARF",
        name: "Index-Tarif",
        info: "index de tarrification",
        unit: TicUnit::None,
        count: 1,
    };

    pub const RELAIS: TicObject = TicObject {
        uid: "RELAY",
        name: "Relay-Status",
        info: "Current relay position",
        unit: TicUnit::None,
        count: 1,
    };

    pub const PCOUP: TicObject = TicObject {
        uid: "PCOUP",
        name: "Power-Cutting",
        info: "Max current cutting power",
        unit: TicUnit::VoltAmpere,
        count: 1,
    };

    pub const IRMS: TicObject = TicObject {
        uid: "IRMS",
        name: "effective-current",
        info: "courrant efficace par phase",
        unit: TicUnit::Ampere,
        count: 3,
    };

    pub const URMS: TicObject = TicObject {
        uid: "URMS",
        name: "effective-tension",
        info: "tension efficace par phase",
        unit: TicUnit::Volt,
        count: 3,
    };

    pub const ADSC: TicObject = TicObject {
        uid: "ADSC",
        name: "Status-Register",
        info: "Linky status register",
        unit: TicUnit::None,
        count: 1,
    };

    pub const ADPS: TicObject = TicObject {
        uid: "ADPS",
        name: "Over-Power",
        info: "Over consumption",
        unit: TicUnit::Ampere,
        count: 4,
    };

    pub const IINST: TicObject = TicObject {
        uid: "IINST",
        name: "Instant-Current",
        info: "Current intensity (A)",
        unit: TicUnit::Ampere,
        count: 4,
    };

    pub const SINSTS: TicObject = TicObject {
        uid: "SINSTS",
        name: "Instant-Power",
        info: "Current Power (VA)",
        unit: TicUnit::VoltAmpere,
        count: 4,
    };

    pub const IGNORED: TicObject = TicObject {
        uid: "IGNORED",
        name: "Ignored",
        info: "Ignored uid",
        unit: TicUnit::None,
        count: 0,
    };

    pub fn get_uid(&self) -> &'static str {
        self.uid
    }

    pub fn get_name(&self) -> &'static str {
        self.name
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

            TicValue::IINST(_) => &TicObject::IINST,
            TicValue::SINSTS(_) => &TicObject::IINST,
            TicValue::SINSTS1(_) => &TicObject::IINST,
            TicValue::SINSTS2(_) => &TicObject::IINST,
            TicValue::SINSTS3(_) => &TicObject::IINST,

            TicValue::ADPS(_) => &TicObject::ADPS,
            TicValue::ADIR1(_) => &TicObject::ADPS,
            TicValue::ADIR2(_) => &TicObject::ADPS,
            TicValue::ADIR3(_) => &TicObject::ADPS,

            TicValue::PCOUP(_) => &TicObject::PCOUP,
            TicValue::PREF(_) => &TicObject::PCOUP,

            TicValue::NTARF(_) => &TicObject::NTARF,

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

// register status
fn adsc(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = label_to_register(s, "ADSC")?;
    Ok((s, TicValue::ADSC(value)))
}

// i32 message data
_numeric_data!(ADPS);
_numeric_data!(ADIR1);
_numeric_data!(ADIR2);
_numeric_data!(ADIR3);
_numeric_data!(IINST);
_numeric_data!(IINST1);
_numeric_data!(IINST2);
_numeric_data!(IINST3);
_numeric_data!(NTARF);
_numeric_data!(PREF);
_numeric_data!(PCOUP);
_numeric_data!(RELAIS);
_numeric_data!(SINSTS);
_numeric_data!(SINSTS1);
_numeric_data!(SINSTS2);
_numeric_data!(SINSTS3);
_numeric_data!(URMS1);
_numeric_data!(URMS2);
_numeric_data!(URMS3);
_numeric_data!(IRMS1);
_numeric_data!(IRMS2);
_numeric_data!(IRMS3);

fn numeric_data_a(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = char('A')(s)?;
    let (s, value) = alt((adsc, ADPS, ADIR1, ADIR2, ADIR3))(s)?;
    Ok((s, value))
}

fn numeric_data_i(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = char('I')(s)?;
    let (s, value) = alt((
       IINST, IINST, IINST1, IINST2, IINST3, IRMS1, IRMS2, IRMS3,
    ))(s)?;
    Ok((s, value))
}

fn numeric_data_p(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = char('P')(s)?;
    let (s, value) = alt((PCOUP, PREF))(s)?;
    Ok((s, value))
}

fn numeric_data_s(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = char('S')(s)?;
    let (s, value) = alt((SINSTS, SINSTS1, SINSTS2, SINSTS3))(s)?;
    Ok((s, value))
}

fn numeric_data_x(s: &str) -> IResult<&str, TicValue> {
    let (s, value) = alt((RELAIS, NTARF, URMS1, URMS2, URMS3))(s)?;
    Ok((s, value))
}

// --- ignored messages ---
_ignore_data!(BASE);
_ignore_data!(BBRH);
_ignore_data!(CCAIN);
_ignore_data!(DATE);
_ignore_data!(DEMAIN);
_ignore_data!(DPM);
_ignore_data!(EAS);
_ignore_data!(EAIT);
_ignore_data!(EJPH);
_ignore_data!(FPM);
_ignore_data!(HC);
_ignore_data!(HHPHC);
_ignore_data!(IRMS);
_ignore_data!(IMAX);
_ignore_data!(ISOUSC);
_ignore_data!(LTARF);
_ignore_data!(MOTDETAT);
_ignore_data!(MSG);
_ignore_data!(NGTF);
_ignore_data!(NJOURF);
_ignore_data!(OPTARIF);
_ignore_data!(PAPP);
_ignore_data!(PEJP);
_ignore_data!(PMAX);
_ignore_data!(PJOURF);
_ignore_data!(PPOINTE);
_ignore_data!(PPOT);
_ignore_data!(PRM);
_ignore_data!(PTEC);
_ignore_data!(STGE);
_ignore_data!(SMAX);
_ignore_data!(UMOY);
_ignore_data!(VTIC);

fn ignore_data_b_c_d(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = alt((char('B'), char('C'), char('D')))(s)?;
    let (s, _) = alt((BASE, BBRH, CCAIN, DATE, DEMAIN, DPM))(s)?;
    Ok((s, TicValue::UNSET))
}

fn ignore_data_e_f_h_i(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = alt((char('E'), char('H'), char('I'), char('F')))(s)?;
    let (s, _) = alt((EAS, EAIT, FPM, EJPH, HC, HHPHC, IRMS, IMAX, ISOUSC))(s)?;
    Ok((s, TicValue::UNSET))
}

fn ignore_data_l_m_n(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = alt((char('L'), char('M'), char('N')))(s)?;
    let (s, _) = alt((LTARF, MOTDETAT, MSG, NGTF, NJOURF))(s)?;
    Ok((s, TicValue::UNSET))
}

fn ignore_data_o_p_s(s: &str) -> IResult<&str, TicValue> {
    let (_, _) = alt((char('O'), char('P'), char('S')))(s)?;
    let (s, _) = alt((OPTARIF, PAPP, PEJP, PMAX, PPOINTE, PJOURF, PPOT, PRM, PTEC, STGE, SMAX))(s)?;
    Ok((s, TicValue::UNSET))
}

fn ignore_data_x(s: &str) -> IResult<&str, TicValue> {
    let (s, _) = alt((UMOY, VTIC))(s)?;
    Ok((s, TicValue::UNSET))
}


// Fulup note: size of nom 'alt' is limited which impose to split labels grammar
fn tic_data(s: &str) -> IResult<&str, TicValue> {
    let (s, data) = alt((
        numeric_data_a,
        numeric_data_i,
        numeric_data_p,
        numeric_data_s,
        numeric_data_x,
        ignore_data_b_c_d,
        ignore_data_e_f_h_i,
        ignore_data_l_m_n,
        ignore_data_o_p_s,
        ignore_data_x,
    ))(s)?;
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
