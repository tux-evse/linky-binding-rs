/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
use std::env;

fn main() {
    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=capi/capi-map.h");
    println!("cargo:rustc-link-search=/usr/local/lib64");
    if let Ok(value) = env::var("CARGO_TARGET_DIR") {
        if let Ok(profile) = env::var("PROFILE") {
            println!("cargo:rustc-link-search=crate={}{}", value, profile);
        }
    }

    let header = "
    // -----------------------------------------------------------------------
    //         <- private '_capiserial.rs' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs for C/Rust glue options
    //     - src/capi/capi-map.h for C prototype inputs
    // -----------------------------------------------------------------------
    ";
    let _capiserial
     = bindgen::Builder::default()
        .header("capi/capi-map.h") // Pionix C++ prototype wrapper input
        .raw_line(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("open")
        .allowlist_function("read")
        .allowlist_function("close")
        .allowlist_function("tcgetattr")
        .allowlist_function("tcsetattr")
        .allowlist_function("tcflush")
        .allowlist_function("cfsetispeed")
        .allowlist_function("cfsetospeed")
        .allowlist_var("TIO_.*")
        .allowlist_var("TCF_.*")
        .allowlist_var("TIF_.*")
        .allowlist_var("TTY_O_.*")
        .allowlist_function("__errno_location")
        .allowlist_function("errno")
        .allowlist_function("strerror_r")
        .generate()
        .expect("Unable to generate _capiserial.rs");

    _capiserial
        .write_to_file("capi/_capi-map.rs")
        .expect("Couldn't write _capiserial.rs!");
}
