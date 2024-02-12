#! /bin/bash
#

set -eux
set -o pipefail

cargo build --release

SELF_DIR="$(dirname $(realpath $0))"

make_example() {
  EXDIR="$SELF_DIR"/"$1"
  shift
  mkdir -p "$EXDIR"
  cargo run --release --bin ff-mandelbrot \
    -- \
    --out-dir "$EXDIR" \
    "$@"
}

make_example circle \
  --x-start=-2 --x-end 2 \
  --y-start=-2 --y-end 2 \
  --width 512 --height 512 \
  --iterations 1,1000,1000000

