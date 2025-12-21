#!/bin/bash
export BASEDIR=$( readlink -e $( dirname $0 )/..)

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/redpesk/linky-binding-rs/lib:$LD_LIBRARY_PATH"
export LD_LIBRARY_PATH=$BASEDIR/../target/debug

clear
echo "debug with: socat - UDP4-LISTEN:2000"
pkill afb-linky


# start binder with test config
afb-binder -vvv --config=$BASEDIR/etc/binding-linky-network.yaml --port=1236
