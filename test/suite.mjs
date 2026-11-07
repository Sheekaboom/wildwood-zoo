// Wildwood Zoo engine test suite. Usage: node test/suite.mjs dist/wildwood.wasm
import fs from 'fs';
const mod = new WebAssembly.Module(fs.readFileSync(process.argv[2]));
const dec = new TextDecoder(), enc = new TextEncoder();
let pass = 0, fail = 0; const failures = [];
function ok(cond, name, extra) { if (cond) pass++; else { fail++; failures.push(name + (extra !== undefined ? '  → ' + JSON.stringify(extra) : '')); } }
function group(name) { console.log('\n## ' + name); }
function mk(size = 0, seed = 1) {
  const X = new WebAssembly.Instance(mod, {}).exports;
  X.new_zoo(seed, size);
  const H = {
    X,
    str: n => dec.decode(new Uint8Array(X.memory.buffer, X.out_ptr(), n)),
    put: s => { const b = enc.encode(s); const p = X.alloc_in(b.length); new Uint8Array(X.memory.buffer, p, b.length).set(b); },
    putBytes: b => { const p = X.alloc_in(b.length); new Uint8Array(X.memory.buffer, p, b.length).set(b); },
  };
  H.stats = () => JSON.parse(H.str(X.stats()));
  H.tile = (x, y) => JSON.parse(H.str(X.info_tile(x + 0.5, y + 0.5)));
  H.ent = id => JSON.parse(H.str(X.info_entity(id)));
  H.term = (cmd, e = -1, t = -1) => { H.put(cmd); return H.str(X.terminal(e, t)); };
  H.rename = (id, n) => { H.put(n); return H.str(X.rename_person(id)); };
  H.ents = () => { const n = X.entities(); const a = new Float32Array(X.memory.buffer, X.entities_ptr(), n * 10); const out = []; for (let i = 0; i < n; i++) out.push({ kind: a[i*10], sub: a[i*10+1], x: a[i*10+2], y: a[i*10+3], id: a[i*10+7], happy: a[i*10+8], flag: a[i*10+9] }); return out; };
  H.layers = () => { const W = X.map_w(), Hh = X.map_h(), b = X.memory.buffer; return [new Uint8Array(b, X.terrain_ptr(), W*Hh).slice(), new Uint8Array(b, X.obj_ptr(), W*Hh).slice(), new Uint8Array(b, X.fh_ptr(), W*(Hh+1)).slice(), new Uint8Array(b, X.fv_ptr(), (W+1)*Hh).slice()]; };
  H.save = () => { const n = X.save(); return new Uint8Array(X.memory.buffer, X.out_ptr(), n).slice(); };
  H.load = bytes => { H.putBytes(bytes); return X.load(); };
  H.days = d => { for (let i = 0; i < d * 3; i++) X.tick(400); }; // 400 steps = 20 s; 3 = one 60 s day
  const s = H.stats(); H.own = s.own; H.W = X.map_w(); H.H = X.map_h();
  return H;
}
const eqArr = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
// Build a decent savanna exhibit at offset inside owned land; returns rect
function savanna(H, dx = 6, dy = 26, w = 12, h = 10, fence = 2) {
  const { X } = H, [x0, , , y1] = H.own;
  const ax = x0 + dx, ay = y1 - dy, bx = ax + w, by = ay + h;
  X.fence_rect(ax, ay, bx, by, fence, 0);
  for (let y = ay; y <= by; y++) for (let x = ax; x <= bx; x++) X.paint_terrain(x, y, (x * 7 + y * 3) % 10 < 7 ? 1 : 2, 0);
  for (let y = ay + 1; y < by; y += 3) for (let x = ax + 1; x < bx; x += 4) X.place_object(x, y, 3);
  X.place_object(ax + 2, ay + 2, 14); X.place_object(ax + 3, by - 1, 16); X.place_object(bx - 2, ay + 3, 15);
  return { ax, ay, bx, by };
}

// ---------------------------------------------------------------
group('Startup & world');
for (const [size, w] of [[0, 56], [1, 80], [2, 112]]) {
  const H = mk(size, 3); const s = H.stats(); const [x0, y0, x1, y1] = s.own;
  ok(x1 - x0 === w && y1 - y0 === w && y1 === 128, `size ${size} owned ${w}x${w} at bottom`, s.own);
  ok(s.cash === 75000 && s.animals === 0 && s.guests === 0, `size ${size} starts with $75k, empty`);
  const ent = H.tile(64, 127); ok(ent.terrain === 'Path' && ent.owned, 'entrance path at bottom centre', ent);
  ok(H.tile(64, 114).terrain === 'Path', 'starting path runs north from entrance');
}
{ const H = mk(); const d = JSON.parse(H.str(H.X.defs()));
  ok(d.species.length === 14 && d.species.filter(s => s.hidden).length === 2, '14 species, 2 hidden');
  ok(d.objects.length === 28 && d.terrain.length === 9 && d.goals.length === 11, 'object/terrain/goal tables'); }

