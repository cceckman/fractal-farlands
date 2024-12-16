#!/bin/sh

set -eu

wasm-pack build --target=web >&2

env

if type redo-always 2>&1 >/dev/null
then
    redo-always
    sha256sum pkg/* >$3
    redo-stamp <$3
fi
