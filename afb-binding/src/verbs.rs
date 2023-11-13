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
use liblinky::prelude::*;
use std::rc::Rc;
use std::cell::Cell;
use ::core::mem::MaybeUninit;

struct SensorCtxData {
    event: &'static AfbEvent,
    value: Cell<i32>,
}

impl SensorCtxData {
    pub fn notify(&self, value: i32) {
        if value != self.value.get() {
            self.value.set(value);
            self.event.push(value);
        }
    }
}

struct SerialCtx {
    pub handle: LinkyHandle,
    pub error: &'static AfbEvent,
    pub iinst: Rc<SensorCtxData>,
    pub imax: Rc<SensorCtxData>,
}

struct IinstCtxData {
    sensor: Rc<SensorCtxData>,
}

AfbVerbRegister!(IinstVerb, iinst_cb, IinstCtxData);
fn iinst_cb(rqt: &AfbRequest, args: &AfbData, ctx: &mut IinstCtxData) -> Result<(), AfbError> {
    match args.get::<&ApiAction>(0)? {
        ApiAction::READ => {}
        ApiAction::SUBSCRIBE => {ctx.sensor.event.subscribe(rqt)?;},
        ApiAction::UNSUBSCRIBE => {ctx.sensor.event.unsubscribe(rqt)?;},
    }
    rqt.reply(ctx.sensor.value.get(), 0);
    Ok(())
}

// this method is call each time a message is waiting on session raw_socket
AfbEvtFdRegister!(SerialAsyncCtrl, async_serial_cb, SerialCtx);
fn async_serial_cb(_fd: &AfbEvtFd, revent: u32, ctx: &mut SerialCtx) {

    // There is no value initializing a buffer before reading operation
    #[allow(invalid_value)]
    let mut buffer= unsafe{MaybeUninit::<[u8;256]>::uninit().assume_init()};

    if revent == AfbEvtFdPoll::IN.bits() {
        match ctx.handle.decode(&mut buffer) {
            Err(error) => {
                afb_log_msg!(
                    Error,
                    ctx.error,
                    "device:{} invalid data {:?}",
                    ctx.handle.get_name(),
                    error
                );
                ctx.error.push(format!("{:?}", error));
            }
            Ok(data) => match data {
                TicValue::IMAX(value) => ctx.iinst.notify(value),
                TicValue::IINST(value) => ctx.imax.notify(value),
                TicValue::LTARF(value) => {} // Fulup TBD

                _ => {},
            },
        }
    }
}

// register a new linky sensor
fn mk_sensor(api: &mut AfbApi, verb: &mut AfbVerb, tic: &'static TicObject) -> Result<Rc<SensorCtxData>, AfbError> {
    let label = tic.get_label();
    let sensor = Rc::new(SensorCtxData {
        value: Cell::new(0),
        event: AfbEvent::new(label),
    });

    api.add_event(sensor.event);

    verb.set_info(tic.get_info());
    verb.set_action("['reset', 'subscribe', 'unsubscribe']")?;
    Ok(sensor)
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: LinkyConfig) -> Result<(), AfbError> {

    let verb = AfbVerb::new("IINST");
    let iinst = mk_sensor(api, verb,&TicObject::IINST)?;
    verb.set_callback(Box::new(IinstCtxData {
        sensor: iinst.clone(),
    }));
    verb.finalize()?;
    api.add_verb(verb);

    let verb = AfbVerb::new("IMAX");
    let imax = mk_sensor(api, verb,&TicObject::IMAX)?;
    verb.set_callback(Box::new(IinstCtxData {
        sensor: iinst.clone(),
    }));
    verb.finalize()?;
    api.add_verb(verb);


    let serial_ctx= SerialCtx {
            handle: LinkyHandle::new(config.device, config.speed, config.parity)?,
            error: AfbEvent::new("Linky/Error"),
            iinst,
            imax,
        };

    AfbEvtFd::new(config.device)
        .set_fd(serial_ctx.handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(serial_ctx))
        .start()?;

    Ok(())
}