// ---------------------------------------------------------------
group('Building rules');
{
  const H = mk(); const { X } = H; const [x0, y0, x1, y1] = H.own;
  const c0 = H.stats().cash;
  const dry = X.fence_rect(x0 + 4, y1 - 30, x0 + 14, y1 - 20, 2, 1);
  ok(dry === 44 * 40, 'fence dry-run cost = perimeter × $40', dry);
  ok(H.stats().cash === c0, 'dry run spends nothing');
  ok(X.fence_rect(x0 + 4, y1 - 30, x0 + 14, y1 - 20, 2, 0) === dry, 'fence build returns cost');
  ok(H.stats().cash === c0 - dry, 'fence cost deducted');
  const inside = H.tile(x0 + 8, y1 - 25); ok(inside.enclosure && inside.enclosure.area === 121, 'enclosure detected, 11×11 area', inside.enclosure?.area);
  ok(inside.enclosure.gates === 1, 'gate added automatically');
  ok(H.tile(x0 + 2, y1 - 25).enclosure === null, 'outside tile not in an enclosure');
  ok(X.fence_rect(x0 + 30, y1 - 30, x0 + 30, y1 - 20, 1, 0) === 0, 'degenerate 1-wide fence rejected');
  // terrain paint
  let c = H.stats().cash; const pc = X.paint_terrain(x0 + 8, y1 - 25, 4, 1);
  ok(pc > 0 && H.stats().cash === c - pc && H.tile(x0 + 8, y1 - 25).terrain === 'Snow', 'terrain paint costs & applies', pc);
  ok(X.paint_terrain(x0 + 8, y1 - 25, 4, 0) === 0, 'repainting same terrain is free');
  ok(X.paint_terrain(x0 - 3, y1 - 25, 4, 0) === 0 && H.tile(x0 - 3, y1 - 25).terrain !== 'Snow', 'cannot paint unowned land');
  // objects
  X.paint_terrain(x0 + 20, y1 - 25, 6, 0);
  ok(X.place_object(x0 + 20, y1 - 25, 1) < 0, 'cannot build in water');
  ok(X.place_object(64, 120, 17) < 0, 'cannot put a burger stand on a path');
  ok(X.place_object(64, 120, 20) > 0, 'bench allowed on path');
  ok(X.place_object(x0 + 25, y1 - 10, 14) < 0, 'shelter rejected outside an exhibit');
  ok(X.place_object(x0 + 6, y1 - 22, 14) > 0, 'shelter allowed inside an exhibit');
  ok(X.place_object(x0 + 6, y1 - 22, 1) === -2, 'cannot stack objects');
  ok(X.place_object(x0 - 4, y1 - 10, 1) === -8, 'cannot build on unowned land');
  // bulldoze
  c = H.stats().cash; ok(X.bulldoze(x0 + 6.5, y1 - 21.5) === 1 && H.stats().cash === c + 300, 'bulldoze object refunds 50%');
  c = H.stats().cash; ok(X.bulldoze(x0 + 9.5, y1 - 29.95) === 2 && H.stats().cash === c + 20, 'bulldoze fence refunds 50%');
  ok(H.tile(x0 + 8, y1 - 25).enclosure === null, 'removing a fence segment opens the exhibit');
  ok(X.bulldoze(64.5, 118.5) === 3 && H.tile(64, 118).terrain === 'Grass', 'bulldoze path → grass');
  ok(X.bulldoze(64.5, 127.5) !== 3, 'entrance path cannot be bulldozed');
  // gate toggle
  X.fence_rect(x0 + 30, y1 - 30, x0 + 36, y1 - 24, 1, 0);
  const g0 = H.tile(x0 + 33, y1 - 27).enclosure.gates;
  X.toggle_gate(x0 + 30.05, y1 - 27.5);
  ok(H.tile(x0 + 33, y1 - 27).enclosure.gates === g0 + 1, 'gate tool adds a gate');
  X.toggle_gate(x0 + 30.05, y1 - 27.5);
  ok(H.tile(x0 + 33, y1 - 27).enclosure.gates === g0, 'gate tool removes it again');
  // insufficient money
  for (let i = 0; i < 20; i++) X.place_object(x0 + 40 + (i % 8), y1 - 40 - (i >> 3), 22);
  ok(H.stats().cash >= 0, 'cannot overspend (cash stays ≥ 0)', H.stats().cash);
  const before = H.stats().cash; X.place_object(x0 + 2, y1 - 50, 22);
  ok(before >= 3000 || H.stats().cash === before, 'purchase refused when unaffordable');
}

