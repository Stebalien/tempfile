#!/system/bin/sh

CMD="$1"
shift

exec "/data/host$CMD" "$@"
