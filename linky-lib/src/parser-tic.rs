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
 * Reference: Enedis-MOP-CPT_002E https://www.enedis.fr/media/2035/download
 *
 */

use crate::prelude::*;
use afbv4::prelude::*;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, char, i32, multispace0},
    combinator::opt,
    number::complete::hex_u32,
    IResult,
};
use serde::{Deserialize, Serialize};

macro_rules! _ignore_data {
    ($label:ident) => {};
}

macro_rules! _numeric_data {
    ($label:ident) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = label_to_int(s, stringify!($label))?;
            Ok((s, TicMsg::$label(value)))
        }
    };
    ($label:ident, $name:expr) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = label_to_int(s, $name)?;
            Ok((s, TicMsg::$label(value)))
        }
    };
}

macro_rules! _text_data {
    ($label:ident) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = _label_to_str(s, stringify!($label))?;
            Ok((s, TicMsg::$label(value.to_string())))
        }
    };
    ($label:ident, $name:expr) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = label_to_int(s, $name)?;
            Ok((s, TicMsg::$label(value)))
        }
    };
}

macro_rules! _stamped_numeric {
    ($label:ident) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = stamp_profile(s, stringify!($label))?;
            Ok((s, TicMsg::$label(value)))
        }
    };
    ($label:ident, $name:expr) => {
        #[allow(non_snake_case)]
        fn $label(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = stamp_profile(s, $name)?;
            Ok((s, TicMsg::$label(value)))
        }
    };
}

