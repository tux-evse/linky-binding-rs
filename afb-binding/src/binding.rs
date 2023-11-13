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

use afbv4::prelude::*;
use crate::prelude::*;

pub(crate) fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

AfbDataConverter!(api_actions, ApiAction);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "UPPERCASE")]
pub(crate)enum ApiAction {
    #[default]
    READ,
    SUBSCRIBE,
    UNSUBSCRIBE
}

pub struct LinkyConfig {
    pub uid: &'static str,
    pub device: &'static str,
    pub parity: &'static str,
    pub speed: u32,
}

impl AfbApiControls for LinkyConfig {
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
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);


    let uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "linky"
    };

    let api = if let Ok(value) = jconf.get::<String>("api") {
        to_static_str(value)
    } else {
        uid
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    let acls = if let Ok(value) = jconf.get::<String>("acls") {
        AfbPermission::new(to_static_str(value))
    } else {
        AfbPermission::new("acl:linky:client")
    };

    let device = if let Ok(value) = jconf.get::<String>("device") {
        to_static_str(value)
    } else {
        return Err(AfbError::new("linky-config-fail", "mandatory label 'device' missing"))
    };


    let speed= if let Ok(value) = jconf.get::<u32>("speed") {
       value
    } else {
        1200
    };

    let parity= if let Ok(value) = jconf.get::<String>("parity") {
       to_static_str(value)
    } else {
        "even"
    };

    // register data converter
    // v106::register_datatype() ?;

    let config= LinkyConfig {
        uid,
        device,
        speed,
        parity,
    };

    // create backend API
    let api = AfbApi::new(api).set_info(info).set_permission(acls);
    register_verbs(api, config)?;

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