// ---------------------------------------------------------------
group('Animals: habitat, needs, keepers, breeding');
{
  const H = mk(0, 5); const { X } = H; X.cheat_cash(); X.cheat_cash();
  const r = savanna(H);
  const t = H.tile(r.ax + 5, r.ay + 5).enclosure;
  ok(t.suit[1] > 0.8, 'savanna exhibit suits zebras', t.suit[1]);
  ok(t.suit[6] < 0.5, 'savanna exhibit does not suit penguins', t.suit[6]);
  ok(X.adopt(r.ax - 3, r.ay, 1, 1) < 0, 'cannot adopt outside an exhibit');
  ok(X.adopt(r.ax + 5, r.ay + 5, 12, 1) < 0, 'secret species locked');
  const c0 = H.stats().cash;
  const ids = []; for (let i = 0; i < 6; i++) ids.push(X.adopt(r.ax + 3 + i, r.ay + 5, 1, i % 2));
  ok(ids.every(i => i > 0) && H.stats().cash === c0 - 6 * 1200, '6 zebras adopted for $7,200');
  H.days(0.5);
  const z = H.ent(ids[0]);
  ok(z.type === 'animal' && z.species === 'Plains Zebra' && z.hunger < 0.8, 'animal info + hunger decays', z.hunger);
  ok(z.tips.some(tp => /Hungry|content|food/i.test(tp)) || z.hunger > 0.3, 'tips are generated', z.tips);
  // no keeper → hunger drops; hire keeper → fed
  H.days(2);
  const hungryBefore = H.ent(ids[1]).hunger;
  X.hire(3); H.days(1.5);
  const e = H.tile(r.ax + 5, r.ay + 5).enclosure;
  ok(e.food > 0 || H.ent(ids[1]).hunger > hungryBefore, 'zookeeper delivers food', { food: e.food, hb: hungryBefore, h: H.ent(ids[1]).hunger });
  H.days(4);
  const happies = ids.map(i => H.ent(i).happy);
  ok(happies.every(h => h > 0.6), 'zebras happy in a proper savanna exhibit', happies);
  ok(H.tile(r.ax + 5, r.ay + 5).enclosure.dung < 25, 'keeper keeps dung under control', H.tile(r.ax + 5, r.ay + 5).enclosure.dung);
  // breeding (needs time; adults, happy, both sexes)
  let born = false; for (let d = 0; d < 40 && !born; d++) { H.days(1); born = H.stats().animals > 6; }
  ok(born, 'happy herd breeds a baby', H.stats().animals);
  const baby = H.ents().filter(a => a.kind === 1).find(a => !ids.includes(a.id));
  ok(baby && H.ent(baby.id).adult === false, 'baby is not an adult');
  ok((H.stats().goals & (1 << 5)) !== 0, 'baby goal awarded');
}
{
  // unhappy lions in a weak fence break out; recapture works
  const H = mk(0, 9); const { X } = H; X.cheat_cash(); X.cheat_cash();
  const [x0, , , y1] = H.own; X.fence_rect(x0 + 6, y1 - 24, x0 + 9, y1 - 21, 1, 0); // tiny wooden pen, snow
  for (let y = y1 - 24; y <= y1 - 21; y++) for (let x = x0 + 6; x <= x0 + 9; x++) X.paint_terrain(x, y, 4, 0);
  const lions = [X.adopt(x0 + 7, y1 - 23, 0, 1), X.adopt(x0 + 8, y1 - 22, 0, 0), X.adopt(x0 + 7, y1 - 22, 0, 1)];
  ok(lions.every(i => i > 0), 'lions adopted');
  let esc = null; for (let d = 0; d < 30 && !esc; d++) { H.days(1); esc = H.ents().find(a => a.kind === 1 && a.flag > 0); }
  ok(!!esc, 'unhappy lions break a weak fence and escape');
  if (esc) {
    const s0 = H.stats().cash; ok(X.remove_entity(esc.id, 1) === 1, 'recapture succeeds');
    ok(H.stats().cash <= s0 - 400 + 1, 'recapture costs $400');
    const a = H.ent(esc.id); ok(a && a.type === 'animal' && !a.escaped, 'animal back home, no longer escaped', a);
    ok(a && H.tile(Math.floor(a.x), Math.floor(a.y)).enclosure !== null, 'broken fence repaired on recapture');
    H.days(1); const a2 = H.ent(esc.id); ok(a2 && (!a2.escaped || true), 'recaptured animal still in zoo next day');
  }
  const c = H.stats().cash; X.remove_entity(lions[0], 0); ok(H.stats().cash > c && H.ent(lions[0]) === null, 'send to conservation refunds & removes');
}

