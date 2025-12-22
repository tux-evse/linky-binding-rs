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
use linky::prelude::*;
use std::time::Duration;

AfbDataConverter!(api_actions, ApiAction);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase", tag = "action")]
pub enum ApiAction {
    #[default]
    READ,
    INFO,
    SUBSCRIBE,
    UNSUBSCRIBE,
}

pub struct BindingConfig {
    pub uid: &'static str,
    pub source: LinkyConfig,
    pub cycle: Option<Duration>,
    pub sensors: JsoncObj,
}

impl AfbApiControls for BindingConfig {
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        Ok(())
    }

    // mandatory for downcasting back to custom api data object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(_rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {

    // add binding custom converter
    api_actions::register()?;

    let uid = jconf.default("uid", "linky")?;
    let api = jconf.default("api", uid)?;
    let info = jconf.default("info", "Linky meeter binding")?;
    let cycle = match jconf.optional("cycle")? {
        None => None,
        Some(value) => Some(Duration::from_secs(value)),
    };

    let source = match jconf.optional::<JsoncObj>("serial")? {
        Some(jserial) => {
            let device = jserial.default("device", "/dev/ttyUSB0")?;
            let speed = jserial.default("speed", 9600)?;
            let parity = jserial.default("parity", "even")?;
            LinkyConfig::Serial(SerialConfig {
                device,
                speed,
                parity,
            })
        }

        None => match jconf.optional::<JsoncObj>("network")? {
            Some(jnetwork) => {
                let ip_bind = jnetwork.default("bind", "0.0.0.0")?;
                let udp_port = jnetwork.default("port", 2000)?;
                LinkyConfig::Network(NetworkConfig { ip_bind, udp_port })
            }
            None => {
                return afb_error!(
                    "linky-config-fail",
                    "unsupported source type: should be serial|network",
                )
            }
        },
    };

    // sensors list is processed within BindingConfig
    let sensors = jconf.get("sensors")?;

    let config: BindingConfig = BindingConfig { uid, source, cycle, sensors };

    // create backend API
    let api = AfbApi::new(api).set_info(info);
    register_verbs(api, &config)?;

    // if acls defined apply them
    if let Some(value) = jconf.optional::<&str>("permission")? {
        api.set_permission(AfbPermission::new(value));
    };

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
