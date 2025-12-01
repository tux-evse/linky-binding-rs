#!/bin/bash
cd `dirname $0/../..`

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

echo "debug with: socat - UDP4-LISTEN:2000"

# start binder with test config
afb-binder -vvv --config=afb-binding/etc/binding-linky-network.yaml --port=1236
