#!/bin/bash

FNAME="/data/local/tmp/exe-${RANDOM}"

CMD="$1"
shift

adb -s localhost:5555 push "$CMD" "$FNAME"
adb -s localhost:5555 shell chmod 755 "$FNAME"
adb -s localhost:5555 shell -e none "$FNAME" "$@"