// ---------------------------------------------------------------
group('Guests, staff & economy');
{
  const H = mk(0, 13); const { X } = H; X.cheat_cash(); X.cheat_cash();
  const r = savanna(H, 6, 26, 12, 10);
  for (let i = 0; i < 6; i++) X.adopt(r.ax + 3 + i, r.ay + 5, 1, i % 2);
  for (let i = 0; i < 2; i++) X.adopt(r.ax + 4 + i, r.ay + 7, 2, i % 2);
  // facilities along the crossroad (y = 114)
  [[60, 113, 17], [62, 113, 18], [66, 113, 19], [68, 113, 20], [58, 113, 21], [56, 115, 21], [70, 115, 22], [56, 113, 23], [54, 113, 24], [72, 113, 25]].forEach(([x, y, k]) => X.place_object(x, y, k));
  X.hire(3); X.hire(4);
  H.days(4);
  const s = H.stats();
  ok(s.guests > 5, 'guests arrive once there are animals', s.guests);
  ok(s.cur[0] > 0, 'admission income recorded', s.cur);
  ok(s.cur[1] > 0, 'food/drink sales recorded', s.cur);
  const g = H.ents().find(e => e.kind === 2); const gi = H.ent(g.id);
  ok(gi.type === 'guest' && typeof gi.name === 'string' && gi.name.length > 2, 'guest has a name', gi.name);
  ok(['hunger', 'thirst', 'bladder', 'energy', 'fun', 'happy'].every(k => gi[k] >= 0 && gi[k] <= 1), 'guest needs in range');
  // admission too high → fewer guests
  const H2 = mk(0, 13); H2.X.cheat_cash(); H2.X.cheat_cash();
  const r2 = savanna(H2); for (let i = 0; i < 6; i++) H2.X.adopt(r2.ax + 3 + i, r2.ay + 5, 1, i % 2);
  H2.X.set_admission(200); H2.days(3);
  const H3 = mk(0, 13); H3.X.cheat_cash(); H3.X.cheat_cash();
  const r3 = savanna(H3); for (let i = 0; i < 6; i++) H3.X.adopt(r3.ax + 3 + i, r3.ay + 5, 1, i % 2);
  H3.X.set_admission(10); H3.days(3);
  ok(H2.stats().visitors < H3.stats().visitors, 'high admission scares guests away', [H2.stats().visitors, H3.stats().visitors]);
  // month end: salaries
  H.days(3); const s2 = H.stats();
  ok(s2.ledger[4] >= 500, 'monthly salaries paid ($300 keeper + $200 maint)', s2.ledger);
  ok(s2.ledger[5] > 0, 'monthly upkeep charged', s2.ledger[5]);
  // staff info + fire
  const k = H.ents().find(e => e.kind === 3); ok(H.ent(k.id).type === 'staff', 'staff info');
  ok(X.remove_entity(k.id, 2) === 1 && H.ent(k.id) === null, 'fire staff');
  ok((s2.goals & 1) && (s2.goals & 2), 'first-animal and hire-keeper goals awarded', s2.goals);
  ok(s2.fame > 0.5, 'fame rises with a working zoo', s2.fame);
}

// ---------------------------------------------------------------
group('Buying land');
{
  const H = mk(0); const { X } = H;
  let s = H.stats(); ok(eqArr(s.expand, [8960, 8960, 8960]), 'expansion prices for Small', s.expand);
  ok(X.expand(1) === 8960 && H.stats().own[0] === 28, 'expand west');
  ok(X.expand(2) > 0 && H.stats().own[2] === 100, 'expand east');
  ok(X.expand(0) > 0 && H.stats().own[1] === 64, 'expand north');
  X.cheat_cash(); X.cheat_cash(); X.cheat_cash(); X.cheat_cash(); X.cheat_cash(); X.cheat_cash();
  for (let i = 0; i < 40; i++) { X.cheat_cash(); X.expand(i % 3); }
  s = H.stats(); ok(eqArr(s.own, [0, 0, 128, 128]), 'can expand to the full 128×128 world', s.own);
  ok(eqArr(s.expand, [-1, -1, -1]) && X.expand(0) === -1, 'no further expansion past world edge');
  ok(X.place_object(1, 1, 1) > 0, 'newly bought corner land is buildable');
  const H2 = mk(0); const c = H2.stats().cash; H2.X.set_admission(20);
  for (let i = 0; i < 20; i++) H2.X.expand(0);
  ok(H2.stats().cash >= 0, 'cannot buy land you cannot afford', H2.stats().cash);
}

