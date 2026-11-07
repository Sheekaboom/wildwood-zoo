//! Wildwood Zoo — a zoo-building simulation in Rust, compiled to WebAssembly.
//! The exported C ABI below is consumed by the JavaScript front end.

pub mod cheats;
pub mod defs;
pub mod sim;
pub mod walk;

use defs::*;
use sim::*;

struct State {
    game: Game,
    walk: walk::Walk,
    ents: Vec<f32>,
    out: Vec<u8>,
    inbuf: Vec<u8>,
}

static mut STATE: Option<State> = None;

fn st() -> &'static mut State {
    unsafe {
        let p = &raw mut STATE;
        if (*p).is_none() {
            *p = Some(State { game: Game::new(0x5eed_1234), walk: walk::Walk::new(), ents: vec![], out: vec![], inbuf: vec![] });
        }
        (*p).as_mut().unwrap()
    }
}

fn out(s: String) -> u32 {
    let st = st();
    st.out = s.into_bytes();
    st.out.len() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn init(seed: u32) {
    let s = st();
    s.game = Game::new(seed as u64 * 2654435761 + 7);
    s.walk = walk::Walk::new();
}

#[unsafe(no_mangle)]
pub extern "C" fn new_zoo(seed: u32, size: u32) {
    let s = st();
    s.game = Game::new_sized(seed as u64 * 2654435761 + 7, size.min(2) as u8);
    s.walk = walk::Walk::new();
}
#[unsafe(no_mangle)] pub extern "C" fn expand(dir: u32) -> i32 { st().game.expand(dir as u8) }

#[unsafe(no_mangle)]
pub extern "C" fn tick(steps: u32) {
    let g = &mut st().game;
    for _ in 0..steps.min(400) {
        g.step();
    }
}

#[unsafe(no_mangle)] pub extern "C" fn map_w() -> u32 { W as u32 }
#[unsafe(no_mangle)] pub extern "C" fn map_h() -> u32 { H as u32 }
#[unsafe(no_mangle)] pub extern "C" fn terrain_ptr() -> *const u8 { st().game.terrain.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn obj_ptr() -> *const u8 { st().game.obj.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn fh_ptr() -> *const u8 { st().game.fh.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn fv_ptr() -> *const u8 { st().game.fv.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn dung_ptr() -> *const u8 { st().game.dung.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn litter_ptr() -> *const u8 { st().game.litter.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn region_ptr() -> *const u16 { st().game.region.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn outside_region() -> u32 { st().game.outside as u32 }

#[unsafe(no_mangle)]
pub extern "C" fn entities() -> u32 {
    let s = st();
    s.game.fill_entities(&mut s.ents);
    (s.ents.len() / 10) as u32
}
#[unsafe(no_mangle)] pub extern "C" fn entities_ptr() -> *const f32 { st().ents.as_ptr() }
#[unsafe(no_mangle)] pub extern "C" fn out_ptr() -> *const u8 { st().out.as_ptr() }

#[unsafe(no_mangle)] pub extern "C" fn stats() -> u32 { let j = st().game.stats_json(); out(j) }
#[unsafe(no_mangle)] pub extern "C" fn defs() -> u32 { let j = st().game.defs_json(); out(j) }
#[unsafe(no_mangle)] pub extern "C" fn info_entity(id: u32) -> u32 { let j = st().game.info_entity(id); out(j) }
#[unsafe(no_mangle)] pub extern "C" fn info_tile(fx: f32, fy: f32) -> u32 { let j = st().game.info_tile(fx, fy); out(j) }
#[unsafe(no_mangle)] pub extern "C" fn pick(fx: f32, fy: f32) -> i32 { st().game.pick(fx, fy).map(|v| v as i32).unwrap_or(-1) }

#[unsafe(no_mangle)] pub extern "C" fn paint_terrain(x: i32, y: i32, t: u32, r: i32) -> i32 { st().game.paint_terrain(x, y, t as u8, r) }
#[unsafe(no_mangle)] pub extern "C" fn place_object(x: i32, y: i32, k: u32) -> i32 { st().game.place_object(x, y, k as u8) }
#[unsafe(no_mangle)] pub extern "C" fn fence_rect(x0: i32, y0: i32, x1: i32, y1: i32, kind: u32, dry: u32) -> i32 { st().game.fence_rect(x0, y0, x1, y1, kind as u8, dry != 0) }
#[unsafe(no_mangle)] pub extern "C" fn toggle_gate(fx: f32, fy: f32) -> i32 { st().game.toggle_gate(fx, fy) }
#[unsafe(no_mangle)] pub extern "C" fn bulldoze(fx: f32, fy: f32) -> i32 { st().game.bulldoze(fx, fy) }
#[unsafe(no_mangle)] pub extern "C" fn adopt(x: i32, y: i32, sp: u32, male: u32) -> i32 { st().game.adopt(x, y, sp as u8, male != 0) }
#[unsafe(no_mangle)] pub extern "C" fn hire(kind: u32) -> i32 { st().game.hire(kind as u8) }
#[unsafe(no_mangle)] pub extern "C" fn remove_entity(id: u32, mode: u32) -> i32 { st().game.remove_entity(id, mode as u8) }
#[unsafe(no_mangle)] pub extern "C" fn set_admission(v: f32) { st().game.admission = v.clamp(0.0, 200.0); }
#[unsafe(no_mangle)] pub extern "C" fn get_cash() -> f64 { st().game.cash }
#[unsafe(no_mangle)] pub extern "C" fn cheat_cash() { let g = &mut st().game; g.cash += 50000.0; g.msg("Received a $50,000 conservation grant!".into()); }

// ---- Cheats & terminal (string argument passed via alloc_in) ----
fn take_in() -> String {
    let s = st();
    String::from_utf8_lossy(&std::mem::take(&mut s.inbuf)).into_owned()
}
#[unsafe(no_mangle)] pub extern "C" fn money_cheat() { st().game.money_cheat(); }
#[unsafe(no_mangle)] pub extern "C" fn rename_person(id: u32) -> u32 { let n = take_in(); let r = st().game.rename_person(id, &n); out(r) }
#[unsafe(no_mangle)] pub extern "C" fn rename_exhibit(tile: u32) -> u32 { let n = take_in(); let r = st().game.rename_exhibit(tile as usize, &n); out(r) }
#[unsafe(no_mangle)] pub extern "C" fn terminal(sel_ent: i32, sel_tile: i32) -> u32 { let n = take_in(); let r = st().game.terminal(&n, sel_ent, sel_tile); out(r) }

// ---- Zoo Walk ----
#[unsafe(no_mangle)] pub extern "C" fn walk_w() -> u32 { st().walk.w as u32 }
#[unsafe(no_mangle)] pub extern "C" fn walk_h() -> u32 { st().walk.h as u32 }
#[unsafe(no_mangle)] pub extern "C" fn walk_resize(w: u32, h: u32) { st().walk.resize(w as usize, h as usize); }
#[unsafe(no_mangle)]
pub extern "C" fn walk_set(x: f32, y: f32, a: f32) {
    let s = st();
    let t = s.game.tile_of(x, y);
    let inside = x.is_finite() && y.is_finite() && x >= 0.0 && y >= 0.0 && x < W as f32 && y < H as f32;
    if inside && s.game.owned(t) && s.game.terrain[t] != T_WATER && !s.game.blocks(t) {
        s.walk.x = x;
        s.walk.y = y;
    } else {
        let e = s.game.entrance as usize;
        s.walk.x = (e % W) as f32 + 0.5;
        s.walk.y = (e / W) as f32 + 0.3;
    }
    s.walk.a = a;
}
#[unsafe(no_mangle)]
pub extern "C" fn walk_move(fwd: f32, strafe: f32, turn: f32, dt: f32) {
    let s = st();
    s.walk.movement(&s.game, fwd, strafe, turn, dt);
}
#[unsafe(no_mangle)]
pub extern "C" fn walk_render(dt: f32) -> *const u32 {
    let s = st();
    s.walk.render(&s.game, dt);
    s.walk.buf.as_ptr()
}
#[unsafe(no_mangle)] pub extern "C" fn walk_x() -> f32 { st().walk.x }
#[unsafe(no_mangle)] pub extern "C" fn walk_y() -> f32 { st().walk.y }
#[unsafe(no_mangle)] pub extern "C" fn walk_a() -> f32 { st().walk.a }

// ---- Save / Load ----

struct Wr(Vec<u8>);
impl Wr {
    fn u8(&mut self, v: u8) { self.0.push(v) }
    fn u32(&mut self, v: u32) { self.0.extend_from_slice(&v.to_le_bytes()) }
    fn f32(&mut self, v: f32) { self.0.extend_from_slice(&v.to_le_bytes()) }
    fn f64(&mut self, v: f64) { self.0.extend_from_slice(&v.to_le_bytes()) }
    fn bytes(&mut self, v: &[u8]) { self.u32(v.len() as u32); self.0.extend_from_slice(v) }
}
struct Rd<'a>(&'a [u8], usize, bool);
impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        if self.1 + n > self.0.len() { self.2 = false; return &[]; }
        let s = &self.0[self.1..self.1 + n];
        self.1 += n;
        s
    }
    fn u8(&mut self) -> u8 { self.take(1).first().copied().unwrap_or(0) }
    fn u32(&mut self) -> u32 { let b = self.take(4); if b.len() == 4 { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else { 0 } }
    fn f32(&mut self) -> f32 { f32::from_bits(self.u32()) }
    fn f64(&mut self) -> f64 { let b = self.take(8); if b.len() == 8 { let mut a = [0u8; 8]; a.copy_from_slice(b); f64::from_le_bytes(a) } else { 0.0 } }
    fn bytes(&mut self) -> Vec<u8> { let n = self.u32() as usize; self.take(n).to_vec() }
}

