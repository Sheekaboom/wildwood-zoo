# Wildwood Zoo — Specification

Version 1.0 · November 2026

Wildwood Zoo is a zoo-building simulation inspired by the gameplay of *Zoo Tycoon 2* (2004). The whole simulation is written in Rust and compiled to WebAssembly; a single self-contained HTML page draws the game and handles input. It runs in any modern browser on desktop and mobile with no server and no install.

This document describes what the game does and how it is built. It was written from the design conversation that produced the game, and every number below matches the code at the time of writing.

---

## 1. Goals and non-goals

### Goals
- **Play like Zoo Tycoon 2.** Build exhibits with fences, shape each habitat (terrain, foliage, shelters, toys, water) to suit the species living there, keep animals and guests happy, and grow the zoo's fame.
- **Rust + WebAssembly core.** All game rules, simulation, saving and the first-person renderer live in Rust. JavaScript only draws the isometric view, runs the UI and forwards input.
- **One-file delivery.** The build produces one `index.html` with the wasm embedded (base64), so the game can be published as a single web page or opened from disk.
- **Desktop and mobile.** Mouse, keyboard and touch all fully supported; layout adapts to narrow screens.
- **Retro 2000s feel.** Beveled green-and-gold panels, chunky buttons, a pixelated first-person mode and a green-on-black command terminal.
- **Classic cheats.** The well-known secret names and key combo from the original *Zoo Tycoon* series, plus a terminal to enter them.

### Non-goals
- **No copyrighted material.** The game uses its own name, art (all procedurally drawn), text and animal names. It does not use the *Zoo Tycoon* name, logo, assets or audio.
- **No true 3D.** The main view is 2D isometric with four rotations; first-person mode is a software raycaster.
- **Not in v1:** terrain height (hills), marine shows and tanks, photo mode, sound, multiplayer, cloud saves.

---

## 2. Platform and constraints

| Area | Requirement |
|---|---|
| Browsers | Current Chrome, Safari, Firefox, Edge (desktop and mobile). Needs WebAssembly and Canvas 2D. |
| Hosting | Runs inside a sandboxed iframe (for example a claude.ai artifact). Therefore: no `alert`/`confirm`/`prompt` (all dialogs are in-page), no downloads, no network requests. |
| Storage | `localStorage` may be missing or throw. Every access is wrapped in `try/catch`; the game must boot and play without it, and the Export/Import save code is the portable fallback. |
| Size | Single HTML file, about 500 KB (wasm about 360 KB before base64). |
| Performance | 60 fps target. One simulation step must take well under 1 ms (measured: about 31 µs with 107 guests and 67 animals). |

---

## 3. The world

### 3.1 Map and land ownership
- The world is a **128 × 128** tile grid. Only the **owned rectangle** is playable; everything else is *land for sale*, drawn darker with a dashed gold border and "FOR SALE" signs.
- The zoo entrance is fixed at the bottom-centre tile (64, 127). A starting path runs 14 tiles north from it and meets a 25-tile east–west crossroad.
- Starting sizes (chosen on the welcome screen or via **New Zoo**):

| Size | Owned area | Position |
|---|---|---|
| Small | 56 × 56 | bottom-centre |
| Medium | 80 × 80 | bottom-centre |
| Large | 112 × 112 | bottom-centre |

- **Buying land:** the Buy Land tool expands the owned rectangle by an **8-tile strip** to the north, west or east (the south edge is the world edge). Price: **$20 per tile** added (an 8-tile strip beside a Small zoo costs $8,960). Expansion stops at the world edge; the zoo can grow to the full 128 × 128.
- Land for sale is generated with natural scenery (trees, shrubs, boulders, ferns, ponds, dirt patches), so bought land arrives with features.
- Nothing can be painted, built, fenced or walked on outside the owned area (guests, staff, animals and the Zoo Walk player all respect the boundary).

### 3.2 Terrain
Painted with a brush (S = 1 tile, M = radius 1, L = radius 2). Cost is charged per changed tile; repainting the same terrain is free. Path and water cannot be painted over objects; the entrance tile can't be changed.