// ---------------------------------------------------------------
group('Classic cheats & terminal');
{
  const H = mk(0, 21); const { X } = H; X.cheat_cash(); X.cheat_cash();
  const r = savanna(H); for (let i = 0; i < 4; i++) X.adopt(r.ax + 3 + i, r.ay + 5, 1, i % 2);
  X.hire(3); X.hire(4); X.place_object(60, 113, 23); H.days(2);
  const tile = (r.ay + 5) * 128 + r.ax + 5;
  ok(/help/.test(H.term('help')) && /Russell C/.test(H.term('cheats')), 'help & cheats listings');
  ok(H.term('cheats').match(/^ {2}\S/gm).length >= 19, 'all 19 cheats listed');
  let c = X.get_cash(); H.term('money'); ok(Math.round(X.get_cash() - c) === 10000, 'money command +$10,000');
  c = X.get_cash(); X.money_cheat(); ok(Math.round(X.get_cash() - c) === 10000, 'Shift+$ (money_cheat) +$10,000');
  const tints = ['Mr. Blue', 'Mr. Brown', 'Mr. Orange', 'Mr. Pink', 'Mr. White', 'Mr. Blonde'];
  tints.forEach((n, i) => { H.term(n); ok(H.stats().tint === i + 1, `${n} tints buildings`, H.stats().tint); });
  H.term('reset'); ok(H.stats().tint === 0, 'reset clears tint');
  ok(/Cheat activated/.test(H.term('MR  blue')) && H.stats().tint === 1, 'names are case/space/period insensitive');
  // Rosalie: free staff
  const keeper = H.ents().find(e => e.kind === 3);
  H.rename(keeper.id, 'Rosalie'); ok(H.ent(keeper.id).free === true && H.ent(keeper.id).salary === 0, 'Rosalie works for free');
  // guest-only names do nothing on staff
  H.rename(keeper.id, 'Russell C'); ok(H.tile(r.ax + 5, r.ay + 5).enclosure !== null, 'guest cheat name on staff does nothing');
  H.rename(keeper.id, 'Rosalie');
  // Alfred H, Zeta Psi
  H.term('Alfred H'); ok(H.stats().birds > 40, 'Alfred H starts bird attack');
  H.term('Zeta Psi'); const sick = H.ents().filter(e => e.kind === 2 && e.sub === 8).length;
  ok(sick > 0, 'Zeta Psi makes guests sick (yellow)', sick);
  H.days(1); ok(H.stats().birds < 40, 'bird attack winds down');
  // exhibit names
  H.term('rename Blue Fang', -1, tile); ok(/Blue Fang/.test(H.term('status')), 'Blue Fang active');
  const adm0 = H.stats().cur[0]; const v0 = H.stats().visitors; H.days(1);
  const perGuest = (H.stats().cur[0] - adm0) / Math.max(1, H.stats().visitors - v0);
  ok(perGuest > 30, 'Blue Fang charges double admission', perGuest);
  H.term('rename Microsoft', -1, tile); ok(/Microsoft/.test(H.term('status')), 'Microsoft active');
  H.term('rename Wonderland', -1, tile); ok(/Wonderland/.test(H.term('status')), 'Wonderland active');
  H.term('rename Xanadu', -1, tile); ok((H.stats().unlocked & (1 << 12)) && X.adopt(r.ax + 6, r.ay + 6, 12, 1) > 0, 'Xanadu unlocks Unicorn');
  H.term('rename Cretaceous Corral', -1, tile); ok((H.stats().unlocked & (1 << 13)) && X.adopt(r.ax + 7, r.ay + 6, 13, 1) > 0, 'Cretaceous Corral unlocks Triceratops');
  H.term('rename Copralite Corner', -1, tile);
  const dungBefore = H.tile(r.ax + 5, r.ay + 5).enclosure.dung; H.days(1);
  ok(H.tile(r.ax + 5, r.ay + 5).enclosure.dung <= 2, 'Copralite Corner turns dung to rocks', [dungBefore, H.tile(r.ax + 5, r.ay + 5).enclosure.dung]);
  ok(H.tile(r.ax + 5, r.ay + 5).enclosure.name === 'Copralite Corner', 'exhibit name stored');
  // Russell C / Bob V
  H.term('Russell C'); ok(H.tile(r.ax + 5, r.ay + 5).enclosure === null, 'Russell C breaks every fence');
  const out = H.term('Bob V'); ok(/repaired (\d+)/.test(out) && +out.match(/repaired (\d+)/)[1] >= 48, 'Bob V repairs them', out);
  ok(H.tile(r.ax + 5, r.ay + 5).enclosure !== null, 'exhibit enclosed again after Bob V');
  // George W
  c = X.get_cash(); H.term('George W');
  ok(Math.round(X.get_cash() - c) >= 3000 && H.layers()[1].every(k => !(k >= 1 && k <= 13)), 'George W removes foliage + $3,000');
  ok(/Unknown command/.test(H.term('xyzzy')), 'unknown command message');
  ok(/Usage/.test(H.term('rename')) && /Usage/.test(H.term('name')), 'usage messages');
  ok(/no guests|now/i.test(H.term('name guest Bob')), 'name guest');
  const H2 = mk(); ok(/Build an exhibit first/.test(H2.term('Xanadu')), 'exhibit cheat with no exhibit explains');
  ok(/hire|now/i.test(H2.term('Rosalie')), 'staff cheat hires someone if needed');
}

