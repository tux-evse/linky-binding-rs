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
use ::core::mem::MaybeUninit;
use afbv4::prelude::*;
use linky::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SensorNumericData {
    cycle: u32,
    counters: [i32; 4],
}

struct SensorNumericCtx {
    multi: bool,
    tic: &'static TicObject,
    event: &'static AfbEvent,
    values: RefCell<SensorNumericData>,
}

// if new/old value diverge send event and update value cache
impl SensorNumericCtx {
    pub fn new(tic: &'static TicObject, event: &'static AfbEvent, multi: bool) -> Self {
        Self {
            multi,
            tic,
            event,
            values: RefCell::new(SensorNumericData {
                cycle: 0,
                counters: [0; 4],
            }),
        }
    }

    pub fn updated(
        &self,
        cycle: u32,
        data: TicMsg,
        idx: usize,
        value: i32,
    ) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-msg-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };
        // increase cycle counter and force event if needed
        let forced = if cycle > 0 {
            if values.cycle == cycle {
                values.cycle = 0;
                true
            } else {
                values.cycle += 1;
                false
            }
        } else {
            false
        };

        if value != values.counters[idx] || forced {
            values.counters[idx] = value;
            values.cycle = 0;
            self.event.push(data);
        }
        Ok(())
    }
}

pub struct SensorTextCtx {
    multi: bool,
    tic: &'static TicObject,
    values: RefCell<[String; 2]>,
}

impl SensorTextCtx {
    pub fn new(tic: &'static TicObject, multi: bool) -> Self {
        Self {
            multi,
            tic,
            values: RefCell::new(["--".to_string(), "--".to_string()]),
        }
    }

    pub fn updated(&self, index: usize, text: String) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-msg-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };

        values[index] = text;
        Ok(())
    }
}

pub struct SensorProfileCtx {
    multi: bool,
    tic: &'static TicObject,
    values: RefCell<[ProviderProfile; 2]>,
}

impl SensorProfileCtx {
    pub fn new(tic: &'static TicObject, next_day: &str, next_pic: &str, multi: bool) -> Self {
        Self {
            multi,
            tic,
            values: RefCell::new([
                ProviderProfile::new(next_day),
                ProviderProfile::new(next_pic),
            ]),
        }
    }

    pub fn updated(&self, index: usize, profile: ProviderProfile) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-msg-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };

        values[index] = profile;
        Ok(())
    }
}

pub struct SensorStampCtx {
    tic: &'static TicObject,
    values: RefCell<TimeStampData>,
}

impl SensorStampCtx {
    pub fn new(tic: &'static TicObject) -> Result<Self, AfbError> {
        let obj = Self {
            tic,
            values: RefCell::new(TimeStampData::new("H000000000000", None)?),
        };

        Ok(obj)
    }

    pub fn updated(&self, _idx: usize, stamp_data: TimeStampData) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-msg-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };

        *values = stamp_data;
        Ok(())
    }
}

pub struct EnergyCountersCtx {
    tic: &'static TicObject,
    values: RefCell<[i32;2]>,
}

impl EnergyCountersCtx {
    pub fn new(tic: &'static TicObject) -> Result<Self, AfbError> {
        let obj = Self {
            tic,
            values: RefCell::new([0;2]),
        };
        Ok(obj)
    }

    pub fn updated(&self, idx: usize, energy: i32) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-energy-ctx-fail", "fail to access energy value ctx"),
            Ok(value) => value,
        };

        values[idx] = energy;
        Ok(())
    }
}


pub struct SensorPowerCtx {
    tic: &'static TicObject,
    values: RefCell<[TimeStampData;2]>,
}

impl SensorPowerCtx {
    pub fn new(tic: &'static TicObject) -> Result<Self, AfbError> {
        let empty=TimeStampData::new("H000000000000", None)?;
        Ok(Self {
            tic,
            values: RefCell::new([empty.clone(), empty]),
        })
    }