| Id | Terrain | Cost / tile | Notes |
|---|---|---|---|
| 0 | Grass | $5 | |
| 1 | Savanna Grass | $6 | |
| 2 | Dirt | $4 | |
| 3 | Sand | $5 | |
| 4 | Snow | $8 | |
| 5 | Rock | $8 | |
| 6 | Fresh Water | $15 | Only swimming species can enter; provides drinking water |
| 7 | Path | $10 | Guests walk only on paths |
| 8 | Rainforest Floor | $7 | |

### 3.3 Objects
One object per tile. Bulldozing refunds 50%. "Blocks" = nothing can walk through it. Animal items can only go inside an exhibit; blocking buildings can't go on paths; nothing can go in water.

| Category | Objects (cost) |
|---|---|
| Foliage (biome tag) | Oak $60 (temperate), Pine $60 (boreal), Acacia $80 (savanna), Palm $80 (tropical/desert), Bamboo $50 (Asian), Saguaro Cactus $40 (desert), Shrub $30 (most biomes), Kapok $90 (tropical/Asian), Boulder $40 (neutral), Elephant Grass $20 (savanna/Asian), Tree Fern $25 (tropical/temperate), Ice Rock $50 (tundra), Birch $60 (temperate/boreal) |
| Animal items | Animal Shelter $600 (+$5/mo), Enrichment Toy $150, Water Trough $200 (+$2/mo) |
| Guest buildings | Burger Stand $1,500 (+$40/mo), Soda Stand $1,000 (+$30/mo), Restroom $1,200 (+$30/mo), Bench $120, Trash Can $80, Gift Shop $3,000 (+$60/mo), Donation Box $200 |
| Scenery | Flower Bed $60, Fountain $900 (+$5/mo), Lamp Post $100 |

Scenery and foliage have a **beauty** value that spreads to tiles within 2 and raises guest happiness nearby.

### 3.4 Fences, gates and exhibits
- Fences sit on tile **edges**. The Fences tool builds a rectangle of segments (minimum 2 × 2 tiles); the dry-run cost shows while dragging.

| Fence | Cost / segment | Strength |
|---|---|---|
| Wooden Fence | $25 | 1 |
| Chain-Link Fence | $40 | 2 |
| Concrete Wall | $75 | 3 |

- A **gate** is added automatically, on a segment facing an adjacent path if there is one, otherwise in the middle of the south side. The Gate tool ($50) toggles a gate on any segment.
- **Exhibits** are found by flood fill: any area closed off by fences that isn't connected to the entrance is an exhibit. Changing fences recalculates exhibits, and each exhibit keeps its food store and name across recalculations.
- Exhibit stats (refreshed every game second and immediately after any edit, even while paused): area, terrain mix, object counts, water (water tile or trough), shelters, toys, dung, weakest fence strength, gates, species counts and happiness, and attraction.
- Exhibits can be **renamed** (some names are cheats, see §7).

---

## 4. Animals

### 4.1 Species
12 public species plus 2 secret ones unlocked by cheats. Terrain preferences are the ideal fractions of the exhibit's area.

