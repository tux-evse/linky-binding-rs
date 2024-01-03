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
use std::cell::Cell;
use std::rc::Rc;

struct SensorHandleCtx {
    tic: &'static TicObject,
    event: &'static AfbEvent,
    values: Cell<[i32; 4]>,
    count: Cell<u32>,
}

struct EventDataCtx {
    pub cycle: u32,
    pub handle: LinkyHandle,
    pub event: &'static AfbEvent,
    pub iinst: Rc<SensorHandleCtx>,
    pub sinsts: Rc<SensorHandleCtx>,
    pub adsp: Rc<SensorHandleCtx>,
    pub adsc: Rc<SensorHandleCtx>,
    pub pcou: Rc<SensorHandleCtx>,
    pub ntarf: Rc<SensorHandleCtx>,
    pub irms: Rc<SensorHandleCtx>,
    pub urms: Rc<SensorHandleCtx>,
}

// this method is call each time a message is waiting on session raw_socket
AfbEvtFdRegister!(SerialAsyncCtrl, async_serial_cb, EventDataCtx);
fn async_serial_cb(_fd: &AfbEvtFd, revent: u32, ctx: &mut EventDataCtx) -> Result<(), AfbError>{
    // There is no value initializing a buffer before reading operation
    #[allow(invalid_value)]
    let mut buffer = unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() };

    if revent == AfbEvtFdPoll::IN.bits() {
        match ctx.handle.decode(&mut buffer) {
            Err(error) => match error {
                LinkyError::ChecksumError(_) => {}
                _ => {
                    afb_log_msg!(
                        Debug,
                        ctx.event,
                        "device:{} invalid data {:?}",
                        ctx.handle.get_name(),
                        error
                    );
                    ctx.event.broadcast(format!("{:?}", error));
                }
            },
            Ok(data) => {
                match data {
                    // register status
                    TicValue::ADSC(value) => ctx.adsc.updated(ctx.cycle, data, 0, value.raw as i32),

                    // over power
                    TicValue::ADPS(value) => ctx.adsp.updated(ctx.cycle, data, 0, value),
                    TicValue::ADIR1(value) => ctx.adsp.updated(ctx.cycle, data, 1, value),
                    TicValue::ADIR2(value) => ctx.adsp.updated(ctx.cycle, data, 2, value),
                    TicValue::ADIR3(value) => ctx.adsp.updated(ctx.cycle, data, 3, value),

                    // cutting power
                    TicValue::PCOUP(value) => ctx.pcou.updated(ctx.cycle, data, 0, value),
                    TicValue::PREF(value) => ctx.pcou.updated(ctx.cycle, data, 1, value),

                    // instant current
                    TicValue::IINST(value) => ctx.iinst.updated(ctx.cycle, data, 0, value),
                    TicValue::IINST1(value) => ctx.iinst.updated(ctx.cycle, data, 1, value),
                    TicValue::IINST2(value) => ctx.iinst.updated(ctx.cycle, data, 2, value),
                    TicValue::IINST3(value) => ctx.iinst.updated(ctx.cycle, data, 3, value),

                    // instant active current
                    TicValue::SINSTS(value) => ctx.sinsts.updated(ctx.cycle, data, 0, value),
                    TicValue::SINSTS1(value) => ctx.sinsts.updated(ctx.cycle, data, 1, value),
                    TicValue::SINSTS2(value) => ctx.sinsts.updated(ctx.cycle, data, 2, value),
                    TicValue::SINSTS3(value) => ctx.sinsts.updated(ctx.cycle, data, 3, value),

                    // efficient current
                    TicValue::IRMS1(value) => ctx.irms.updated(ctx.cycle, data, 0, value),
                    TicValue::IRMS2(value) => ctx.irms.updated(ctx.cycle, data, 1, value),
                    TicValue::IRMS3(value) => ctx.irms.updated(ctx.cycle, data, 2, value),

                    // efficient tension
                    TicValue::URMS1(value) => ctx.urms.updated(ctx.cycle, data, 0, value),
                    TicValue::URMS2(value) => ctx.urms.updated(ctx.cycle, data, 1, value),
                    TicValue::URMS3(value) => ctx.urms.updated(ctx.cycle, data, 2, value),

                    // Index tarrifaire
                    TicValue::NTARF(value) => ctx.ntarf.updated(ctx.cycle, data, 1, value),

                    _ => {} // ignore any other data
                };
            }
        }
    } else {
        ctx.event.broadcast("tty-error");
    }
    Ok(())
}

// if new/old value diverge send event and update value cache
impl SensorHandleCtx {
    pub fn updated(&self, cycle: u32, data: TicValue, idx: usize, value: i32) {
        let mut values = self.values.get();

        // increase cycle counter and force event if needed
        let forced = if cycle > 0 {
            let count = self.count.get();
            if count == cycle {
                true
            } else {
                self.count.set(count+1);
                false
            }
        } else {
            false
        };

        if value != values[idx] || forced {
            values[idx] = value;
            self.count.set(0);
            self.values.set(values);
            self.event.push(data);
        }
    }
}

struct SensorDataCtx {
    handle: Rc<SensorHandleCtx>,
}
AfbVerbRegister!(SensorVerb, sensorcb, SensorDataCtx);
fn sensorcb(rqt: &AfbRequest, args: &AfbData, ctx: &mut SensorDataCtx) -> Result<(), AfbError> {
    let mut response = AfbParams::new();
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {
            let values = ctx.handle.values.get();
            let jsonc= JsoncObj::array();
            for idx in 0..ctx.handle.tic.get_count() {
                jsonc.insert(values[idx])?;
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
fn mk_sensor(api: &mut AfbApi, tic: &'static TicObject) -> Result<Rc<SensorHandleCtx>, AfbError> {
    let uid = tic.get_uid();
    let name = tic.get_name();
    let event = AfbEvent::new(name);
    let verb = AfbVerb::new(name);

    let ctx = Rc::new(SensorHandleCtx {
        tic,
        event,
        values: Cell::new([0; 4]),
        count: Cell::new(0),
    });

    verb.set_name(uid);
    verb.set_info(tic.get_info());
    verb.set_action("['read', 'info', 'subscribe', 'unsubscribe']")?;
    verb.set_callback(Box::new(SensorDataCtx {
        handle: ctx.clone(),
    }));
    verb.finalize()?;

    api.add_verb(verb);
    api.add_event(event);
    Ok(ctx)
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: LinkyConfig) -> Result<(), AfbError> {
    // register custom parser afb-v4 type within binder
    linky::prelude::tic_register_type()?;
    let event = AfbEvent::new("Serial");

    let event_ctx = EventDataCtx {
        cycle: config.cycle,
        handle: LinkyHandle::new(config.device, config.speed, config.parity)?,
        event: event,
        iinst: mk_sensor(api, &TicObject::IINST)?,
        sinsts: mk_sensor(api, &TicObject::SINSTS)?,
        adsp: mk_sensor(api, &TicObject::ADPS)?,
        adsc: mk_sensor(api, &TicObject::ADSC)?,
        pcou: mk_sensor(api, &TicObject::PCOUP)?,
        ntarf: mk_sensor(api, &TicObject::NTARF)?,
        irms: mk_sensor(api, &TicObject::IRMS)?,
        urms: mk_sensor(api, &TicObject::URMS)?,
    };

    api.add_event(event);

    AfbEvtFd::new(config.device)
        .set_fd(event_ctx.handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(event_ctx))
        .start()?;

    Ok(())
}