    pub fn updated(&self, idx: usize, data: TimeStampData) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-power-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };
        values[idx] = data;
        Ok(())
    }
}

pub struct SensorRegisterCtx {
    tic: &'static TicObject,
    values: RefCell<RegisterStatus>,
}

impl SensorRegisterCtx {
    pub fn new(tic: &'static TicObject) -> Result<Self, AfbError> {
        Ok(Self {
            tic,
            values: RefCell::new(RegisterStatus::new()),
        })
    }

    pub fn updated(&self, _idx: usize, register: RegisterStatus) -> Result<(), AfbError> {
        let mut values = match self.values.try_borrow_mut() {
            Err(_) => return afb_error!("update-msg-ctx-fail", "fail to access sensor value ctx"),
            Ok(value) => value,
        };

        *values = register;
        Ok(())
    }
}

struct EventDataCtx {
    pub cycle: u32,
    pub handle: LinkyHandle,
    pub event: &'static AfbEvent,
    pub iinst: Option<Rc<SensorNumericCtx>>,
    pub sinsts: Option<Rc<SensorNumericCtx>>,
    pub adsp: Option<Rc<SensorNumericCtx>>,
    pub pcou: Option<Rc<SensorNumericCtx>>,
    pub ntarf: Option<Rc<SensorNumericCtx>>,
    pub irms: Option<Rc<SensorNumericCtx>>,
    pub umoy: Option<Rc<SensorStampCtx>>,
    pub urms: Option<Rc<SensorNumericCtx>>,
    pub energy: Option<Rc<EnergyCountersCtx>>,
    pub njourf: Option<Rc<SensorNumericCtx>>,
    pub msg: Option<Rc<SensorTextCtx>>,
    pub adsc: Option<Rc<SensorTextCtx>>,
    pub tariff: Option<Rc<SensorTextCtx>>,
    pub profile: Option<Rc<SensorProfileCtx>>,
    pub date: Option<Rc<SensorStampCtx>>,
    pub powerin: Option<Rc<SensorPowerCtx>>,
    pub powerout: Option<Rc<SensorPowerCtx>>,
    pub stge: Option<Rc<SensorRegisterCtx>>,
}

