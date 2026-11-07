#!/bin/sh
# Build Wildwood Zoo: compile Rust -> wasm and inline it into dist/index.html
set -e
cd "$(dirname "$0")"
RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort
mkdir -p dist
cp target/wasm32-unknown-unknown/release/wildwood.wasm dist/
python3 - <<'PY'
import base64
w = base64.b64encode(open('dist/wildwood.wasm','rb').read()).decode()
h = open('web/index.html').read().replace('__WASM_B64__', w)
open('dist/index.html','w').write(h)
print('dist/index.html', len(h)//1024, 'KB')
PY
