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
use liblinky::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

struct EventDataCtx {
    pub handle: LinkyHandle,
    pub event: &'static AfbEvent,
    pub iinst: Rc<SensorHandleCtx>,
    pub sinsts: Rc<SensorHandleCtx>,
    pub adsp: Rc<SensorHandleCtx>,
    pub adsc: Rc<SensorHandleCtx>,
}

// this method is call each time a message is waiting on session raw_socket
AfbEvtFdRegister!(SerialAsyncCtrl, async_serial_cb, EventDataCtx);
fn async_serial_cb(_fd: &AfbEvtFd, revent: u32, ctx: &mut EventDataCtx) {
    // There is no value initializing a buffer before reading operation
    #[allow(invalid_value)]
    let mut buffer = unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() };

    if revent == AfbEvtFdPoll::IN.bits() {
        match ctx.handle.decode(&mut buffer) {
            Err(error) => {
                afb_log_msg!(
                    Error,
                    ctx.event,
                    "device:{} invalid data {:?}",
                    ctx.handle.get_name(),
                    error
                );
                ctx.event.broadcast(format!("{:?}", error));
            }
            Ok(data) => {
                match data {
                    // register status
                    TicValue::ADSC(value) => ctx.adsc.updated(data, 0, value.raw as i32),
                    TicValue::ADPS(value) => ctx.adsp.updated(data, 0, value),
                    // instant current
                    TicValue::IINST(value) => ctx.iinst.updated(data, 0, value),
                    TicValue::IINST1(value) => ctx.iinst.updated(data, 1, value),
                    TicValue::IINST2(value) => ctx.iinst.updated(data, 1, value),
                    TicValue::IINST3(value) => ctx.iinst.updated(data, 2, value),
                    // instant active current
                    TicValue::SINSTS(value) => ctx.sinsts.updated(data, 0, value),
                    TicValue::SINSTS1(value) => ctx.sinsts.updated(data, 1, value),
                    TicValue::SINSTS2(value) => ctx.sinsts.updated(data, 2, value),
                    TicValue::SINSTS3(value) => ctx.sinsts.updated(data, 3, value),
                    _ => {} // ignore any other data
                };
            }
        }
    } else {
        ctx.event.broadcast("tty-error");
    }
}

struct SensorHandleCtx {
    tic: &'static TicObject,
    event: &'static AfbEvent,
    values: Cell<[i32; 4]>,
}

// if new/old value diverge send event and update value cache
impl SensorHandleCtx {
    pub fn updated(&self, data: TicValue, idx: usize, value: i32) {
        let mut values = self.values.get();
        if value != values[idx] {
            values[idx] = value;
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
            for idx in 0.. ctx.handle.tic.get_count() {
                response.push(values[idx])?;
            }
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
            response.push(AFB_NO_DATA)?;
        }
        ApiAction::UNSUBSCRIBE => {
            ctx.handle.event.unsubscribe(rqt)?;
            response.push(AFB_NO_DATA)?;
        }
    }

    rqt.reply(response, 0);
    Ok(())
}

// register a new linky sensor
fn mk_sensor(api: &mut AfbApi, tic: &'static TicObject) -> Result<Rc<SensorHandleCtx>, AfbError> {
    let label = tic.get_label();
    let event = AfbEvent::new(label);
    let verb = AfbVerb::new(label);

    let ctx = Rc::new(SensorHandleCtx {
        tic,
        event,
        values: Cell::new([0; 4]),
    });

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
    liblinky::prelude::tic_register_type()?;

    let event_ctx = EventDataCtx {
        handle: LinkyHandle::new(config.device, config.speed, config.parity)?,
        event: AfbEvent::new("Linky"),
        iinst: mk_sensor(api, &TicObject::IINST)?,
        sinsts: mk_sensor(api, &TicObject::SINSTS)?,
        adsp: mk_sensor(api, &TicObject::ADPS)?,
        adsc: mk_sensor(api, &TicObject::ADSC)?,
    };

    AfbEvtFd::new(config.device)
        .set_fd(event_ctx.handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(event_ctx))
        .start()?;

    Ok(())
}