// this method is call each time a message is waiting on session raw_socket
//AfbEvtFdRegister!(SerialAsyncCtrl, async_msg_cb, EventDataCtx);
fn async_msg_cb(
    _fd: &AfbEvtFd,
    revent: u32,
    ctx: &AfbCtxData, //&mut EventDataCtx
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<EventDataCtx>()?;

    // There is no value initializing a buffer before reading operation
    #[allow(invalid_value)]
    let mut buffer = unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() };

    if revent == AfbEvtFdPoll::IN.bits() {
        loop {
            match ctx.handle.decode(&mut buffer) {
                Err(error) => match error {
                    LinkyError::RetryLater => break, // force buffer read,
                    LinkyError::ChecksumError(_) => { /* ignored */ }
                    _ => {
                        afb_log_msg!(
                            Debug,
                            ctx.event,
                            "device:{} invalid data {:?}",
                            ctx.handle.get_uid(),
                            error
                        );
                        ctx.event.broadcast(format!("{:?}", error));
                        break;
                    }
                },
                Ok((tic_msg, eob)) => {
                    macro_rules! _profile_num_update {
                        ($label:ident, $idx:expr, $cycle:expr, $value:expr) => {
                            match &ctx.$label {
                                Some(ctx) => ctx.updated($cycle, tic_msg, $idx, $value)?,
                                None => {}
                            }
                        };
                    }
                    macro_rules! _profile_ronly_update {
                        ($label:ident, $idx:expr, $value:expr) => {
                            match &ctx.$label {
                                Some(ctx) => ctx.updated($idx, $value)?,
                                None => {}
                            }
                        };
                    }
                    match tic_msg {
                        // no data ignore buffer
                        TicMsg::NODATA => break,
                        TicMsg::IGNORED => continue,

                        // over power
                        TicMsg::ADPS(value) => _profile_num_update!(adsp, 0, ctx.cycle, value),
                        TicMsg::ADIR1(value) => _profile_num_update!(adsp, 1, ctx.cycle, value),
                        TicMsg::ADIR2(value) => _profile_num_update!(adsp, 2, ctx.cycle, value),
                        TicMsg::ADIR3(value) => _profile_num_update!(adsp, 3, ctx.cycle, value),


                        // cutting power
                        TicMsg::PCOUP(value) => _profile_num_update!(pcou, 0, ctx.cycle, value),
                        TicMsg::PREF(value) => _profile_num_update!(pcou, 1, ctx.cycle, value),

                        // instant current
                        TicMsg::IINST(value) => _profile_num_update!(iinst, 0, ctx.cycle, value),
                        TicMsg::IINST1(value) => _profile_num_update!(iinst, 1, ctx.cycle, value),
                        TicMsg::IINST2(value) => _profile_num_update!(iinst, 2, ctx.cycle, value),
                        TicMsg::IINST3(value) => _profile_num_update!(iinst, 3, ctx.cycle, value),

                        // instant active current
                        TicMsg::SINSTS(value) => _profile_num_update!(sinsts, 0, ctx.cycle, value),
                        TicMsg::SINSTS1(value) => _profile_num_update!(sinsts, 1, ctx.cycle, value),
                        TicMsg::SINSTS2(value) => _profile_num_update!(sinsts, 2, ctx.cycle, value),
                        TicMsg::SINSTS3(value) => _profile_num_update!(sinsts, 3, ctx.cycle, value),

                        // efficient current
                        TicMsg::IRMS1(value) => _profile_num_update!(irms, 0, ctx.cycle, value),
                        TicMsg::IRMS2(value) => _profile_num_update!(irms, 1, ctx.cycle, value),
                        TicMsg::IRMS3(value) => _profile_num_update!(irms, 2, ctx.cycle, value),

                        // efficient tension
                        TicMsg::URMS1(value) => _profile_num_update!(urms, 0, ctx.cycle, value),
                        TicMsg::URMS2(value) => _profile_num_update!(urms, 1, ctx.cycle, value),
                        TicMsg::URMS3(value) => _profile_num_update!(urms, 2, ctx.cycle, value),

                        // Tariff index
                        TicMsg::NTARF(value) => _profile_num_update!(ntarf, 0, ctx.cycle, value),
                        TicMsg::NJOURF(value) => _profile_num_update!(njourf, 0, ctx.cycle, value),
                        TicMsg::NJOURF_T(value) => _profile_num_update!(njourf, 1, ctx.cycle, value),

                        // Power
                        TicMsg::EAST(value) => _profile_ronly_update!(energy, 0, value),
                        TicMsg::EAIT(value) => _profile_ronly_update!(energy, 1, value),
                        TicMsg::SMAXSN(value) => _profile_ronly_update!(powerin, 0, value),
                        TicMsg::SMAXSN_Y(value) => _profile_ronly_update!(powerin, 1, value),
                        TicMsg::SMAXIN(value) => _profile_ronly_update!(powerout, 0, value),
                        TicMsg::SMAXIN_Y(value) => _profile_ronly_update!(powerout, 1, value),

                        // read only linky register
                        TicMsg::MSG1(value) => _profile_ronly_update!(msg, 0, value),
                        TicMsg::MSG2(value) => _profile_ronly_update!(msg, 1, value),
                        TicMsg::ADSC(value) => _profile_ronly_update!(adsc, 0, value),
                        TicMsg::NGTF(value) => _profile_ronly_update!(tariff, 0, value),
                        TicMsg::LTARF(value) => _profile_ronly_update!(tariff, 1, value),

                        TicMsg::DATE(value) => _profile_ronly_update!(date, 0, value),
                        TicMsg::PJOURF_T(value) => _profile_ronly_update!(profile, 0, value),
                        TicMsg::PPOINTE(value) => _profile_ronly_update!(profile, 0, value),
                        TicMsg::STGE(value) => _profile_ronly_update!(stge, 0, value),

                        // average voltage
                        TicMsg::UMOY1(value) => _profile_ronly_update!(umoy, 0, value),
                        //TicMsg::UMOY2(value) => _profile_num_update!(umoy, 2, value),
                        //TicMsg::UMOY3(value) => _profile_num_update!(umoy, 3,value),

                        _ => {} // ignore any other data
                    };
                    // end of buffer reached, wait for new one
                    if eob {
                        break;
                    }
                }
            }
        }
    } else {
        ctx.event.broadcast("data-input-error");
    }
    Ok(())
}