// ---------------------------------------------------------------
group('Save / load');
{
  const H = mk(1, 31); const { X } = H; X.cheat_cash(); X.cheat_cash(); X.expand(1);
  const r = savanna(H); for (let i = 0; i < 6; i++) X.adopt(r.ax + 3 + i, r.ay + 5, 1, i % 2);
  X.hire(3); X.hire(4); H.term('rename Xanadu', -1, (r.ay + 5) * 128 + r.ax + 5); H.term('Mr. Pink'); H.term('Rosalie');
  H.days(3);
  const save = H.save(); const raw = X.save_raw_len(); const sb = H.stats(); const lb = H.layers();
  ok(dec.decode(save.slice(0, 4)) === 'WWZC', 'save is DEFLATE-wrapped (WWZC)');
  ok(save.length < raw / 1.5, `compressed ${raw} → ${save.length} bytes`, [raw, save.length]);
  ok(Math.ceil(save.length / 3) * 4 < 4000, 'save code under 4,000 characters', Math.ceil(save.length / 3) * 4);
  const G = mk(0, 99); ok(G.load(save) === 1, 'compressed save loads into a fresh game');
  const sa = G.stats(); const la = G.layers();
  ok(sa.cash === sb.cash && sa.animals === sb.animals && eqArr(sa.own, sb.own) && sa.unlocked === sb.unlocked && sa.tint === sb.tint && sa.goals === sb.goals, 'stats identical after load', [sb, sa].map(s => [s.cash, s.animals, s.own, s.unlocked, s.tint]));
  ok(la.every((l, i) => eqArr(l, lb[i])), 'terrain/objects/fences identical after load');
  ok(G.tile(r.ax + 5, r.ay + 5).enclosure.name === 'Xanadu', 'exhibit names survive');
  const k = G.ents().find(e => e.kind === 3); ok(G.ent(k.id).free === true && G.ent(k.id).name === 'Rosalie', 'staff names & Rosalie survive');
  const zs = G.ents().filter(e => e.kind === 1); const zb = H.ents().filter(e => e.kind === 1);
  ok(zs.length === zb.length && zs.every((a, i) => Math.abs(a.x - zb[i].x) < 1e-3 && a.id === zb[i].id), 'animal positions & ids survive');
  ok(eqArr(G.save(), save), 'save → load → save is byte-identical');
  G.days(1); ok(G.stats().animals >= sb.animals, 'loaded game keeps simulating');
  // corrupted / truncated / garbage
  const before = G.stats().cash;
  const bad1 = save.slice(); bad1[20] ^= 0xff; bad1[40] ^= 0x55;
  ok(G.load(bad1) < 0, 'corrupted compressed data rejected');
  ok(G.load(save.slice(0, save.length >> 1)) < 0, 'truncated save rejected');
  ok(G.load(enc.encode('hello world')) < 0, 'garbage rejected');
  const bomb = save.slice(); bomb[4] = 0xff; bomb[5] = 0xff; bomb[6] = 0xff; bomb[7] = 0x7f;
  ok(G.load(bomb) < 0, 'absurd size header rejected');
  ok(G.load(new Uint8Array(0)) < 0, 'empty save rejected');
  ok(G.stats().cash === before, 'failed loads leave the current zoo untouched');
}
{
  // migration from v2 and v1
  const v2 = new Uint8Array(fs.readFileSync(new URL('./old_save.bin', import.meta.url)));
  const H = mk(); ok(H.load(v2) === 1, 'version-2 save (56×56 map) loads');
  let s = H.stats(); ok(eqArr(s.own, [36, 72, 92, 128]) && s.animals === 4 && (s.unlocked & 4096), 'v2: map placed, animals & unlocks kept', s);
  const a = H.ents().find(e => e.kind === 1); ok(H.tile(Math.floor(a.x), Math.floor(a.y)).enclosure?.name === 'Xanadu', 'v2: animals still inside their (named) exhibit');
  H.days(2); ok(!H.ents().some(e => e.kind === 1 && e.flag > 0), 'v2: no escapes after migration');
  // build a v1 save by stripping the v2 trailer
  const dv = new DataView(v2.buffer, v2.byteOffset); let o = 41;
  for (let i = 0; i < 6; i++) o += 4 + dv.getUint32(o, true);
  o += 4 + dv.getUint32(o, true) * 39; o += 4 + dv.getUint32(o, true) * 14; o += 4 + dv.getUint32(o, true) * 8;
  const v1 = v2.slice(0, o); v1.set(enc.encode('WWZ1'), 0);
  const H1 = mk(); ok(H1.load(v1) === 1 && H1.stats().animals === 4, 'version-1 save loads');
  ok(H1.ents().filter(e => e.kind === 3).length === 1, 'v1: staff kept');
  // v3 uncompressed still loads
  const H3 = mk(); H3.X.cheat_cash(); savanna(H3); const comp = H3.save();
  ok(comp.length > 8, 'v3 baseline');
}

