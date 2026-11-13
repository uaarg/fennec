#!/usr/bin/env bash
# Cargo runner for Renode. Cargo passes the ELF path as $1.
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "usage: $0 <path-to-elf>" >&2
    exit 1
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
elf="$(realpath "$1")"

exec renode --console \
    -e "\$repl=@${script_dir}/discovery.repl; \$bin=@${elf}; include @${script_dir}/blinky.resc"