struct NumericSensorVcb {
    handle: Rc<SensorNumericCtx>,
}

fn sensor_numeric_cb(
    rqt: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<NumericSensorVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => {
                    return afb_error!("sensor-numeric-cb", "fail to access sensor value ctx")
                }
                Ok(value) => value,
            };

            let jsonc = if ctx.handle.multi {
                let jsonc = JsoncObj::array();
                for idx in 0..values.counters.len() {
                    jsonc.insert(idx, values.counters[idx])?;
                }
                jsonc
            } else {
                JsoncObj::import(values.counters[0] as i64)?
            };

            response.push(jsonc)?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        ApiAction::SUBSCRIBE => {
            ctx.handle.event.subscribe(rqt)?;
        }
        ApiAction::UNSUBSCRIBE => {
            ctx.handle.event.unsubscribe(rqt)?;
        }
    }

    rqt.reply(response, 0);
    Ok(())
}

// register a new linky sensor
fn mk_numeric_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    multi: u8,
) -> Result<Rc<SensorNumericCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let event = AfbEvent::new(name);
    let verb = AfbVerb::new(name);

    let ctx = Rc::new(SensorNumericCtx::new(tic, event, multi!=0));

    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info', 'subscribe', 'unsubscribe']")?;
    verb.set_callback(sensor_numeric_cb); //
    verb.set_context(NumericSensorVcb {
        handle: ctx.clone(),
    });

    verb.finalize()?;

    api.add_verb(verb);
    api.add_event(event);
    Ok(ctx)
}

struct TextSensorVcb {
    handle: Rc<SensorTextCtx>,
}

fn sensor_text_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<TextSensorVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-masg-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            let jsonc = if ctx.handle.multi {
                let jsonc = JsoncObj::array();
                for idx in 0..values.len() {
                    jsonc.insert(idx, &values[idx])?;
                }
                jsonc
            } else {
                JsoncObj::import(&values[0])?
            };

            response.push(jsonc)?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-msg-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

// text sensors do not send events
fn mk_text_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    multi: u8,
) -> Result<Rc<SensorTextCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);

    let ctx = Rc::new(SensorTextCtx::new(tic, multi!=0));

    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(sensor_text_cb); //
    verb.set_context(TextSensorVcb {
        handle: ctx.clone(),
    });

    verb.finalize()?;

    api.add_verb(verb);
    Ok(ctx)
}

struct StampSensorVcb {
    handle: Rc<SensorStampCtx>,
}

fn sensor_stamp_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<StampSensorVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-stamp-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            // push stamp and data if any
            let jsonc = values.to_jsonc()?;
            response.push(jsonc)?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-stamp-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

// date sensors do not send events
fn mk_stamp_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    _multi: u8,
) -> Result<Rc<SensorStampCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);
    let ctx = Rc::new(SensorStampCtx::new(tic)?);
    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(sensor_stamp_cb); //
    verb.set_context(StampSensorVcb {
        handle: ctx.clone(),
    });
    verb.finalize()?;
    api.add_verb(verb);
    Ok(ctx)
}