fn rle(v: &[u8]) -> Vec<u8> {
    let mut o = Vec::with_capacity(v.len() / 8);
    let mut i = 0;
    while i < v.len() {
        let b = v[i];
        let mut n = 1;
        while i + n < v.len() && v[i + n] == b && n < 255 {
            n += 1;
        }
        o.push(n as u8);
        o.push(b);
        i += n;
    }
    o
}
fn unrle(v: &[u8], len: usize) -> Option<Vec<u8>> {
    let mut o = Vec::with_capacity(len);
    for c in v.chunks(2) {
        if c.len() < 2 {
            return None;
        }
        for _ in 0..c[0] {
            o.push(c[1]);
        }
    }
    if o.len() == len { Some(o) } else { None }
}

#[unsafe(no_mangle)]
pub extern "C" fn save() -> u32 {
    let s = st();
    let g = &s.game;
    let mut w = Wr(Vec::with_capacity(16 * 1024));
    w.0.extend_from_slice(b"WWZ3");
    w.f64(g.cash);
    w.f32(g.fame);
    w.f64(g.time);
    w.f32(g.admission);
    w.u32(g.goals_done);
    w.u8(g.baby_born as u8);
    w.u32(g.next_id);
    w.u32(g.total_visitors);
    for v in g.own {
        w.u32(v as u32);
    }
    for layer in [&g.terrain, &g.obj, &g.fh, &g.fv, &g.dung, &g.litter] {
        w.bytes(&rle(layer));
    }
    w.u32(g.animals.len() as u32);
    for a in g.animals.iter() {
        w.u32(a.id);
        w.u8(a.sp);
        w.f32(a.x);
        w.f32(a.y);
        w.u8(a.male as u8);
        w.f32(a.age);
        w.f32(a.hunger);
        w.f32(a.thirst);
        w.f32(a.happy);
        w.f32(a.breed_cd);
        w.u32(a.home);
        w.u8(a.name as u8);
    }
    let staff: Vec<&Person> = g.people.iter().filter(|p| p.kind != 2).collect();
    w.u32(staff.len() as u32);
    for p in staff.iter() {
        w.u32(p.id);
        w.u8(p.kind);
        w.f32(p.x);
        w.f32(p.y);
        w.u8(p.variant);
    }
    let foods: Vec<(u32, f32)> = g.encs.iter().filter(|e| !e.outside && e.first_tile != u32::MAX).map(|e| (e.first_tile, e.food)).collect();
    w.u32(foods.len() as u32);
    for (t, f) in foods {
        w.u32(t);
        w.f32(f);
    }
    w.u32(g.unlocked);
    w.u8(g.tint);
    for p in staff {
        w.bytes(p.name.as_bytes());
        w.u8(p.free as u8);
    }
    let named: Vec<(u32, &str)> = g.encs.iter().filter(|e| !e.outside && !e.name.is_empty()).map(|e| (e.first_tile, e.name.as_str())).collect();
    w.u32(named.len() as u32);
    for (t, n) in named {
        w.u32(t);
        w.bytes(n.as_bytes());
    }
    w.u32(g.broken.len() as u32);
    for &(t, d, v) in g.broken.iter() {
        w.u32(t);
        w.u8(d);
        w.u8(v);
    }
    // Wrap the v3 payload in DEFLATE: "WWZC" + raw length + compressed bytes.
    let packed = miniz_oxide::deflate::compress_to_vec(&w.0, 10);
    let mut o = Vec::with_capacity(packed.len() + 8);
    o.extend_from_slice(b"WWZC");
    o.extend_from_slice(&(w.0.len() as u32).to_le_bytes());
    o.extend_from_slice(&packed);
    s.out = o;
    s.out.len() as u32
}