| Species | Cost | Biome | Ideal terrain | Foliage | Space/animal | Group | Fence | Swims | Predator | Fame |
|---|---|---|---|---|---|---|---|---|---|---|
| Lion | $3,000 | Savanna | 60% savanna, 30% dirt, 10% grass | savanna, 6% | 12 | 2–6 | 2 | | ✓ | 8 |
| Plains Zebra | $1,200 | Savanna | 70% savanna, 20% grass, 10% dirt | savanna, 5% | 10 | 3–10 | 1 | | | 4 |
| Reticulated Giraffe | $2,500 | Savanna | 60% savanna, 30% dirt, 10% grass | savanna, 8% | 16 | 2–8 | 1 | | | 6 |
| African Elephant | $6,000 | Savanna | 40% savanna, 40% dirt, 10% water, 10% grass | savanna, 5% | 28 | 2–6 | 3 | | | 10 |
| Grizzly Bear | $3,500 | Temperate Forest | 45% grass, 20% dirt, 20% rock, 15% water | temperate/boreal, 15% | 15 | 1–3 | 2 | ✓ | ✓ | 8 |
| Giant Panda | $8,000 | Bamboo Forest | 60% grass, 20% dirt, 20% rock | Asian, 25% | 12 | 1–2 | 1 | | | 12 |
| Emperor Penguin | $900 | Tundra | 60% snow, 30% water, 10% rock | tundra, none needed | 3 | 6–20 | 1 | ✓ | | 5 |
| Polar Bear | $5,000 | Tundra | 60% snow, 25% water, 15% rock | tundra, 3% | 18 | 1–3 | 3 | ✓ | ✓ | 10 |
| Bengal Tiger | $5,500 | Tropical Forest | 40% grass, 30% rainforest, 20% dirt, 10% water | Asian/tropical, 20% | 15 | 1–2 | 3 | ✓ | ✓ | 11 |
| Western Gorilla | $6,000 | Tropical Rainforest | 70% rainforest, 20% grass, 10% dirt | tropical, 30% | 14 | 3–8 | 2 | | | 10 |
| Dromedary Camel | $1,500 | Desert | 80% sand, 20% dirt | desert, 4% | 12 | 2–6 | 1 | | | 4 |
| Hippopotamus | $4,000 | Savanna Wetland | 40% water, 30% dirt, 30% grass | savanna/tropical, 5% | 18 | 2–8 | 2 | ✓ | | 7 |
| *Unicorn* (secret) | $10,000 | Enchanted Forest | 70% grass, 20% water, 10% dirt | temperate, 15% | 14 | 1–4 | 2 | | | 16 |
| *Triceratops* (secret) | $12,000 | Cretaceous Forest | 40% grass, 40% dirt, 20% rock | tropical/temperate, 12% | 30 | 2–6 | 3 | | | 15 |

Animals can only be adopted onto a tile inside an exhibit that the species can stand on. Each gets a name and sex (the player picks ♂/♀). Adults are adopted; babies take 720 s (12 game days) to grow up.

### 4.2 Habitat suitability
For animal *a* of species *s* in exhibit *e*, each factor is 0–1:

| Factor | Rule |
|---|---|
| Terrain | `1 − ½ · Σ |ideal_i − actual_i|` over all terrain types |
| Foliage | matching foliage count ÷ (area × density), capped at 1, minus a penalty of up to 0.6 for foliage from the wrong biome. Species with density 0 only take the penalty. |
| Space | area ÷ Σ(space needed by every animal in the exhibit), capped at 1 |
| Social | inside the group range → 1; below → 0.3–1 rising with group size; above → max ÷ count |
| Shelter | 1 with a shelter, otherwise 0.4 |
| Enrichment | 1 with a toy, otherwise 0.5 |
| Cleanliness | `1 − dung ÷ (area × 0.12)` |
| Water | 1 if the exhibit has a water tile or trough, otherwise 0 |
| Safety | 0.2 for a non-predator sharing an exhibit with a predator, otherwise 1 |

```
base   = 0.28·terrain + 0.18·foliage + 0.14·space + 0.14·social
       + 0.08·shelter + 0.06·toys + 0.06·clean + 0.06·water
biome  = 0.6·terrain + 0.4·foliage
total  = min(1, base · (0.35 + 0.65·biome) · 1.12) · (0.4 + 0.6·safety)
```
The biome multiplier means the wrong habitat can't be made up for with toys and shelters (in a well-built savanna exhibit, zebras score about 87% and penguins about 37%).

**Happiness** moves 6% per second toward `total × (0.3 + 0.7 · min(1, 2 · min(hunger, thirst))) × 1.08`, clamped to 0–1.

The animal info panel shows every factor as a bar plus plain-language tips ("Wants more Savanna Grass terrain (70% ideal)", "Lonely. Prefers a group of 3–10", "Fence too weak. Needs Concrete Wall", "This exhibit has no gate…"). While placing an animal on desktop, a tooltip shows the target exhibit's suitability for that species; the exhibit panel lists the best-suited species on any device.