struct RegisterSensorVcb {
    handle: Rc<SensorRegisterCtx>,
}

fn sensor_register_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<RegisterSensorVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-register-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            response.push(values.clone())?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-register-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

fn mk_register_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    _multi: u8,
) -> Result<Rc<SensorRegisterCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);
    let ctx = Rc::new(SensorRegisterCtx::new(tic)?);
    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(sensor_register_cb); //
    verb.set_context(RegisterSensorVcb {
        handle: ctx.clone(),
    });
    verb.finalize()?;
    api.add_verb(verb);
    Ok(ctx)
}

struct EnergyCountersVcb {
    handle: Rc<EnergyCountersCtx>,
}

fn energy_counter_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<EnergyCountersVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            const DIRECTIONS:[&str;2]= ["consumed", "injected"];
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-energy-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            // push power and data if any
            let jsonc= JsoncObj::new();
            for idx in 0 .. 2 {
                jsonc.add(DIRECTIONS[idx], values[idx])?;
            }
            response.push(jsonc)?;
        }

        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-energy-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

fn mk_energy_counters(
    api: &mut AfbApi,
    tic: &'static TicObject,
    _multi: u8,
) -> Result<Rc<EnergyCountersCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);
    let ctx = Rc::new(EnergyCountersCtx::new(tic)?);
    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(energy_counter_cb); //
    verb.set_context(EnergyCountersVcb {
        handle: ctx.clone(),
    });
    verb.finalize()?;
    api.add_verb(verb);
    Ok(ctx)
}

struct TextProfileVcb {
    handle: Rc<SensorProfileCtx>,
}

fn sensor_profile_cb(
    rqt: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<TextProfileVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-masg-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            let jsonc = if ctx.handle.multi {
                let jsonc = JsoncObj::array();
                for idx in 0..values.len() {
                    jsonc.insert(idx, &values[idx].to_jsonc()?)?;
                }
                jsonc
            } else {
                values[0].to_jsonc()?
            };

            response.push(jsonc)?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-msg-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

// text sensors do not send events
fn mk_profile_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    multi: u8,
) -> Result<Rc<SensorProfileCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);

    let ctx = Rc::new(SensorProfileCtx::new(tic, "next-day", "next-pic", multi!=0));

    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(sensor_profile_cb); //
    verb.set_context(TextProfileVcb {
        handle: ctx.clone(),
    });

    verb.finalize()?;
    api.add_verb(verb);
    Ok(ctx)
}


struct PowerSensorVcb {
    handle: Rc<SensorPowerCtx>,
}

fn sensor_power_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<PowerSensorVcb>()?;

    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            const DAYS:[&str;2]= ["today", "yesterday"];
            let values = match ctx.handle.values.try_borrow() {
                Err(_) => return afb_error!("sensor-power-cb", "fail to access sensor value ctx"),
                Ok(value) => value,
            };

            // push power and data if any
            let jsonc= JsoncObj::new();
            for idx in 0 .. 2 {
                jsonc.add(DAYS[idx], values[idx].to_jsonc()?)?;
            }
            response.push(jsonc)?;
        }
        ApiAction::INFO => {
            let info = match serde_json::to_string(ctx.handle.tic) {
                Ok(value) => value,
                Err(_) => "no-sensor-info".to_string(),
            };
            response.push(info)?;
        }
        _ => return afb_error!("sensor-power-cb", "read only data without subscription"),
    }

    rqt.reply(response, 0);
    Ok(())
}

// date sensors do not send events
fn mk_power_sensor(
    api: &mut AfbApi,
    tic: &'static TicObject,
    _multi: u8,
) -> Result<Rc<SensorPowerCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let verb = AfbVerb::new(name);
    let ctx = Rc::new(SensorPowerCtx::new(tic)?);
    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_actions("['read', 'info']")?;
    verb.set_callback(sensor_power_cb); //
    verb.set_context(PowerSensorVcb {
        handle: ctx.clone(),
    });
    verb.finalize()?;
    api.add_verb(verb);
    Ok(ctx)
}


