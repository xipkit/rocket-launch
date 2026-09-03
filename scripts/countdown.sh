#!/usr/bin/env bash
# Print a terminal count to stdout, optionally archiving the downlink capture.
set -euo pipefail

archive=false
[[ "${1:-}" == "--archive" ]] && archive=true

for t in $(seq 10 -1 1); do
  printf 'T-%02d\n' "$t"
  sleep 1
done
echo "LIFTOFF"

if $archive; then
  stamp=$(date -u +%Y%m%dT%H%M%SZ)
  mkdir -p captures
  tar -czf "captures/downlink-$stamp.tgz" -C /var/kestrel downlink.bin
  echo "archived captures/downlink-$stamp.tgz"
fi
