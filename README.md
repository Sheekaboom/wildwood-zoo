# 🦒 Wildwood Zoo

A zoo-building simulation inspired by **Zoo Tycoon 2**, written in **Rust**, compiled to **WebAssembly**, and played in the browser on desktop or mobile.

Build fenced exhibits, paint terrain and plant foliage to match each species' home biome, adopt 12 species (plus 2 secret ones), hire zookeepers, keep guests fed and entertained, buy more land as you grow, and climb to 5-star fame. Explore it all on foot in first-person **Zoo Walk**, and enter the classic cheat codes from the original *Zoo Tycoon* series in the built-in **terminal**.

> Wildwood Zoo is an original fan-made game. It uses its own name, procedurally drawn art and text, and is not affiliated with or endorsed by the makers of Zoo Tycoon.

## Play

Open **`dist/index.html`** in any modern browser. It's a single self-contained file with the WebAssembly embedded, with no server or install needed.

**Quick start**
1. Pick a zoo size (Small / Medium / Large).
2. 🚧 **Fences:** drag a rectangle beside the path. A gate is added automatically.
3. ⛰️ **Terrain** and 🌳 **Foliage:** match the species' biome. 🛖 **Animal Items:** shelter, toy, water.
4. 🦁 **Animals:** adopt a group with both sexes. 🧑‍🌾 **Staff:** hire a zookeeper.
5. 🍔 **Buildings:** food, drinks, restrooms, benches and trash cans beside paths.
6. 🗺️ **Buy Land** when you run out of room. 🚶 **Zoo Walk** to look around. **>_** for the terminal.

| | Desktop | Phone |
|---|---|---|
| Pan | drag (Select), WASD / arrows | drag (Select), two fingers |
| Zoom | mouse wheel | pinch |
| Rotate | Q / E | 🔄 |
| Terminal | ~ key or >_ | >_ |
| Money cheat | Shift + $ | `money` in the terminal |

## Build

Requires Rust (1.85+ for edition 2024) and Python 3.

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
python3 - <<'PY'
import base64
w = base64.b64encode(open('target/wasm32-unknown-unknown/release/wildwood.wasm','rb').read()).decode()
open('dist/index.html','w').write(open('web/index.html').read().replace('__WASM_B64__', w))
PY
```

`build.sh` does the same, but builds the standard library from source (`-Zbuild-std`). That was needed in the environment where this was developed; it requires the `rust-src` component.

## Test

```sh
node test/suite.mjs dist/wildwood.wasm   # engine: 152 checks incl. fuzzing and save compatibility
python3 test/ui.py                       # browser UI on desktop + phone (pip install playwright; playwright install chromium)
```

## Project layout

```
src/lib.rs     WebAssembly API (C ABI), save/load with RLE + DEFLATE and old-save migration
src/sim.rs     simulation: exhibits, habitat suitability, animals, guests, staff, economy, fame
src/defs.rs    game data: terrain, objects, fences, species, goals
src/cheats.rs  classic cheats and the terminal
src/walk.rs    first-person Zoo Walk raycaster
web/index.html isometric renderer, UI, input, menus
docs/SPEC.md   full specification
```

See **[docs/SPEC.md](docs/SPEC.md)** for the complete design: rules, formulas, data tables, cheats, save format, architecture and test coverage.
