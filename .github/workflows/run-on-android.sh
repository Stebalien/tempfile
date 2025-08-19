#!/bin/bash

CMD="$1"
FNAME="/data/local/tmp/$(basename "$CMD")"

shift

adb -s localhost:5555 push "$CMD" "$FNAME"
adb -s localhost:5555 shell "cd /data/local/tmp && chmod 755 $FNAME && $FNAME $*"