### 4.3 Needs and behaviour
- Hunger empties in about 160 s and thirst in about 120 s. Below 50% hunger an animal eats one unit from the exhibit's food store; below 55% thirst it drinks if the exhibit has water.
- Animals wander tile to tile with pauses, rest, play with toys, and avoid water unless they swim. Each produces dung about every 70 s.
- **Breeding:** an adult female with happiness > 0.7, an adult male of the same species in the exhibit, group below max + 2, and space ≥ 0.7 → 1-in-150 chance per second; cooldown 1,500 s.
- **Escapes:** when an animal tries to cross a fence it may break the segment: +4% chance if happiness < 0.25, plus 1.5% per point the fence is weaker than the species needs (doubled if happiness < 0.5). The broken segment is recorded. An animal outside every exhibit is *escaped*: it's flagged ❗, guests within 5 tiles panic and may leave.
- **Recapture** ($400, from the animal panel): keepers first repair every fence segment escaped animals broke, then the animal is returned to its home exhibit. If that exhibit no longer exists, the animal is sent to another zoo (30% refund).
- **Send to conservation:** removes the animal and refunds 50% (adults) or 70% (babies).

---

## 5. People

### 5.1 Guests
- **Arrival rate per second:** `(0.12 + 0.22·fame) · price_factor` when the zoo has animals (0.03 otherwise). `price_factor = clamp(2 − admission ÷ fair_price, 0, 1.3)`, where `fair_price = 12 + 8·fame`. Guests in the zoo are capped at `60 + 70·fame`.
- Each guest arrives with $40–$140 and a random name, and pays admission on entry. Guests feel ripped off if admission is more than 1.3 × the fair price.
- **Needs** (1 = satisfied): hunger (empties in about 110 s), thirst (90 s), restroom (130 s), energy (200 s), fun (120 s). When a need falls below its threshold (about 0.3) the guest pathfinds (BFS over path tiles) to the nearest matching building beside a path:

| Building | Effect | Price |
|---|---|---|
| Burger Stand | hunger → full, carries trash | $6 |
| Soda Stand | thirst → full, carries trash | $3 |
| Restroom | restroom → full | free |
| Bench | +60% energy (8 s sit) | free |
| Gift Shop | +25% fun (if cash > $30) | $12 |