macro_rules! _provider_profile {
    ($tic:ident, $label:expr) => {
        #[allow(non_snake_case)]
        fn $tic(s: &str) -> IResult<&str, TicMsg> {
            let (s, value) = provider_profile(s, $label)?;
            Ok((s, TicMsg::$tic(value)))
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TicUnit {
    Ampere,
    Volt,
    Watt,
    VoltAmpere,
    Whour,
    Time,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum RegisterMod {
    PROVIDER,
    CONSUMER,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum RegisterEnergy {
    POSITIVE,
    NEGATIVE,
}

AfbDataConverter!(register_status, RegisterStatus);
#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl RegisterStatus {
    pub fn new() -> Self {
        Self {
            raw: 0,
            relay_open: false,
            cut: RegisterCut::UNKNOWN,
            door_open: false,
            over_tension: false,
            over_power: false,
            mode: RegisterMod::CONSUMER,
            energy: RegisterEnergy::NEGATIVE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProviderInfo {
    hour: u8,
    minute: u8,
    selector: u16,
}

impl ProviderInfo {
    pub fn to_jsonc(&self) -> Result<JsoncObj, AfbError> {
        let jslot = JsoncObj::new();
        jslot.add("hour", self.hour)?;
        jslot.add("minute", self.hour)?;
        jslot.add("selector", &format!("{:#08x}", self.selector))?;
        Ok(jslot)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeStampData {
    time: [u8; 13],
    data: Option<i32>,
}

impl TimeStampData {
    pub fn new(time: &str, data: Option<i32>) -> Result<Self, AfbError> {
        if time.len() != 13 {
            return afb_error!("time-stamp-invalid", "invalid date len:{}", time);
        }
        let mut stamp = Self {
            time: [0; 13],
            data,
        };

        let time = time.as_bytes();
        for idx in 0..time.len() {
            stamp.time[idx] = time[idx];
        }
        Ok(stamp)
    }

    fn token_to_num(&self, token: &[u8]) -> Result<u8, AfbError> {
        let token = match std::str::from_utf8(token) {
            Ok(value) => value,
            Err(_err) => return afb_error!("time-stamp-invalid", "invalid utf8 token:{:?}", token),
        };

        let value = match FromStr::from_str(token) {
            Ok(value) => value,
            Err(_err) => {
                return afb_error!("time-stamp-invalid", "invalid numeric token:{}", token)
            }
        };
        Ok(value)
    }

    pub fn is_summer_time(&self) -> Result<bool, AfbError> {
        let value = match self.time[0] {
            b'H' => false,
            b'E' => true,
            _ => {
                return afb_error!(
                    "time-stamp-invalid",
                    "invalid summer/winter data:{}",
                    self.time[0]
                )
            }
        };
        Ok(value)
    }

    pub fn get_year(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[1..3])
    }

    pub fn get_month(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[4..5])
    }

    pub fn get_day(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[6..7])
    }

    pub fn get_hour(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[8..9])
    }

    pub fn get_minute(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[10..11])
    }

    pub fn get_seconde(&self) -> Result<u8, AfbError> {
        self.token_to_num(&self.time[12..13])
    }

    pub fn to_jsonc(&self) -> Result<JsoncObj, AfbError> {
        let time = format!(
            "20{:02}-{:02}-{:02}T{:02}:{:02}-{:02}:00",
            self.get_year()?,
            self.get_month()?,
            self.get_day()?,
            self.get_hour()?,
            self.get_minute()?,
            self.get_month()?
        );
        let jsonc = JsoncObj::new();
        jsonc.add("stamp", &time)?;
        match self.data {
            Some(value) => {
                jsonc.add("data", value)?;
            }
            None => {}
        }
        Ok(jsonc)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProviderProfile {
    uid: String,
    count: usize,
    data: [Option<ProviderInfo>; 11],
}

impl ProviderProfile {
    pub fn new(uid: &str) -> Self {
        ProviderProfile {
            uid: uid.to_string(),
            count: 0,
            data: [
                None, None, None, None, None, None, None, None, None, None, None,
            ],
        }
    }

    pub fn to_jsonc(&self) -> Result<JsoncObj, AfbError> {
        let jsonc = JsoncObj::new();
        jsonc.add("uid", &self.uid)?;
        let jarray = JsoncObj::array();
        for idx in 0..self.count {
            match &self.data[idx] {
                Some(info) => {
                    jarray.append(info.to_jsonc()?)?;
                }
                None => return afb_error!("provider_profile_jsonc", "no data at index:{}", idx),
            }
        }
        jsonc.add("times", jarray)?;
        Ok(jsonc)
    }
}

AfbDataConverter!(tic_value, TicMsg);
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum TicMsg {
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
    PREF(i32),  // preference power
    PCOUP(i32), // cutting power
    EAST(i32),  // total consumed power
    EAIT(i32),  // total produced power

    //misc
    STGE(RegisterStatus),
    ADSC(String),
    RELAIS(i32),
    NTARF(i32),    // current tariff index
    NJOURF(i32),   // current provider calendar day
    NJOURF_T(i32), // next provider calendar day

    // next day/pic profile
    PJOURF_T(ProviderProfile),
    PPOINTE(ProviderProfile),

    MSG1(String),
    MSG2(String),
    NGTF(String),
    LTARF(String),

    // stamped data
    DATE(TimeStampData),
    SMAXSN(TimeStampData),
    SMAXSN_Y(TimeStampData),
    SMAXIN(TimeStampData),
    SMAXIN_Y(TimeStampData),

    // tenstion moyenne
    UMOY1(TimeStampData),
    UMOY2(TimeStampData),
    UMOY3(TimeStampData),

    NODATA,
    IGNORED,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicObject {
    uid: &'static str,
    name: &'static str,
    info: &'static str,
    unit: TicUnit,
}

impl TicObject {
    pub const STGE: TicObject = TicObject {
        uid: "STGE",
        name: "meeter-register",
        info: "linky meeter status register",
        unit: TicUnit::None,
    };
    pub const UMOY: TicObject = TicObject {
        uid: "UMOY",
        name: "average-voltage",
        info: "linky meeter status register",
        unit: TicUnit::Volt,
    };
    pub const ENERGY: TicObject = TicObject {
        uid: "energy",
        name: "energy-counter",
        info: "Total consumes/injected counters average",
        unit: TicUnit::Whour,
    };
    pub const TARIFF: TicObject = TicObject {
        uid: "tarrif",
        name: "tariff-name",
        info: "current calendar/tariff name",
        unit: TicUnit::None,
    };

    pub const MSG: TicObject = TicObject {
        uid: "msg",
        name: "provider-message",
        info: "Provider short messages",
        unit: TicUnit::None,
    };

    pub const POWERIN: TicObject = TicObject {
        uid: "POWERIN",
        name: "power-in",
        info: "Max consumed power (Today/Yesterday)",
        unit: TicUnit::VoltAmpere,
    };

    pub const POWEROUT: TicObject = TicObject {
        uid: "POWEROUT",
        name: "power-out",
        info: "Max Injected power (today/yesterday)",
        unit: TicUnit::VoltAmpere,
    };

    pub const DATE: TicObject = TicObject {
        uid: "today",
        name: "Current-Date",
        info: "Date/time provider",
        unit: TicUnit::Time,
    };

    pub const NTARF: TicObject = TicObject {
        uid: "NTARF",
        name: "tariff-index",
        info: "provider tariff index",
        unit: TicUnit::None,
    };

    pub const NJOURF: TicObject = TicObject {
        uid: "NJOURF",
        name: "current-day",
        info: "Current day number within provider calendar",
        unit: TicUnit::None,
    };

    pub const PROFILE: TicObject = TicObject {
        uid: "profiles",
        name: "Next day/pic profile",
        info: "day & pic next profile within provider calendar",
        unit: TicUnit::None,
    };

    pub const PCOUP: TicObject = TicObject {
        uid: "PCOUP",
        name: "power-cut",
        info: "Max current cutting power",
        unit: TicUnit::VoltAmpere,
    };

    pub const RELAIS: TicObject = TicObject {
        uid: "RELAY",
        name: "relay-status",
        info: "Current relay position",
        unit: TicUnit::None,
    };

    pub const IRMS: TicObject = TicObject {
        uid: "IRMS",
        name: "instant-current",
        info: "courrant efficace par phase",
        unit: TicUnit::Ampere,
    };

    pub const URMS: TicObject = TicObject {
        uid: "URMS",
        name: "instant-voltage",
        info: "tension efficace par phase",
        unit: TicUnit::Volt,
    };

    pub const ADSC: TicObject = TicObject {
        uid: "ADSC",
        name: "meeter-address",
        info: "Linky meeter address code",
        unit: TicUnit::None,
    };

    pub const ADPS: TicObject = TicObject {
        uid: "ADPS",
        name: "over-power",
        info: "Over consumption signal",
        unit: TicUnit::Ampere,
    };

    pub const IINST: TicObject = TicObject {
        uid: "IINST",
        name: "instant-current",
        info: "Current intensity (A)",
        unit: TicUnit::Ampere,
    };

    pub const SINSTS: TicObject = TicObject {
        uid: "SINSTS",
        name: "instant-power",
        info: "Current Power (VA)",
        unit: TicUnit::VoltAmpere,
    };

    pub const IGNORED: TicObject = TicObject {
        uid: "IGNORED",
        name: "Ignored",
        info: "Ignored uid",
        unit: TicUnit::None,
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
}

impl TicMsg {
    pub fn metadata(&self) -> &TicObject {
        match self {
            TicMsg::SINSTS(_) => &TicObject::IINST,
            TicMsg::SINSTS1(_) => &TicObject::IINST,
            TicMsg::SINSTS2(_) => &TicObject::IINST,
            TicMsg::SINSTS3(_) => &TicObject::IINST,

            TicMsg::ADPS(_) => &TicObject::ADPS,
            TicMsg::ADIR1(_) => &TicObject::ADPS,
            TicMsg::ADIR2(_) => &TicObject::ADPS,
            TicMsg::ADIR3(_) => &TicObject::ADPS,

            TicMsg::PCOUP(_) => &TicObject::PCOUP,
            TicMsg::PREF(_) => &TicObject::PCOUP,

            TicMsg::NTARF(_) => &TicObject::NTARF,
            TicMsg::NJOURF(_) => &TicObject::NJOURF,
            TicMsg::NJOURF_T(_) => &TicObject::NJOURF,
            TicMsg::MSG1(_) => &TicObject::MSG,
            TicMsg::MSG2(_) => &TicObject::MSG,

            TicMsg::DATE(_) => &TicObject::DATE,
            TicMsg::SMAXSN(_) => &TicObject::DATE,
            TicMsg::SMAXSN_Y(_) => &TicObject::DATE,
            TicMsg::SMAXIN(_) => &TicObject::DATE,
            TicMsg::SMAXIN_Y(_) => &TicObject::DATE,

            // linky 60/90A only
            TicMsg::IINST(_) => &TicObject::IINST,

            _ => &TicObject::IGNORED,
        }
    }
}

fn separator(input: &str) -> IResult<&str, char> {
    char(0x09 as char)(input)
}

fn not_eol(chr: char) -> bool {
    chr != 0x0A as char
}

fn not_separator(chr: char) -> bool {
    chr != 0x09 as char
}

fn checksum(s: &str) -> IResult<&str, ()> {
    let (s, _) = separator(s)?;
    let (s, _) = take_while(not_eol)(s)?;
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

fn msg_to_ignore(s: &str) -> IResult<&str, TicMsg> {
    let (s, _msg) = take_while(not_eol)(s)?;
    // println!("***ignored: {}", _msg);
    Ok((s, TicMsg::IGNORED))
}

// register status
fn status_register(s: &str) -> IResult<&str, TicMsg> {
    let (s, register) = label_to_register(s, "STGE")?;
    Ok((s, TicMsg::STGE(register)))
}

fn get_one_profile_info<'a>(s: &'a str) -> IResult<&'a str, Option<ProviderInfo>> {
    let (s, token) = alphanumeric1(s)?;

    if token == "NONUTILE" {
        return Ok((s, None));
    }

    let hour: u8 = match FromStr::from_str(&token[0..1]) {
        Ok(value) => value,
        Err(_err) => {
            let err = nom::error::Error {
                input: s,
                code: nom::error::ErrorKind::Alpha,
            };
            return Err(nom::Err::Error(err));
        }
    };

    let minute: u8 = match FromStr::from_str(&token[2..3]) {
        Ok(value) => value,
        Err(_err) => {
            let err = nom::error::Error {
                input: s,
                code: nom::error::ErrorKind::Alpha,
            };
            return Err(nom::Err::Error(err));
        }
    };

    let selector = match hexa_to_value(&token[4..]) {
        Ok((_, value)) => value as u16,
        Err(err) => return Err(err),
    };

    let provider = ProviderInfo {
        hour,
        minute,
        selector,
    };
    let (s, _) = multispace0(s)?;
    Ok((s, Some(provider)))
}

fn provider_profile<'a>(s: &'a str, label: &str) -> IResult<&'a str, ProviderProfile> {
    let (s, _) = tag(label)(s)?;
    let (mut s, _) = separator(s)?;

    let mut provider = ProviderProfile::new(label);

    for idx in 0..11 {
        match get_one_profile_info(s) {
            Ok((remainder, info)) => match info {
                Some(profile) => {
                    provider.data[idx] = Some(profile);
                    s = remainder;
                }
                None => {
                    provider.count = idx;
                    break;
                }
            },
            Err(err) => return Err(err),
        };
    }
    let (s, _) = take_while(not_eol)(s)?;
    Ok((s, provider))
}

fn stamp_profile<'a>(s: &'a str, label: &str) -> IResult<&'a str, TimeStampData> {
    let (s, _) = tag(label)(s)?;
    let (s, _) = separator(s)?;
    let (s, time) = alphanumeric1(s)?;
    let (s, _) = separator(s)?;
    let (s, data) = opt(i32)(s)?;

    let (s, _) = checksum(s)?;
    let stamp = match TimeStampData::new(time, data) {
        Ok(value) => value,
        Err(_err) => {
            let err = nom::error::Error {
                input: s,
                code: nom::error::ErrorKind::Alpha,
            };
            return Err(nom::Err::Error(err));
        }
    };
    Ok((s, stamp))
}

fn extract_data_a(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('A')(s)?;
    let (s, value) = alt((ADSC, ADPS, ADIR1, ADIR2, ADIR3))(s)?;
    Ok((s, value))
}

fn extract_data_d(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('D')(s)?;
    let (s, value) = (DATE)(s)?;
    Ok((s, value))
}

fn extract_data_e(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('E')(s)?;
    let (s, value) = alt((EAST, EAIT))(s)?;
    Ok((s, value))
}

fn extract_data_i(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('I')(s)?;
    let (s, value) = alt((IINST, IINST, IINST1, IINST2, IINST3, IRMS1, IRMS2, IRMS3))(s)?;
    Ok((s, value))
}

fn extract_data_l(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('L')(s)?;
    let (s, value) = (LTARF)(s)?;
    Ok((s, value))
}

fn extract_data_m(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('M')(s)?;
    let (s, value) = alt((MSG1, MSG2))(s)?;
    Ok((s, value))
}

fn extract_data_n(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('N')(s)?;
    let (s, value) = alt((NJOURF, NJOURF_T, NGTF))(s)?;
    Ok((s, value))
}

fn extract_data_p(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('P')(s)?;
    let (s, value) = alt((PCOUP, PREF, PJOURF_T))(s)?;
    Ok((s, value))
}

fn extract_data_s(s: &str) -> IResult<&str, TicMsg> {
    let (_, _) = char('S')(s)?;
    let (s, value) = alt((
        status_register,
        SINSTS,
        SINSTS1,
        SINSTS2,
        SINSTS3,
        SMAXSN,
        SMAXSN_Y,
        SMAXIN,
        SMAXIN_Y,
    ))(s)?;
    Ok((s, value))
}

fn extract_data_x(s: &str) -> IResult<&str, TicMsg> {
    let (s, value) = alt((RELAIS, NTARF, URMS1, URMS2, URMS3, UMOY1, UMOY2, UMOY3))(s)?;
    Ok((s, value))
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
_numeric_data!(NJOURF);
_numeric_data!(NJOURF_T, "NJOURF_1");
_numeric_data!(RELAIS);
_numeric_data!(SINSTS);
_numeric_data!(SINSTS1);
_numeric_data!(SINSTS2);
_numeric_data!(SINSTS3);

_numeric_data!(EAST);
_numeric_data!(EAIT);
_numeric_data!(URMS1);
_numeric_data!(URMS2);
_numeric_data!(URMS3);
_numeric_data!(IRMS1);
_numeric_data!(IRMS2);
_numeric_data!(IRMS3);

// text messages
_text_data!(MSG1);
_text_data!(MSG2);
_text_data!(ADSC);
_text_data!(NGTF);
_text_data!(LTARF);

_stamped_numeric!(DATE);
_stamped_numeric!(SMAXSN);
_stamped_numeric!(SMAXSN_Y, "SMAXSN-1");
_stamped_numeric!(SMAXIN);
_stamped_numeric!(SMAXIN_Y, "SMAXSN-1");
_stamped_numeric!(UMOY1);
_stamped_numeric!(UMOY2);
_stamped_numeric!(UMOY3);
// profile messages
_provider_profile!(PJOURF_T, "PJOURF+1");

// --- ignored messages ---
_ignore_data!(BASE);
_ignore_data!(BBRH);
_ignore_data!(CCAIN);
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
_ignore_data!(MOTDETAT);
_ignore_data!(OPTARIF);
_ignore_data!(PAPP);
_ignore_data!(PEJP);
_ignore_data!(PMAX);
_ignore_data!(PPOT);
_ignore_data!(PRM);
_ignore_data!(PTEC);
_ignore_data!(SMAX);
_ignore_data!(VTIC);
_ignore_data!(CCASN);

// Fulup note: size of nom 'alt' is limited which impose to split labels grammar
fn tic_data(s: &str) -> IResult<&str, TicMsg> {
    let (s, data) = alt((
        extract_data_a,
        extract_data_d,
        extract_data_e,
        extract_data_d,
        extract_data_i,
        extract_data_l,
        extract_data_m,
        extract_data_n,
        extract_data_p,
        extract_data_s,
        extract_data_x,
        extract_data_p,
        msg_to_ignore, // ignore any other messages
    ))(s)?;
    Ok((s, data))
}

pub fn tic_from_str(tic_str: &str) -> Result<TicMsg, LinkyError> {
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
    register_status::register()?;
    Ok(())
}
