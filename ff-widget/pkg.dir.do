#!/bin/sh

set -eu

wasm-pack build --target=web >&2

if type redo-always >/dev/null 2>&1
then
    redo-always
    sha256sum pkg/* >$3
    redo-stamp <$3
fi