- Trash is dropped in a trash can within 2 tiles, or becomes litter on the path after 25 s.
- **Viewing exhibits:** a path tile within 2 tiles of an exhibit lets guests stop and watch for 3–6 s. Fun gained: `min(0.55, 0.12 + 0.22·attraction)` the first time, 35% of that on repeat views (one view per exhibit per 70 s). Attraction = Σ over species of `fame · (0.25 + 0.75·happiness) · √min(count, 5) ÷ 10`. On a first view, a donation box within 3 tiles collects `1 + 3 · attraction · happiness` (up to 30% of the guest's cash).
- **Happiness** moves toward `0.45·avg(needs) + 0.45·fun + 0.1 + environment`, where environment adds beauty, −0.15 for litter, and −0.4 near an escaped animal.
- Guests leave when their visit (140–300 s) ends, happiness < 0.15, energy < 0.1, or cash runs out. They show recent **thoughts** in their info panel (24 messages plus 2 cheat-only ones).

### 5.2 Staff
| Role | Hire | Salary | Job |
|---|---|---|---|
| Zookeeper | $500 | $300/month | Picks the exhibit with the greatest need (food below half of 3 per animal, or dung > 2), walks in through a gate, cleans dung tile by tile, refills food to `6 × animals + 4`, then leaves. Exhibits without a gate can't be serviced. |
| Maintenance Worker | $300 | $200/month | Finds the nearest litter, walks to it and sweeps it up; patrols paths when idle. |

Staff can be renamed and fired from their panel.

---

## 6. Economy, fame and goals

- Start: **$75,000**, admission $20 (adjustable $0–$80 in the Finances window), fame 0.5 stars.
- **Time:** 1 game day = 60 s at 1× speed; 1 month = 6 days; 12 months per year. Speeds: pause, 1×, 2×, 4×.
- **Month end:** pays salaries (staff named *Rosalie* are free), building upkeep, and animal upkeep (2% of each animal's price), then posts a monthly report. The Finances window shows this month and last month: admissions, concessions, donations, gift shop, salaries, upkeep, construction, animals and profit.
- **Fame (0–5 stars)**, updated every second:
  `rating = 0.25·guest_happiness + 0.30·animal_happiness·min(1, animals/15) + 0.45·min(1, species/10)` (× 0.3 with no animals). Fame moves toward `min(5, 5.5·rating)`, rising at 0.4% and falling at 0.8% of the gap per second. One species alone levels off around 1 star; variety is what earns stars.
- **Goals** (reward paid once):

| Goal | Reward |
|---|---|
| Adopt your first animal | $1,000 |
| Hire a zookeeper | $500 |
| Have 40 guests in the zoo at once | $2,000 |
| Exhibit 3 different species | $3,000 |
| Reach 2-star zoo fame | $3,000 |
| Have a baby animal born | $5,000 |
| Own 20 animals | $5,000 |
| Reach 3.5-star zoo fame | $10,000 |
| Exhibit 7 different species | $10,000 |
| Reach 5-star zoo fame | $20,000 |
| Have $150,000 in the bank | trophy |

---

## 7. Classic cheats and the terminal

*Zoo Tycoon 2* shipped without cheat codes or a console. These are the well-known cheats from the original *Zoo Tycoon* (2001) and its *Dinosaur Digs* expansion, brought into this game, plus a new terminal for entering them.

### 7.1 Key combo
- **Shift + $** (Shift + 4): +$10,000. As in the original, each use has a 15% chance to wear through a random fence segment. Ignored while typing in a text field.

### 7.2 Secret names
Names match ignoring case, extra spaces, periods and apostrophes (`mr blue` = `Mr. Blue`). A name only works on the right kind of target.

| Name | Give it to | Effect | Origin |
|---|---|---|---|
| Rosalie | any staff | That staff member works for free | Zoo Tycoon |
| Mr. Blue / Brown / Orange / Pink / White / Blonde | a guest | Clothes and buildings turn that color (blonde = yellow) | Zoo Tycoon |
| Russell C | a guest | Every fence in the zoo breaks | Zoo Tycoon |
| Alfred H | a guest | Birds attack guests for 45 s (scared thoughts, −happiness) | Zoo Tycoon |
| Zeta Psi | a guest | ~35% of guests turn yellow, get sick and vomit litter | Zoo Tycoon |
| Blue Fang | an exhibit | Guests pay double admission without noticing | Zoo Tycoon |
| Microsoft | an exhibit | Donations doubled | Zoo Tycoon |
| Wonderland | an exhibit | +60% guest arrivals, +40% guest cap | Zoo Tycoon |
| Xanadu | an exhibit | Unlocks the Unicorn (permanent) | Dinosaur Digs |
| Cretaceous Corral | an exhibit | Unlocks the Triceratops (permanent) | Dinosaur Digs |
| Copralite Corner | an exhibit | Dung in that exhibit turns into boulders | Dinosaur Digs |
| Bob V | a maintenance worker | Repairs every broken fence on record | Dinosaur Digs |
| George W | a maintenance worker | Removes all foliage and pays $3,000 | Dinosaur Digs |

Left out because they don't apply here: research and program unlocks (nothing is locked that way) and the expansion cheats that target children or women (guests have no age or gender).

### 7.3 Terminal
Opened with the **>_** button (top bar), the **~ / `** key, or Menu → Terminal & Cheats. Styled as a green-on-black CRT with scanlines; supports ↑/↓ history, Esc to close.

| Command | Action |
|---|---|
| `help` | List commands |
| `cheats` | List every secret name with its effect and origin |
| `money` | +$10,000 (same as Shift + $; useful on phones) |
| `rename <name>` | Rename the selected guest, staff member or exhibit |
| `name guest\|staff\|maint\|exhibit <name>` | Name a random guest / a staff member (hires one if needed) / the selected exhibit (else the first one) |
| `<secret name>` | Shortcut: applies the cheat to a suitable target automatically |
| `status` | Date, zoo size, cash, fame, guests, animals, staff, broken fences, active exhibit cheats |
| `reset` | Undo tint, birds and sickness |
| `clear`, `exit` | Clear the screen / close |

---

## 8. Zoo Walk (first person)

- Entered from the toolbar (starts on the path nearest the camera) or from any animal, guest or tile panel ("Walk here", "See through their eyes").
- A software raycaster written in Rust renders into an RGBA buffer that JavaScript scales up pixelated. Resolution follows the screen's shape: 400 px wide in landscape and 300 px wide in portrait, with height matching the aspect ratio (120–720 px). Square pixels, 66° horizontal FOV.
- Draws a sky with clouds and a distant treeline, textured ground per terrain (animated water, dung, litter) with distance fog, fences as part-height walls (wood rails, see-through chain-link mesh, concrete blocks, gates), buildings as solid boxes, and billboards for every foliage and scenery object, animal and person. Sprites are depth-sorted per column against walls.
- Controls: W/S or ↑/↓ to move, A/D to strafe, ←/→ or Q/E to turn, **hold Shift to run** (2.2×), drag to look. On phones, an on-screen d-pad.
- Collision: blocked by fences (except gates), blocking objects, water and land for sale.

---

## 9. Saving

### 9.1 Where saves go
- **Browser save:** Menu → Save Zoo writes to `localStorage` (`wildwood_save`); the game also **autosaves every minute** to the same single slot and resumes automatically on the next visit. If storage is blocked, the game says so and suggests Export.
- **Save code:** Menu → Export save code gives a base64 text (about **2,200 characters** for a typical zoo) with a Copy button; Import save code restores it. Works across devices and for sharing.
- What's saved: cash, fame, time, admission, goals, visitor count, owned land, terrain, objects, fences, dung, litter, animals (all stats), staff (names, free flag), exhibit food and names, unlocked species, tint, and broken-fence records. Guests are not saved; new ones arrive after loading.

### 9.2 Format
```
"WWZC" | u32 raw_length | DEFLATE( v3 payload )        ← current
v3 payload: "WWZ3" | header | owned rect (4×u32)
            | 6 layers RLE-encoded (terrain, objects, h-fences, v-fences, dung, litter)
            | animals | staff | exhibit food | unlocks, tint | staff names | exhibit names | broken fences
```
- Layers use (count, value) run-length pairs, then the whole payload is DEFLATE-compressed (miniz_oxide, level 10). A typical v3 payload is about 3.9–4.3 KB raw and about 1.6–1.9 KB compressed.
- **Backward compatibility:** uncompressed `WWZ3`, `WWZ2` (cheats era, 56 × 56 map) and `WWZ1` (first release) all load. Old 56 × 56 maps are placed at the bottom-centre of the 128 × 128 world as a Small zoo, with every coordinate remapped.
- **Safety:** invalid magic, a raw length over 8 MB, failed decompression, truncated data or wrong layer sizes are rejected with a negative code, and the current zoo is left untouched. Save → load → save is byte-identical.

---

## 10. User interface

### 10.1 Layout
- **Top bar:** logo, cash, fame stars (partial fill), guests, animals, date, speed buttons, terminal (>_), menu (☰). Clicking cash, stars, guests/animals or date opens Finances, Zoo Status and Goals. On phones it fits in two rows (logo hidden, compact buttons).
- **Toolbar (bottom):** Select, Fences, Animals, Terrain, Foliage, Animal Items, Buildings, Scenery, Paths, Staff, Bulldoze, Buy Land, Zoo Walk, Rotate. Each build category opens a tray of item cards with a drawn thumbnail, name, price and biome tags.
- **Info panel:** right side on desktop, a bottom sheet on phones. Shows the animal, guest, staff, exhibit or tile under selection with live bars, tips, thoughts and actions (✏️ rename, send to conservation, recapture, fire, center, walk here).
- **Toasts:** messages, goal awards (gold) and alerts (red); at most 4 visible.
- **Menu:** Zoo Status, Finances & Admission, Goals, Save, Load, Export, Import, Terminal & Cheats, How to Play, New Zoo (with size choice).
- **Welcome screen:** on first visit, explains the game and offers Small / Medium / Large.

### 10.2 Controls
| | Desktop | Phone / tablet |
|---|---|---|
| Pan | drag with Select, right/middle-drag, WASD/arrows | drag with Select; two fingers with any tool |
| Zoom | mouse wheel (toward cursor) | pinch |
| Rotate | Q / E or 🔄 | 🔄 |
| Build | click, or drag (terrain, paths, foliage, fence rectangles) | tap or drag |
| Pause | Space | ❚❚ |
| Cancel | Esc (back to Select) | Select |

Isometric view: 64 × 32 px tiles at zoom 1 (zoom range 0.22–2.5); drawing order by rotated depth. Previews: green/red placement diamonds, brush footprint, fence rectangle with live cost, red for land for sale.

### 10.3 Visual style
All art is procedural canvas drawing: terrain tiles with per-tile variation and detail strokes, trees and buildings as iso shapes, animals as side-view figures per species with walk cycles, people with shirt colors and staff uniforms. Cheat tints recolor stands, shelters and clothes; Alfred H draws flapping birds over guests.

---

## 11. Architecture

```
wildwood/
├── Cargo.toml          # cdylib + rlib, edition 2024, opt-level 3, LTO, panic=abort
├── build.sh            # cargo build (wasm32, build-std) → embed wasm into dist/index.html
├── src/
│   ├── lib.rs          # C ABI exported to JS, save/load (RLE + DEFLATE, migration)
│   ├── defs.rs         # static data: terrain, objects, fences, species, names, thoughts, goals
│   ├── sim.rs          # game state & simulation: regions, suitability, animals, guests, staff, economy
│   ├── cheats.rs       # Shift+$, secret names, terminal command parser
│   └── walk.rs         # Zoo Walk raycaster
├── web/index.html      # UI: CSS, isometric renderer, input, menus, terminal (__WASM_B64__ placeholder)
├── dist/index.html     # built single-file game
├── docs/SPEC.md        # this document
└── test/               # suite.mjs (engine), ui.py (browser), old_save.bin (v2 fixture)
```

### 11.1 Rust ↔ JavaScript interface
- Plain `extern "C"` exports, no wasm-bindgen and no imports: the module instantiates with an empty import object.
- **Shared memory views** (re-created each frame): terrain, objects, horizontal and vertical fence edges (`W×(H+1)` and `(W+1)×H`; low 2 bits = strength, bit 2 = gate), dung, litter, region ids.
- **Entity buffer:** 10 × f32 per entity: `[kind (1 animal, 2 guest, 3 keeper, 4 maintenance), species/variant, x, y, facing, growth, action/walking, id, happiness, escaped/state]`.
- **JSON out** via a shared output buffer (`out_ptr` + returned length): `stats`, `defs`, `info_entity`, `info_tile`, terminal output, save bytes. **Strings/bytes in** via `alloc_in(n)`, then the call (`rename_person`, `rename_exhibit`, `terminal`, `load`).
- Commands: `new_zoo`, `tick(steps)`, `paint_terrain`, `place_object`, `fence_rect` (with dry run), `toggle_gate`, `bulldoze`, `adopt`, `hire`, `remove_entity` (sell/recapture/fire), `set_admission`, `expand`, `money_cheat`, `pick`, the `walk_*` functions and `save`/`load`.
- **Game loop:** JS accumulates real time × speed into fixed 0.05 s simulation steps (at most 40 per frame), then draws.

### 11.2 Build
- Target `wasm32-unknown-unknown`. The only dependency is `miniz_oxide` (pure-Rust DEFLATE). No JS libraries.
- `./build.sh` runs `cargo build --release --target wasm32-unknown-unknown` and inlines the wasm into `dist/index.html`.
- The original environment couldn't download the prebuilt wasm standard library, so `build.sh` builds it from source (`RUSTC_BOOTSTRAP=1 … -Zbuild-std=std,panic_abort`, which needs the `rust-src` component). On a normal setup you can instead run `rustup target add wasm32-unknown-unknown` and drop those flags.

---

## 12. Testing

| Suite | Run | Coverage | Result (v1.0) |
|---|---|---|---|
| Engine | `node test/suite.mjs dist/wildwood.wasm` | World setup; building rules and costs; habitat suitability; hunger, keepers, dung; breeding; escapes and recapture; guests, spending and admission pricing; salaries and upkeep; goals; land purchase and limits; all 19 cheats and every terminal command; save round-trip (byte-identical), v1/v2 migration, corrupted/truncated/garbage/oversized saves; Zoo Walk rendering and collision; **fuzz:** 12,000 random actions + 45 game days × 3 runs with invariant checks; performance | 152 / 152 |
| UI | `python3 test/ui.py` (Playwright + Chrome) | Real mouse/keyboard/touch on a 1366 × 820 desktop and a 390 × 844 phone: welcome size picker, every toolbar tray, building by drag and click, adopting with the suitability tooltip, hiring, panning, speed, rename dialogs firing cheats, the terminal (focus, history, Esc, ~, `$` not triggering inside it), Shift + $, rotation round-trips, wheel zoom, all menus, admission slider, Buy Land, save + page reload, export → new zoo → import, bad import code rejected, Zoo Walk (walk, run, turn, render, portrait frame, no black rows), blocked storage, phone layout (no overflow, two-row top bar), two-finger pan, pinch zoom, d-pad, frame rate, no JS errors | 88 / 88, stable over 6 runs |

### Bugs found and fixed during validation
1. Recapturing an animal whose own fence it broke sent it to another zoo; keepers now repair the fence first.
2. Exhibit stats were stale while paused after painting terrain.
3. Wrong-biome habitats scored too high (penguin in savanna: 57%); added the biome multiplier.
4. Zoo Walk was cropped on portrait phones; the render size now follows the screen's shape.
5. A black line ran across the Zoo Walk horizon.
6. `setPointerCapture` exceptions could drop touches.
7. The internal ID counter changed after loading; an invalid Zoo Walk start position wasn't redirected to the entrance.

---

## 13. Known limitations and future ideas
- All testing ran in headless Chrome with phone emulation, not on real devices.
- One browser save slot, which the autosave overwrites.
- No terrain height, marine animals, photo mode, sound or day/night cycle.
- Guests are not saved.
- The suitability tooltip needs hover (desktop only); phones use the exhibit panel's "Best suited for" list.
- Ideas: multiple save slots, hills, an aquarium expansion, research unlocks, scenarios with win conditions, a minimap, sound effects.

---

## 14. History (from the design conversation)
1. **First request:** a Rust/WebAssembly zoo tycoon game with a web UI, in the early-2000s style → changed mid-build to "closer to Zoo Tycoon 2": isometric rotating view, habitat needs, keepers, guest needs, fame, first-person Zoo Walk. Original name and art.
2. **Classic cheats + terminal:** research showed ZT2 had no official cheats or console, so the original *Zoo Tycoon* / *Dinosaur Digs* cheats were added with a new terminal, renaming, and the Unicorn and Triceratops.
3. **Sharing, device support and saves:** confirmed desktop + mobile support; documented browser saves and save codes.
4. **Bigger zoos:** chose "both": a starting size (S/M/L) plus buying land; 128 × 128 world; RLE save compression (25,000 → 5,200 characters).
5. **Compression + validation:** DEFLATE added (5,200 → 2,200 characters); full engine and UI test suites; the bugs in §12 fixed.
6. **This spec** and the GitHub repository.