/// Size of the save before DEFLATE (for diagnostics).
#[unsafe(no_mangle)]
pub extern "C" fn save_raw_len() -> u32 {
    let s = st();
    if s.out.len() >= 8 && &s.out[..4] == b"WWZC" { u32::from_le_bytes([s.out[4], s.out[5], s.out[6], s.out[7]]) } else { s.out.len() as u32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_in(n: u32) -> *mut u8 {
    let s = st();
    s.inbuf = vec![0; n as usize];
    s.inbuf.as_mut_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn load() -> i32 {
    let s = st();
    let mut data = std::mem::take(&mut s.inbuf);
    if data.len() >= 8 && &data[..4] == b"WWZC" {
        let raw_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        if raw_len > 8 << 20 {
            return -4;
        }
        match miniz_oxide::inflate::decompress_to_vec_with_limit(&data[8..], raw_len) {
            Ok(v) if v.len() == raw_len => data = v,
            _ => return -4,
        }
    }
    let mut r = Rd(&data, 0, true);
    let ver = match r.take(4) {
        b"WWZ1" => 1,
        b"WWZ2" => 2,
        b"WWZ3" => 3,
        _ => return -1,
    };
    // Older saves used a 56x56 map; they are placed at the bottom-centre of the new world.
    let (ox, oy) = if ver < 3 { ((W - OLD_W) / 2, H - OLD_H) } else { (0, 0) };
    let remap = |t: u32| -> u32 {
        if ver >= 3 {
            t
        } else {
            let (x, y) = (t as usize % OLD_W, t as usize / OLD_W);
            ((y + oy) * W + x + ox) as u32
        }
    };
    let mut g = Game::new(99);
    g.messages.clear();
    g.cash = r.f64();
    g.fame = r.f32();
    g.time = r.f64();
    g.admission = r.f32();
    g.goals_done = r.u32();
    g.baby_born = r.u8() != 0;
    let saved_next_id = r.u32();
    g.total_visitors = r.u32();
    if ver >= 3 {
        let mut o = [0usize; 4];
        for v in o.iter_mut() {
            *v = r.u32() as usize;
        }
        if o[0] >= o[2] || o[1] >= o[3] || o[2] > W || o[3] > H {
            return -2;
        }
        g.own = o;
        let lens = [N, N, g.fh.len(), g.fv.len(), N, N];
        let mut layers = vec![];
        for len in lens {
            match unrle(&r.bytes(), len) {
                Some(l) => layers.push(l),
                None => return -2,
            }
        }
        g.litter = layers.pop().unwrap();
        g.dung = layers.pop().unwrap();
        g.fv = layers.pop().unwrap();
        g.fh = layers.pop().unwrap();
        g.obj = layers.pop().unwrap();
        g.terrain = layers.pop().unwrap();
    } else {
        let on = OLD_W * OLD_H;
        let terrain = r.bytes();
        let obj = r.bytes();
        let fh = r.bytes();
        let fv = r.bytes();
        let dung = r.bytes();
        let litter = r.bytes();
        if terrain.len() != on || obj.len() != on || fh.len() != OLD_W * (OLD_H + 1) || fv.len() != (OLD_W + 1) * OLD_H || dung.len() != on || litter.len() != on {
            return -2;
        }
        for t in 0..on {
            let n = remap(t as u32) as usize;
            g.terrain[n] = terrain[t];
            g.obj[n] = obj[t];
            g.dung[n] = dung[t];
            g.litter[n] = litter[t];
        }
        for y in 0..=OLD_H {
            for x in 0..OLD_W {
                g.fh[(y + oy) * W + x + ox] = fh[y * OLD_W + x];
            }
        }
        for y in 0..OLD_H {
            for x in 0..=OLD_W {
                g.fv[(y + oy) * (W + 1) + x + ox] = fv[y * (OLD_W + 1) + x];
            }
        }
    }
    g.month_idx = (g.time / DAY_SECS / DAYS_PER_MONTH) as i64;
    let (fx, fy) = (ox as f32, oy as f32);
    let na = r.u32().min(5000);
    for _ in 0..na {
        let id = r.u32();
        let sp = r.u8().min(NS as u8 - 1);
        let x = r.f32() + fx;
        let y = r.f32() + fy;
        let male = r.u8() != 0;
        let age = r.f32();
        let hunger = r.f32();
        let thirst = r.f32();
        let happy = r.f32();
        let bcd = r.f32();
        let home = remap(r.u32());
        let name = r.u8();
        g.spawn_animal(sp, x, y, male, age);
        if let Some(a) = g.animals.last_mut() {
            a.id = id;
            a.hunger = hunger;
            a.thirst = thirst;
            a.happy = happy;
            a.breed_cd = bcd;
            a.home = home.min(N as u32 - 1);
            a.name = (name as usize % ANIMAL_NAMES.len()) as u16;
        }
    }
    let ns = r.u32().min(1000);
    for _ in 0..ns {
        let id = r.u32();
        let kind = r.u8();
        let x = r.f32() + fx;
        let y = r.f32() + fy;
        let variant = r.u8();
        let mut p = Person { id, kind, x, y, variant, target: -1, target_obj: -1, ..Default::default() };
        p.prev_tile = g.tile_of(x, y) as u32;
        p.name = "Staff".into();
        g.people.push(p);
    }
    let max_id = g.animals.iter().map(|a| a.id).chain(g.people.iter().map(|p| p.id)).max().unwrap_or(0);
    g.next_id = saved_next_id.max(max_id + 1);
    g.recompute_regions();
    let nf = r.u32().min(N as u32);
    for _ in 0..nf {
        let t = remap(r.u32()) as usize;
        let f = r.f32();
        if t < N {
            let rg = g.region[t] as usize;
            g.encs[rg].food = f;
        }
    }
    if ver >= 2 {
        g.unlocked = r.u32();
        g.tint = r.u8();
        let idx: Vec<usize> = (0..g.people.len()).filter(|&i| g.people[i].kind != 2).collect();
        for i in idx {
            g.people[i].name = String::from_utf8_lossy(&r.bytes()).into_owned();
            g.people[i].free = r.u8() != 0;
        }
        let nn = r.u32().min(N as u32);
        for _ in 0..nn {
            let t = remap(r.u32()) as usize;
            let n = String::from_utf8_lossy(&r.bytes()).into_owned();
            if t < N {
                let rg = g.region[t] as usize;
                g.encs[rg].name = n;
            }
        }
        let nb = r.u32().min(40000);
        for _ in 0..nb {
            let t = remap(r.u32());
            let d = r.u8();
            let v = r.u8();
            g.broken.push((t, d, v));
        }
        g.refresh_name_effects();
    }
    if !r.2 {
        return -3;
    }
    g.recompute_beauty();
    g.recompute_stats();
    g.msg(if ver < 3 { "Zoo loaded from an older save. There's land for sale all around you now!".into() } else { "Zoo loaded.".into() });
    s.game = g;
    1
}