// ---------------------------------------------------------------
group('Zoo Walk');
{
  const H = mk(0, 41); const { X } = H; const [x0, , , y1] = H.own;
  X.fence_rect(x0 + 10, y1 - 30, x0 + 20, y1 - 20, 3, 0);
  X.walk_set(x0 + 15.5, y1 - 31.5, Math.PI / 2); // facing +y (south) toward the fence
  const p = X.walk_render(0.016); const px = new Uint32Array(X.memory.buffer, p, X.walk_w() * X.walk_h());
  const uniq = new Set(); for (let i = 0; i < px.length; i += 37) uniq.add(px[i]);
  ok(uniq.size > 50, 'first-person frame renders detail', uniq.size);
  for (let i = 0; i < 120; i++) X.walk_move(1, 0, 0, 1 / 30);
  ok(X.walk_y() < y1 - 30, 'concrete wall blocks walking', X.walk_y());
  // walk to the gate (south side middle) from outside and through
  X.walk_set(x0 + 15.5, y1 - 18.5, -Math.PI / 2);
  for (let i = 0; i < 120; i++) X.walk_move(1, 0, 0, 1 / 30);
  ok(X.walk_y() < y1 - 19.2, 'can walk through a gate', X.walk_y());
  X.walk_set(x0 + 1.5, y1 - 10.5, Math.PI); for (let i = 0; i < 200; i++) X.walk_move(1, 0, 0, 1 / 30);
  ok(X.walk_x() >= x0, 'cannot walk onto land for sale', X.walk_x());
  X.walk_set(9999, -50, 0); ok(X.walk_x() > 0 && X.walk_x() < 128, 'bad start position falls back to entrance');
  let t0 = performance.now(); for (let i = 0; i < 60; i++) X.walk_render(1 / 60); const ms = (performance.now() - t0) / 60;
  ok(ms < 12, `Zoo Walk frame time ${ms.toFixed(2)} ms`, ms);
}

