#!/usr/bin/env bash
set -euo pipefail

binary="$1"
shift

binary_name="$(basename "$binary")"

if [ "$#" -eq 0 ] && { [ "$binary_name" = "ripeline" ] || [ "$binary_name" = "ripeline.exe" ]; }; then
  exec "$binary" run
fi

exec "$binary" "$@"
