#!/bin/bash
export BASEDIR=$( readlink -e $( dirname $0 )/..)

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export LD_LIBRARY_PATH=$BASEDIR/../target/debug
clear

echo "debug with: tio /dev/ttyUSB0"

# start binder with test config
afb-binder -vvv --config=binding-linky-serial.yaml --port=1236