pub fn register_verbs(api: &mut AfbApi, config: &BindingConfig) -> Result<(), AfbError> {
    // register custom parser afb-v4 type within binder
    linky::prelude::tic_register_type()?;

    let tariff = match config.sensors.optional("TARIFF")? {
        Some(count) => Some(mk_text_sensor(api, &TicObject::TARIFF, count)?),
        None => None,
    };

    let handle = LinkyHandle::new(&config.source)?;
        let ntarf = match config.sensors.optional("NTARF")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::NTARF, count)?),
        None => None,
    };

    let iinst = match config.sensors.optional("IINSTS")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::IINST, count)?),
        None => None,
    };

    let sinsts = match config.sensors.optional("SINSTS")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::SINSTS, count)?),
        None => None,
    };

    let adsp = match config.sensors.optional("ADPS")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::ADPS, count)?),
        None => None,
    };

    let pcou = match config.sensors.optional("PCOU")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::PCOUP, count)?),
        None => None,
    };

    let njourf = match config.sensors.optional("NJOURF")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::NJOURF, count)?),
        None => None,
    };

    let energy = match config.sensors.optional("ENERGY")? {
        Some(count) => Some(mk_energy_counters(api, &TicObject::ENERGY, count)?),
        None => None,
    };
    let profile = match config.sensors.optional("PROFILE")? {
        Some(count) => Some(mk_profile_sensor(api, &TicObject::PROFILE, count)?),
        None => None,
    };

    let adsc = match config.sensors.optional("ADSC")? {
        Some(count) => Some(mk_text_sensor(api, &TicObject::ADSC, count)?),
        None => None,
    };


    let msg = match config.sensors.optional("MSG")? {
        Some(count) => Some(mk_text_sensor(api, &TicObject::MSG, count)?),
        None => None,
    };

    let date = match config.sensors.optional("DATE")? {
        Some(count) => Some(mk_stamp_sensor(api, &TicObject::DATE, count)?),
        None => None,
    };

    let stge = match config.sensors.optional("STGE")? {
        Some(count) => Some(mk_register_sensor(api, &TicObject::STGE, count)?),
        None => None,
    };

    let powerin = match config.sensors.optional("POWER-IN")? {
        Some(count) => Some(mk_power_sensor(api, &TicObject::POWERIN, count)?),
        None => None,
    };

    let powerout = match config.sensors.optional("POWER-OUT")? {
        Some(count) => Some(mk_power_sensor(api, &TicObject::POWEROUT, count)?),
        None => None,
    };

    let umoy = match config.sensors.optional("UMOY")? {
        Some(count) => Some(mk_stamp_sensor(api, &TicObject::UMOY, count)?),
        None => None,
    };
    let urms = match config.sensors.optional("URMS")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::URMS, count)?),
        None => None,
    };
    let irms = match config.sensors.optional("IRMS")? {
        Some(count) => Some(mk_numeric_sensor(api, &TicObject::IRMS, count)?),
        None => None,
    };
    let event_ctx = EventDataCtx {
        cycle: config.cycle,
        handle: handle,
        event: AfbEvent::new("data_msg"),
        iinst,
        sinsts,
        adsp,
        adsc,
        pcou,
        ntarf,
        njourf,
        profile,
        msg,
        date,
        tariff,
        energy,
        powerin,
        powerout,
        stge,
        umoy,
        urms,
        irms,
    };

    api.add_event(event_ctx.event);

    AfbEvtFd::new(config.uid)
        .set_fd(event_ctx.handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(async_msg_cb)
        .set_context(event_ctx)
        .start()?;

    Ok(())
}
