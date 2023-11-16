#!/bin/bash

# Attempt to remote debug with LLDB and gdb-server (still not working!!!)

if test $# -ne 5; then
    echo "ERROR: syntaxe $0 TARGET_ADDR HOST_BINDING_PATH TARGET_BINDING_CONF GDBPORT TARGET_USER"
    exit 2
fi

TARGET_ADDR="$1"
BINDING_PATH="$2"
BINDING_CONF="$3"
GDBPORT="$4"
TARGET_USER="$5"

REMOTEDBG="gdbserver"
TARGETBIN="afb-binder"
TARGETCWD=`dirname $BINDING_CONF`

if ! test -f $BINDING_PATH; then
    echo "ERROR: binding path not found $BINDING_PATH"
    exit 2
fi

# If binary rebuilt let's re-split debug info and update target
if test $BINDING_PATH -nt $BINDING_PATH.debug; then
  rm -f $BINDING_PATH.debug
  objcopy --only-keep-debug $BINDING_PATH $BINDING_PATH.debug
  objcopy --add-gnu-debuglink=$BINDING_PATH.debug $BINDING_PATH
  strip --strip-debug $BINDING_PATH
  if ! test -f $BINDING_PATH.debug; then
    echo "ERROR: Fail to generate debug info $BINDING_PATH.debug"
    exit 2
  fi
fi #end binary rebuilt

# install target (replace with scp if needed)
ssh ${TARGET_USER}@${TARGET_ADDR} "mkdir -p ${TARGETCWD}"
rsync --progress "${BINDING_PATH}" "${TARGET_USER}@${TARGET_ADDR}:${TARGETCWD}"

# start debugger
ssh -q "${TARGET_USER}@${TARGET_ADDR}" >/dev/null << EOF
  cd ${TARGETCWD}
  killall -q ${REMOTEDBG} ${TARGET_BIN}
  echo  starting ${REMOTEDBG} *:${GDBPORT} afb-binder --config=${APP_CONFIG}
  export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
  rm -rf /tmp/gdb-server.out
  ${REMOTEDBG} --debug *:${GDBPORT} afb-binder  --trap-faults=0 -vvv --config=`basename ${BINDING_CONF}` >/tmp/gdb-server.out 2>&1 &
EOF