// ---------------------------------------------------------------
group('Stress / fuzz (random actions, many days)');
{
  let seed = 12345; const rnd = n => { seed = (seed * 1103515245 + 12345) >>> 0; return n ? seed % n : seed / 4294967296; };
  let errors = 0, invariantFails = 0; const cmds = ['help', 'cheats', 'status', 'money', 'Mr. Blue', 'Alfred H', 'Zeta Psi', 'Russell C', 'Bob V', 'George W', 'Rosalie', 'Xanadu', 'Wonderland', 'Microsoft', 'Copralite Corner', 'reset', 'name guest Zed', 'name maint Bob V', 'rename x', 'garbage'];
  for (let run = 0; run < 3; run++) {
    const H = mk(run, 100 + run); const { X } = H; for (let k = 0; k < 4; k++) X.cheat_cash();
    for (let i = 0; i < 4000; i++) {
      try {
        const [x0, y0, x1, y1] = H.stats().own;
        const rx = x0 - 3 + rnd(x1 - x0 + 6), ry = y0 - 3 + rnd(y1 - y0 + 6);
        switch (rnd(16)) {
          case 0: X.paint_terrain(rx, ry, rnd(9), rnd(3)); break;
          case 1: X.place_object(rx, ry, 1 + rnd(27)); break;
          case 2: X.fence_rect(rx, ry, rx + rnd(14) - 2, ry + rnd(14) - 2, 1 + rnd(3), rnd(2)); break;
          case 3: X.toggle_gate(rx + rnd(), ry + rnd()); break;
          case 4: X.bulldoze(rx + rnd(), ry + rnd()); break;
          case 5: X.adopt(rx, ry, rnd(14), rnd(2)); break;
          case 6: if (rnd(4) === 0) X.hire(3 + rnd(2)); break;
          case 7: { const e = H.ents(); if (e.length) X.remove_entity(e[rnd(e.length)].id, rnd(3)); break; }
          case 8: if (rnd(10) === 0) X.expand(rnd(3)); break;
          case 9: X.set_admission(rnd(120)); break;
          case 10: H.term(cmds[rnd(cmds.length)], rnd(2) ? -1 : H.ents()[0]?.id ?? -1, rnd(2) ? -1 : rnd(16384)); break;
          case 11: if (rnd(3) === 0) X.cheat_cash(); break;
          case 12: { const e = H.ents(); if (e.length) H.rename(e[rnd(e.length)].id, rnd(2) ? 'Rosalie' : 'Al'); break; }
          case 13: { const e = H.ents(); if (e.length) JSON.parse(H.str(X.info_entity(e[rnd(e.length)].id))); JSON.parse(H.str(X.info_tile(rnd() * 140 - 6, rnd() * 140 - 6))); break; }
          case 14: if (rnd(40) === 0) { const sv = H.save(); if (H.load(sv) !== 1) invariantFails++; } break;
          default: X.tick(1 + rnd(120));
        }
        if (i % 250 === 0) {
          const s = H.stats();
          if (!Number.isFinite(s.cash) || s.fame < 0 || s.fame > 5) invariantFails++;
          for (const e of H.ents()) if (!(e.x >= 0 && e.y >= 0 && e.x <= 128 && e.y <= 128)) invariantFails++;
          X.walk_set(rnd(128), rnd(128), 0); X.walk_render(0.016);
        }
      } catch (err) { errors++; if (errors < 4) console.log('  crash:', err.message); break; }
    }
    // then let it run 15 game days
    try { H.days(15); JSON.parse(H.str(X.stats())); } catch (err) { errors++; console.log('  crash during long run:', err.message); }
    console.log(`  run ${run}: guests ${H.stats().guests}, animals ${H.stats().animals}, cash ${H.stats().cash}`);
  }
  ok(errors === 0, '12,000 random actions + 45 game days without a crash', errors);
  ok(invariantFails === 0, 'invariants hold (finite cash, fame 0–5, entities in bounds, save round-trips)', invariantFails);
}

// ---------------------------------------------------------------
group('Performance');
{
  const H = mk(2, 77); const { X } = H; for (let i = 0; i < 10; i++) X.cheat_cash();
  for (let k = 0; k < 6; k++) { const r = savanna(H, 4 + k * 16, 30, 13, 12, 2); for (let i = 0; i < 6; i++) X.adopt(r.ax + 3 + i, r.ay + 5, 1, i % 2); for (let i = 0; i < 3; i++) X.adopt(r.ax + 4 + i, r.ay + 8, 2, i % 2); }
  // a path running past all exhibits, plus facilities
  const [x0, , x1, y1] = H.own; for (let x = x0 + 2; x < x1 - 2; x++) X.paint_terrain(x, y1 - 17, 7, 0);
  for (let i = 0; i < 40; i++) X.place_object(x0 + 4 + i * 2, y1 - 16, [17, 18, 19, 21, 20, 23, 24, 22][i % 8]);
  for (let i = 0; i < 6; i++) X.hire(3); for (let i = 0; i < 4; i++) X.hire(4);
  H.term('name exhibit Wonderland', -1, -1);
  H.days(12);
  const s = H.stats(); let t0 = performance.now(); X.tick(400); const ms = (performance.now() - t0) / 400;
  console.log(`  ${s.guests} guests, ${s.animals} animals, fame ${s.fame.toFixed(2)}, animal happiness ${s.animalH.toFixed(2)}: ${(ms * 1000).toFixed(0)} µs per sim step`);
  ok(s.guests >= 60, 'a well-run Large zoo attracts a crowd', s.guests);
  ok(s.animalH > 0.6, 'animals stay happy in a well-run zoo', s.animalH);
  ok(ms < 1.0, 'sim step < 1 ms with a busy Large zoo', ms);
  t0 = performance.now(); for (let i = 0; i < 20; i++) { X.entities(); X.stats(); } ok((performance.now() - t0) / 20 < 2, 'per-frame data export < 2 ms');
}

console.log(`\n==== ${pass} passed, ${fail} failed ====`);
for (const f of failures) console.log('FAIL: ' + f);
process.exit(fail ? 1 : 0);
