//! Core zoo simulation.

use crate::defs::*;
use std::collections::VecDeque;
use std::fmt::Write as _;

pub const DT: f32 = 0.05;
pub const DAY_SECS: f64 = 60.0;
pub const DAYS_PER_MONTH: f64 = 6.0;
pub const ADULT_AGE: f32 = 720.0;

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
    }
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16777216.0
    }
    pub fn range(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { self.next() % n }
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}

#[derive(Clone)]
pub struct Animal {
    pub id: u32,
    pub sp: u8,
    pub x: f32,
    pub y: f32,
    pub tx: f32,
    pub ty: f32,
    pub face: f32,
    pub male: bool,
    pub age: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub happy: f32,
    pub breed_cd: f32,
    pub home: u32,
    pub name: u16,
    pub moving: bool,
    pub idle: f32,
    pub escaped: bool,
    pub action: u8, // 0 idle,1 walk,2 eat,3 drink,4 sleep/rest,5 play
    pub action_t: f32,
}

#[derive(Clone, Default)]
pub struct Person {
    pub id: u32,
    pub kind: u8, // 2 guest, 3 keeper, 4 maintenance
    pub x: f32,
    pub y: f32,
    pub path: Vec<u32>,
    pub pi: usize,
    pub state: u8,
    pub timer: f32,
    pub target: i32,
    pub target_obj: i32,
    pub hunger: f32,
    pub thirst: f32,
    pub bladder: f32,
    pub energy: f32,
    pub fun: f32,
    pub happy: f32,
    pub cash: f32,
    pub seen: Vec<(u16, f32)>,
    pub thoughts: Vec<u8>,
    pub variant: u8,
    pub visit: f32,
    pub patience: f32,
    pub trash: f32,
    pub prev_tile: u32,
    pub ox: f32,
    pub oy: f32,
    pub face: f32,
    pub species_seen: u32,
    pub spent: f32,
    pub name: String,
    pub free: bool,
    pub sick: bool,
}

#[derive(Clone, Default)]
pub struct Enc {
    pub area: u32,
    pub food: f32,
    pub claimed: i32,
    pub terr: [u32; NT],
    pub objc: [u16; NOBJ],
    pub has_water: bool,
    pub dung: u32,
    pub gates: Vec<(u32, u32)>,
    pub min_fence: u8,
    pub sp_count: [u16; NS],
    pub sp_happy: [f32; NS],
    pub adults_m: [u16; NS],
    pub n_animals: u32,
    pub attraction: f32,
    pub avg_happy: f32,
    pub sx: f32,
    pub sy: f32,
    pub outside: bool,
    pub predators: bool,
    pub first_tile: u32,
    pub name: String,
}

#[derive(Clone, Copy, Default)]
pub struct Suit {
    pub terrain: f32,
    pub foliage: f32,
    pub space: f32,
    pub social: f32,
    pub shelter: f32,
    pub toys: f32,
    pub clean: f32,
    pub water: f32,
    pub safety: f32,
    pub total: f32,
}

#[derive(Default, Clone)]
pub struct Ledger {
    pub admissions: f64,
    pub concessions: f64,
    pub donations: f64,
    pub gifts: f64,
    pub salaries: f64,
    pub upkeep: f64,
    pub construction: f64,
    pub animals: f64,
}

pub struct Game {
    pub rng: Rng,
    pub terrain: Vec<u8>,
    pub obj: Vec<u8>,
    pub fh: Vec<u8>,
    pub fv: Vec<u8>,
    pub dung: Vec<u8>,
    pub litter: Vec<u8>,
    pub region: Vec<u16>,
    pub outside: u16,
    pub encs: Vec<Enc>,
    pub view_enc: Vec<u16>,
    pub beauty: Vec<f32>,
    pub animals: Vec<Animal>,
    pub people: Vec<Person>,
    pub next_id: u32,
    pub cash: f64,
    pub fame: f32,
    pub time: f64,
    pub admission: f32,
    pub ledger: Ledger,
    pub last_ledger: Ledger,
    pub messages: Vec<String>,
    pub goals_done: u32,
    pub baby_born: bool,
    pub need_regions: bool,
    pub need_beauty: bool,
    pub sec_acc: f32,
    pub stat_acc: f32,
    pub guest_acc: f32,
    pub month_idx: i64,
    pub guest_h: f32,
    pub animal_h: f32,
    pub variety: f32,
    pub entrance: u32,
    pub total_visitors: u32,
    // ---- classic cheats ----
    pub unlocked: u32,
    pub tint: u8,
    pub birds: f32,
    pub broken: Vec<(u32, u8, u8)>,
    /// Fence segments broken by escaping animals (repaired when they are recaptured).
    pub animal_breaks: Vec<(u32, u8, u8)>,
    pub fx_double_adm: bool,
    pub fx_double_don: bool,
    pub fx_wonderland: bool,
    pub fx_copralite: Vec<u16>,
    /// Owned land: x0, y0, x1 (exclusive), y1 (exclusive)
    pub own: [usize; 4],
}

#[inline]
pub fn tx(t: usize) -> usize { t % W }
#[inline]
pub fn ty(t: usize) -> usize { t / W }
#[inline]
pub fn tidx(x: usize, y: usize) -> usize { y * W + x }

pub fn clamp01(v: f32) -> f32 { v.max(0.0).min(1.0) }

impl Game {
    pub fn new(seed: u64) -> Game {
        Game::new_sized(seed, 0)
    }

    pub fn new_sized(seed: u64, size: u8) -> Game {
        let mut g = Game {
            rng: Rng(seed | 1),
            terrain: vec![T_GRASS; N],
            obj: vec![0; N],
            fh: vec![0; W * (H + 1)],
            fv: vec![0; (W + 1) * H],
            dung: vec![0; N],
            litter: vec![0; N],
            region: vec![0; N],
            outside: 0,
            encs: vec![],
            view_enc: vec![0; N],
            beauty: vec![0.0; N],
            animals: vec![],
            people: vec![],
            next_id: 1,
            cash: 75000.0,
            fame: 0.5,
            time: 9.0 * 3600.0 / 1440.0,
            admission: 20.0,
            ledger: Ledger::default(),
            last_ledger: Ledger::default(),
            messages: vec![],
            goals_done: 0,
            baby_born: false,
            need_regions: true,
            need_beauty: true,
            sec_acc: 0.0,
            stat_acc: 0.0,
            guest_acc: 0.0,
            month_idx: 0,
            guest_h: 0.5,
            animal_h: 0.0,
            variety: 0.0,
            entrance: 0,
            total_visitors: 0,
            unlocked: 0,
            tint: 0,
            birds: 0.0,
            broken: vec![],
            animal_breaks: vec![],
            fx_double_adm: false,
            fx_double_don: false,
            fx_wonderland: false,
            fx_copralite: vec![],
            own: [0, 0, W, H],
        };
        g.time = 0.0;
        let (sw, sh) = START_SIZES[(size as usize).min(2)];
        let x0 = (W - sw) / 2;
        g.own = [x0, H - sh, x0 + sw, H];
        g.generate();
        g
    }

    fn generate(&mut self) {
        let ex = W / 2;
        self.entrance = tidx(ex, H - 1) as u32;
        // Main path up from entrance and a crossroad.
        for y in (H - 14)..H {
            self.terrain[tidx(ex, y)] = T_PATH;
        }
        for x in (ex - 12)..=(ex + 12) {
            self.terrain[tidx(x, H - 14)] = T_PATH;
        }
        self.obj[tidx(ex - 1, H - 1)] = O_ENTRANCE;
        self.obj[tidx(ex + 1, H - 1)] = O_ENTRANCE;
        // Scatter some natural features across the whole world.
        for _ in 0..(N / 26) {
            let x = self.rng.range(W as u32) as usize;
            let y = self.rng.range(H as u32) as usize;
            let t = tidx(x, y);
            if self.terrain[t] == T_PATH || self.obj[t] != 0 {
                continue;
            }
            let near_path = (x as i32 - ex as i32).abs() < 3 && y > H - 18;
            if near_path {
                continue;
            }
            let k = [1u8, 1, 2, 13, 7, 7, 9, 11][self.rng.range(8) as usize];
            self.obj[t] = k;
        }
        // Ponds.
        for k in 0..(N / 1500) {
            let (px, py) = if k == 0 {
                (self.own[0] + 8, self.own[1] + 10)
            } else {
                (4 + self.rng.range((W - 12) as u32) as usize, 4 + self.rng.range((H - 30) as u32) as usize)
            };
            let (rx, ry) = (2.0 + self.rng.f() * 2.0, 1.4 + self.rng.f() * 1.2);
            for y in py.saturating_sub(4)..(py + 5).min(H) {
                for x in px.saturating_sub(5)..(px + 6).min(W) {
                    let d = (x as f32 - px as f32).powi(2) / (rx * rx) + (y as f32 - py as f32).powi(2) / (ry * ry);
                    let t = tidx(x, y);
                    if d < 1.0 && self.terrain[t] != T_PATH && y < H - 16 {
                        self.terrain[t] = T_WATER;
                        self.obj[t] = 0;
                    }
                }
            }
        }
        // Dirt patches.
        for _ in 0..(N / 520) {
            let cx = self.rng.range(W as u32) as i32;
            let cy = self.rng.range((H - 20) as u32) as i32;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let (x, y) = (cx + dx, cy + dy);
                    if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H && dx * dx + dy * dy <= 4 {
                        let t = tidx(x as usize, y as usize);
                        if self.terrain[t] == T_GRASS && self.rng.chance(0.7) {
                            self.terrain[t] = T_DIRT;
                        }
                    }
                }
            }
        }
        self.recompute_regions();
        self.recompute_beauty();
        self.msg("Welcome to Wildwood Zoo! Build an exhibit with fences, then adopt some animals.".into());
    }

    pub fn msg(&mut self, s: String) {
        if self.messages.len() < 40 {
            self.messages.push(s);
        }
    }

    pub fn spend(&mut self, amount: f64, cat: u8) -> bool {
        if self.cash < amount {
            self.msg("You don't have enough money for that!".into());
            return false;
        }
        self.cash -= amount;
        match cat {
            0 => self.ledger.construction += amount,
            1 => self.ledger.animals += amount,
            _ => self.ledger.salaries += amount,
        }
        true
    }

    // ---------- Fences & walkability ----------

    /// Edge value between tile t and neighbor in direction d (0=N,1=E,2=S,3=W).
    pub fn edge(&self, t: usize, d: u8) -> u8 {
        let (x, y) = (tx(t), ty(t));
        match d {
            0 => self.fh[y * W + x],
            2 => self.fh[(y + 1) * W + x],
            3 => self.fv[y * (W + 1) + x],
            _ => self.fv[y * (W + 1) + x + 1],
        }
    }
    pub fn set_edge(&mut self, t: usize, d: u8, v: u8) {
        let (x, y) = (tx(t), ty(t));
        match d {
            0 => self.fh[y * W + x] = v,
            2 => self.fh[(y + 1) * W + x] = v,
            3 => self.fv[y * (W + 1) + x] = v,
            _ => self.fv[y * (W + 1) + x + 1] = v,
        }
    }
    pub fn neighbor(&self, t: usize, d: u8) -> Option<usize> {
        let (x, y) = (tx(t), ty(t));
        match d {
            0 if y > 0 => Some(t - W),
            2 if y + 1 < H => Some(t + W),
            3 if x > 0 => Some(t - 1),
            1 if x + 1 < W => Some(t + 1),
            _ => None,
        }
    }

    pub fn owned(&self, t: usize) -> bool {
        let (x, y) = (tx(t), ty(t));
        x >= self.own[0] && x < self.own[2] && y >= self.own[1] && y < self.own[3]
    }

    pub fn owned_xy(&self, x: i32, y: i32) -> bool {
        x >= self.own[0] as i32 && x < self.own[2] as i32 && y >= self.own[1] as i32 && y < self.own[3] as i32
    }

    /// Cost of expanding the zoo in a direction (0 north, 1 west, 2 east), or -1 if it can't grow that way.
    pub fn expand_cost(&self, dir: u8) -> i32 {
        let [x0, y0, x1, y1] = self.own;
        let (w, h) = (x1 - x0, y1 - y0);
        let added = match dir {
            0 => w * y0.min(LAND_STEP),
            1 => h * x0.min(LAND_STEP),
            _ => h * (W - x1).min(LAND_STEP),
        };
        if added == 0 { -1 } else { added as i32 * LAND_PRICE }
    }

    pub fn expand(&mut self, dir: u8) -> i32 {
        let cost = self.expand_cost(dir);
        if cost < 0 {
            self.msg("The zoo can't grow any further in that direction.".into());
            return -1;
        }
        if !self.spend(cost as f64, 0) {
            return -1;
        }
        match dir {
            0 => self.own[1] = self.own[1].saturating_sub(LAND_STEP),
            1 => self.own[0] = self.own[0].saturating_sub(LAND_STEP),
            _ => self.own[2] = (self.own[2] + LAND_STEP).min(W),
        }
        let dirn = ["north", "west", "east"][dir.min(2) as usize];
        self.msg(format!("You bought more land to the {}! The zoo is now {} x {} tiles.", dirn, self.own[2] - self.own[0], self.own[3] - self.own[1]));
        self.need_regions = true;
        cost
    }

    pub fn blocks(&self, t: usize) -> bool {
        OBJS[self.obj[t] as usize].blocks
    }

    /// rule: 0 guest, 1 maintenance (no gates), 2 keeper (gates ok)
    pub fn walkable(&self, t: usize, rule: u8) -> bool {
        if self.blocks(t) || !self.owned(t) {
            return false;
        }
        match rule {
            0 => self.terrain[t] == T_PATH,
            _ => self.terrain[t] != T_WATER,
        }
    }

    pub fn bfs<F: Fn(&Game, usize) -> bool>(&self, start: usize, rule: u8, goal: F) -> Option<Vec<u32>> {
        if goal(self, start) {
            return Some(vec![]);
        }
        let mut prev = vec![u32::MAX; N];
        prev[start] = start as u32;
        let mut q = VecDeque::new();
        q.push_back(start);
        while let Some(c) = q.pop_front() {
            for d in 0..4u8 {
                let Some(n) = self.neighbor(c, d) else { continue };
                if prev[n] != u32::MAX {
                    continue;
                }
                let e = self.edge(c, d);
                if e != 0 && !(rule == 2 && e & GATE_BIT != 0) {
                    continue;
                }
                if !self.walkable(n, rule) {
                    continue;
                }
                prev[n] = c as u32;
                if goal(self, n) {
                    let mut path = vec![n as u32];
                    let mut cur = c;
                    while cur != start {
                        path.push(cur as u32);
                        cur = prev[cur] as usize;
                    }
                    path.reverse();
                    return Some(path);
                }
                q.push_back(n);
            }
        }
        None
    }

    pub fn recompute_regions(&mut self) {
        let old_region = self.region.clone();
        let old_food: Vec<f32> = self.encs.iter().map(|e| e.food).collect();
        let old_names: Vec<String> = self.encs.iter().map(|e| e.name.clone()).collect();
        let mut region = vec![u16::MAX; N];
        let mut count = 0u16;
        let mut q = VecDeque::new();
        for s in 0..N {
            if region[s] != u16::MAX {
                continue;
            }
            region[s] = count;
            q.push_back(s);
            while let Some(c) = q.pop_front() {
                for d in 0..4u8 {
                    if let Some(n) = self.neighbor(c, d) {
                        if region[n] == u16::MAX && self.edge(c, d) == 0 {
                            region[n] = count;
                            q.push_back(n);
                        }
                    }
                }
            }
            count += 1;
        }
        self.region = region;
        self.outside = self.region[self.entrance as usize];
        let mut encs = vec![Enc::default(); count as usize];
        for (i, e) in encs.iter_mut().enumerate() {
            e.claimed = -1;
            e.outside = i as u16 == self.outside;
            e.first_tile = u32::MAX;
        }
        for t in 0..N {
            let r = self.region[t] as usize;
            if encs[r].first_tile == u32::MAX {
                encs[r].first_tile = t as u32;
                let or = old_region[t] as usize;
                if or < old_food.len() {
                    encs[r].food = old_food[or];
                    encs[r].name = old_names[or].clone();
                }
            }
        }
        // Gates
        for t in 0..N {
            for d in 0..4u8 {
                if let Some(n) = self.neighbor(t, d) {
                    let e = self.edge(t, d);
                    if e & GATE_BIT != 0 && self.region[n] == self.outside && self.region[t] != self.outside {
                        let r = self.region[t] as usize;
                        encs[r].gates.push((t as u32, n as u32));
                    }
                }
            }
        }
        self.encs = encs;
        self.refresh_name_effects();
        for p in self.people.iter_mut() {
            if p.kind != 2 {
                p.state = 0;
                p.path.clear();
                p.pi = 0;
                p.target = -1;
            }
        }
        // View map: path tiles within 2 of an enclosure tile.
        let mut view = vec![0u16; N];
        for t in 0..N {
            if self.terrain[t] != T_PATH || self.region[t] != self.outside {
                continue;
            }
            let (x, y) = (tx(t) as i32, ty(t) as i32);
            let mut best = 0u16;
            let mut bd = 99;
            for dy in -2..=2i32 {
                for dx in -2..=2i32 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
                        continue;
                    }
                    let r = self.region[tidx(nx as usize, ny as usize)];
                    let d = dx.abs().max(dy.abs());
                    if r != self.outside && d < bd {
                        bd = d;
                        best = r + 1;
                    }
                }
            }
            view[t] = best;
        }
        self.view_enc = view;
        self.need_regions = false;
        self.recompute_stats();
    }

    pub fn recompute_beauty(&mut self) {
        let mut b = vec![0.0f32; N];
        for t in 0..N {
            let be = OBJS[self.obj[t] as usize].beauty;
            if be <= 0.0 {
                continue;
            }
            let (x, y) = (tx(t) as i32, ty(t) as i32);
            for dy in -2..=2i32 {
                for dx in -2..=2i32 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx >= 0 && ny >= 0 && nx < W as i32 && ny < H as i32 {
                        let f = if dx.abs().max(dy.abs()) <= 1 { 1.0 } else { 0.5 };
                        b[tidx(nx as usize, ny as usize)] += be * f;
                    }
                }
            }
        }
        for v in b.iter_mut() {
            *v = v.min(1.0);
        }
        self.beauty = b;
        self.need_beauty = false;
    }

    pub fn recompute_stats(&mut self) {
        for e in self.encs.iter_mut() {
            e.area = 0;
            e.terr = [0; NT];
            e.objc = [0; NOBJ];
            e.dung = 0;
            e.sp_count = [0; NS];
            e.sp_happy = [0.0; NS];
            e.adults_m = [0; NS];
            e.n_animals = 0;
            e.sx = 0.0;
            e.sy = 0.0;
            e.min_fence = 9;
            e.predators = false;
        }
        for t in 0..N {
            let r = self.region[t] as usize;
            let mut mf = 9u8;
            for d in 0..4u8 {
                let ev = self.edge(t, d);
                if ev != 0 && (ev & 3) < mf {
                    mf = ev & 3;
                }
            }
            let e = &mut self.encs[r];
            if mf < e.min_fence {
                e.min_fence = mf;
            }
            e.area += 1;
            e.terr[self.terrain[t] as usize] += 1;
            e.objc[self.obj[t] as usize] += 1;
            e.dung += self.dung[t] as u32;
            e.sx += tx(t) as f32 + 0.5;
            e.sy += ty(t) as f32 + 0.5;
        }
        for e in self.encs.iter_mut() {
            e.has_water = e.terr[T_WATER as usize] > 0 || e.objc[O_TROUGH as usize] > 0;
            if e.area > 0 {
                e.sx /= e.area as f32;
                e.sy /= e.area as f32;
            }
            if e.min_fence == 9 {
                e.min_fence = 0;
            }
        }
        for a in self.animals.iter() {
            let r = self.region[self.tile_of(a.x, a.y)] as usize;
            let e = &mut self.encs[r];
            e.sp_count[a.sp as usize] += 1;
            e.sp_happy[a.sp as usize] += a.happy;
            e.n_animals += 1;
            if a.male && a.age >= ADULT_AGE {
                e.adults_m[a.sp as usize] += 1;
            }
            if SPECIES[a.sp as usize].predator {
                e.predators = true;
            }
        }
        for e in self.encs.iter_mut() {
            let mut att = 0.0;
            let mut hs = 0.0;
            for s in 0..NS {
                let c = e.sp_count[s];
                if c > 0 {
                    let h = e.sp_happy[s] / c as f32;
                    e.sp_happy[s] = h;
                    hs += h * c as f32;
                    att += SPECIES[s].fame * (0.25 + 0.75 * h) * (c.min(5) as f32).sqrt() / 10.0;
                }
            }
            e.attraction = att;
            e.avg_happy = if e.n_animals > 0 { hs / e.n_animals as f32 } else { 0.0 };
        }
    }

    pub fn tile_of(&self, x: f32, y: f32) -> usize {
        let xi = (x.max(0.0) as usize).min(W - 1);
        let yi = (y.max(0.0) as usize).min(H - 1);
        tidx(xi, yi)
    }

    pub fn suit(&self, r: usize, sp: usize) -> Suit {
        let e = &self.encs[r];
        let s = &SPECIES[sp];
        let mut su = Suit::default();
        let area = e.area.max(1) as f32;
        // terrain
        let mut diff = 0.0;
        for i in 0..NT {
            diff += (s.terrain[i] - e.terr[i] as f32 / area).abs();
        }
        su.terrain = clamp01(1.0 - diff * 0.5);
        // foliage
        let mut matched = 0.0;
        let mut wrong = 0.0;
        for k in 1..NOBJ {
            let o = &OBJS[k];
            if o.cat != C_FOLIAGE || o.biome == 0 {
                continue;
            }
            let c = e.objc[k] as f32;
            if o.biome & s.foliage != 0 {
                matched += c;
            } else {
                wrong += c;
            }
        }
        let mut f = if s.density <= 0.0 { 1.0 } else { (matched / (area * s.density)).min(1.0) };
        f -= (wrong / area * 3.0).min(0.6);
        su.foliage = clamp01(f);
        // space
        let mut needed = 0.0;
        for i in 0..NS {
            needed += SPECIES[i].space * e.sp_count[i] as f32;
        }
        su.space = if needed <= 0.0 { 1.0 } else { clamp01(area / needed) };
        // social
        let c = e.sp_count[sp].max(1);
        su.social = if c < s.group_min {
            0.3 + 0.7 * (c as f32 - 1.0) / (s.group_min as f32 - 1.0).max(1.0)
        } else if c > s.group_max {
            s.group_max as f32 / c as f32
        } else {
            1.0
        };
        su.shelter = if e.objc[O_SHELTER as usize] > 0 { 1.0 } else { 0.4 };
        su.toys = if e.objc[O_TOY as usize] > 0 { 1.0 } else { 0.5 };
        su.clean = clamp01(1.0 - e.dung as f32 / (area * 0.12));
        su.water = if e.has_water { 1.0 } else { 0.0 };
        su.safety = if !s.predator && e.predators { 0.2 } else { 1.0 };
        if e.outside {
            su = Suit { terrain: 0.2, foliage: 0.2, space: 1.0, social: 0.2, shelter: 0.0, toys: 0.0, clean: 0.5, water: 0.0, safety: 0.2, total: 0.0 };
        }
        su.total = 0.28 * su.terrain + 0.18 * su.foliage + 0.14 * su.space + 0.14 * su.social
            + 0.08 * su.shelter + 0.06 * su.toys + 0.06 * su.clean + 0.06 * su.water;
        // The wrong biome can't be made up for with toys and shelters.
        let biome = 0.6 * su.terrain + 0.4 * su.foliage;
        su.total *= 0.35 + 0.65 * biome;
        su.total = (su.total * 1.12).min(1.0);
        su.total *= 0.4 + 0.6 * su.safety;
        su
    }

    pub fn animal_walkable(&self, t: usize, sp: usize) -> bool {
        if self.blocks(t) || !self.owned(t) {
            return false;
        }
        if self.terrain[t] == T_WATER && !SPECIES[sp].swims {
            return false;
        }
        if self.terrain[t] == T_PATH && self.region[t] != self.outside {
            return true;
        }
        true
    }

    // ---------- Player actions ----------

    pub fn paint_terrain(&mut self, x: i32, y: i32, t: u8, r: i32) -> i32 {
        if t as usize >= NT {
            return -1;
        }
        let mut cost = 0;
        let mut tiles = vec![];
        for dy in -r..=r {
            for dx in -r..=r {
                if r > 0 && dx * dx + dy * dy > r * r + r {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
                    continue;
                }
                let i = tidx(nx as usize, ny as usize);
                if self.terrain[i] == t || i == self.entrance as usize || !self.owned(i) {
                    continue;
                }
                if (t == T_PATH || t == T_WATER) && self.obj[i] != 0 {
                    continue;
                }
                cost += TERRAIN_COST[t as usize];
                tiles.push(i);
            }
        }
        if tiles.is_empty() {
            return 0;
        }
        if !self.spend(cost as f64, 0) {
            return -1;
        }
        for i in tiles {
            self.terrain[i] = t;
        }
        // Refresh exhibit stats right away so panels are correct even while paused.
        self.recompute_stats();
        // Animals in water that can't swim get nudged out next step (handled in update).
        self.recompute_view_only();
        cost
    }

    fn recompute_view_only(&mut self) {
        // terrain change can affect paths viewing exhibits
        self.need_regions = true;
    }

    pub fn place_object(&mut self, x: i32, y: i32, k: u8) -> i32 {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 || k == 0 || k as usize >= NOBJ - 1 {
            return -1;
        }
        let t = tidx(x as usize, y as usize);
        if !self.owned(t) {
            self.msg("You don't own that land yet. Buy it with the Land tool.".into());
            return -8;
        }
        if self.obj[t] != 0 {
            return -2;
        }
        if self.terrain[t] == T_WATER {
            self.msg("You can't build that in water.".into());
            return -3;
        }
        let def = &OBJS[k as usize];
        if self.terrain[t] == T_PATH && def.blocks {
            self.msg("That can't be placed on a path.".into());
            return -4;
        }
        if def.blocks && self.animals.iter().any(|a| self.tile_of(a.x, a.y) == t) {
            return -5;
        }
        if def.cat == C_ANIMAL_ITEM && self.region[t] == self.outside {
            self.msg(format!("The {} must go inside an exhibit.", def.name));
            return -6;
        }
        if !self.spend(def.cost as f64, 0) {
            return -7;
        }
        self.obj[t] = k;
        self.need_beauty = true;
        if def.blocks {
            // displace people standing here
            for p in self.people.iter_mut() {
                p.path.clear();
                p.pi = 0;
                if p.state != 0 && p.kind == 2 && p.state != 4 {
                    p.state = 0;
                }
            }
        }
        self.recompute_stats();
        def.cost
    }

    pub fn fence_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, kind: u8, dry: bool) -> i32 {
        let o = self.own;
        let (ax, bx) = (x0.min(x1).max(o[0] as i32), x0.max(x1).min(o[2] as i32 - 1));
        let (ay, by) = (y0.min(y1).max(o[1] as i32), y0.max(y1).min(o[3] as i32 - 1));
        let kind = kind.clamp(1, 3);
        if bx - ax < 1 || by - ay < 1 {
            return 0;
        }
        let mut edges: Vec<(usize, u8)> = vec![];
        for x in ax..=bx {
            edges.push((tidx(x as usize, ay as usize), 0));
            edges.push((tidx(x as usize, by as usize), 2));
        }
        for y in ay..=by {
            edges.push((tidx(ax as usize, y as usize), 3));
            edges.push((tidx(bx as usize, y as usize), 1));
        }
        let mut cost = 0;
        for &(t, d) in edges.iter() {
            if self.edge(t, d) & 3 != kind {
                cost += FENCE_COST[kind as usize];
            }
        }
        if dry {
            return cost;
        }
        if cost == 0 {
            return 0;
        }
        if !self.spend(cost as f64, 0) {
            return -1;
        }
        for &(t, d) in edges.iter() {
            let old = self.edge(t, d);
            let v = kind | (old & GATE_BIT);
            self.set_edge(t, d, v);
        }
        // Gate: prefer an edge adjacent to a path outside, else middle of the south side.
        let has_gate = edges.iter().any(|&(t, d)| self.edge(t, d) & GATE_BIT != 0);
        if !has_gate {
            let mut gate = None;
            for &(t, d) in edges.iter() {
                if let Some(n) = self.neighbor(t, d) {
                    if self.terrain[n] == T_PATH {
                        gate = Some((t, d));
                        break;
                    }
                }
            }
            let (gt, gd) = gate.unwrap_or((tidx(((ax + bx) / 2) as usize, by as usize), 2));
            let v = self.edge(gt, gd);
            self.set_edge(gt, gd, v | GATE_BIT);
        }
        self.recompute_regions();
        cost
    }

    fn nearest_edge(&self, fx: f32, fy: f32) -> Option<(usize, u8, f32)> {
        if fx < 0.0 || fy < 0.0 || fx >= W as f32 || fy >= H as f32 {
            return None;
        }
        let t = self.tile_of(fx, fy);
        let (lx, ly) = (fx - tx(t) as f32, fy - ty(t) as f32);
        let ds = [ly, 1.0 - lx, 1.0 - ly, lx];
        let mut best = 0u8;
        for d in 1..4u8 {
            if ds[d as usize] < ds[best as usize] {
                best = d;
            }
        }
        Some((t, best, ds[best as usize]))
    }

    pub fn toggle_gate(&mut self, fx: f32, fy: f32) -> i32 {
        let Some((t, d, _)) = self.nearest_edge(fx, fy) else { return -1 };
        if !self.owned(t) {
            return -1;
        }
        let e = self.edge(t, d);
        if e == 0 {
            self.msg("Click on a fence to add or remove a gate.".into());
            return -1;
        }
        if !self.spend(50.0, 0) {
            return -1;
        }
        self.set_edge(t, d, e ^ GATE_BIT);
        self.recompute_regions();
        50
    }

    pub fn bulldoze(&mut self, fx: f32, fy: f32) -> i32 {
        let Some((t, d, dist)) = self.nearest_edge(fx, fy) else { return -1 };
        if !self.owned(t) {
            return -1;
        }
        if self.obj[t] != 0 && self.obj[t] != O_ENTRANCE && dist > 0.18 {
            let k = self.obj[t] as usize;
            self.obj[t] = 0;
            let refund = OBJS[k].cost as f64 * 0.5;
            self.cash += refund;
            self.need_beauty = true;
            self.recompute_stats();
            return 1;
        }
        if self.edge(t, d) != 0 && dist < 0.35 {
            let k = (self.edge(t, d) & 3) as usize;
            self.set_edge(t, d, 0);
            self.cash += FENCE_COST[k] as f64 * 0.5;
            self.recompute_regions();
            return 2;
        }
        if self.terrain[t] == T_PATH && t != self.entrance as usize {
            self.terrain[t] = T_GRASS;
            self.need_regions = true;
            return 3;
        }
        if self.dung[t] > 0 {
            self.dung[t] = 0;
            return 4;
        }
        0
    }

    pub fn adopt(&mut self, x: i32, y: i32, sp: u8, male: bool) -> i32 {
        if sp as usize >= NS || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return -1;
        }
        let t = tidx(x as usize, y as usize);
        if self.region[t] == self.outside {
            self.msg("Animals must be placed inside a fenced exhibit.".into());
            return -2;
        }
        if !self.animal_walkable(t, sp as usize) {
            self.msg("That animal can't be placed there.".into());
            return -3;
        }
        if sp as usize >= NS_PUBLIC && self.unlocked & (1 << sp) == 0 {
            self.msg("That species hasn't been discovered yet...".into());
            return -5;
        }
        let s = &SPECIES[sp as usize];
        if !self.spend(s.cost as f64, 1) {
            return -4;
        }
        let id = self.spawn_animal(sp, x as f32 + 0.5, y as f32 + 0.5, male, ADULT_AGE + 100.0);
        let name = ANIMAL_NAMES[(id as usize * 7) % ANIMAL_NAMES.len()];
        self.msg(format!("{} the {} has arrived at your zoo!", name, s.name));
        let ns = self.encs[self.region[t] as usize].min_fence;
        if ns < s.strength {
            self.msg(format!("Warning: {} need a stronger fence ({}).", s.plural, FENCE_NAMES[s.strength as usize]));
        }
        self.recompute_stats();
        id as i32
    }

    pub fn spawn_animal(&mut self, sp: u8, x: f32, y: f32, male: bool, age: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let t = self.tile_of(x, y) as u32;
        self.animals.push(Animal {
            id,
            sp,
            x,
            y,
            tx: x,
            ty: y,
            face: 1.0,
            male,
            age,
            hunger: 0.8,
            thirst: 0.8,
            happy: 0.5,
            breed_cd: 300.0,
            home: t,
            name: ((id as usize * 7) % ANIMAL_NAMES.len()) as u16,
            moving: false,
            idle: 1.0,
            escaped: false,
            action: 0,
            action_t: 0.0,
        });
        id
    }

    pub fn hire(&mut self, kind: u8) -> i32 {
        let cost = if kind == 3 { 500.0 } else { 300.0 };
        if !self.spend(cost, 2) {
            return -1;
        }
        let id = self.next_id;
        self.next_id += 1;
        let e = self.entrance as usize;
        let mut p = Person { id, kind, x: tx(e) as f32 + 0.5, y: ty(e) as f32 + 0.5, target: -1, target_obj: -1, ..Default::default() };
        p.variant = self.rng.range(4) as u8;
        p.prev_tile = e as u32;
        p.name = format!("{} {}", FIRST_NAMES[self.rng.range(40) as usize], LAST_NAMES[self.rng.range(26) as usize]);
        self.people.push(p);
        self.msg(if kind == 3 { "A new zookeeper has been hired.".into() } else { "A new maintenance worker has been hired.".into() });
        id as i32
    }

    pub fn remove_entity(&mut self, id: u32, mode: u8) -> i32 {
        // mode 0: sell animal, 1: recapture, 2: fire staff
        if mode == 2 {
            if let Some(i) = self.people.iter().position(|p| p.id == id && p.kind != 2) {
                let p = self.people.remove(i);
                if p.target >= 0 && (p.target as usize) < self.encs.len() {
                    self.encs[p.target as usize].claimed = -1;
                }
                self.msg("Staff member dismissed.".into());
                return 1;
            }
            return 0;
        }
        let Some(i) = self.animals.iter().position(|a| a.id == id) else { return 0 };
        if mode == 0 {
            let a = self.animals.remove(i);
            let s = &SPECIES[a.sp as usize];
            let v = s.cost as f64 * if a.age >= ADULT_AGE { 0.5 } else { 0.7 };
            self.cash += v;
            self.msg(format!("{} the {} was sent to a conservation program. (+${:.0})", ANIMAL_NAMES[a.name as usize], s.name, v));
            self.recompute_stats();
            return 1;
        }
        // recapture
        if !self.animals[i].escaped {
            return 0;
        }
        if !self.spend(400.0, 1) {
            return 0;
        }
        let home = self.animals[i].home as usize;
        if self.region[home] == self.outside && !self.animal_breaks.is_empty() {
            // Keepers patch up the fences escaped animals broke, then bring the animal home.
            let list = std::mem::take(&mut self.animal_breaks);
            let mut n = 0;
            for &(t, d, v) in list.iter() {
                if (t as usize) < N && self.edge(t as usize, d) == 0 {
                    self.set_edge(t as usize, d, v);
                    n += 1;
                }
            }
            self.broken.retain(|b| !list.contains(b));
            self.recompute_regions();
            if n > 0 {
                self.msg(format!("Keepers repaired {} broken fence segment{}.", n, if n == 1 { "" } else { "s" }));
            }
        }
        let ok = self.region[home] != self.outside;
        let sp = self.animals[i].sp as usize;
        if ok {
            // find a walkable tile in home region
            let r = self.region[home];
            let mut dest = home;
            if !self.animal_walkable(home, sp) {
                for t in 0..N {
                    if self.region[t] == r && self.animal_walkable(t, sp) {
                        dest = t;
                        break;
                    }
                }
            }
            let a = &mut self.animals[i];
            a.x = tx(dest) as f32 + 0.5;
            a.y = ty(dest) as f32 + 0.5;
            a.tx = a.x;
            a.ty = a.y;
            a.moving = false;
            a.escaped = false;
            self.msg("The animal was tranquilized and returned to its exhibit.".into());
        } else {
            let a = self.animals.remove(i);
            self.cash += SPECIES[a.sp as usize].cost as f64 * 0.3;
            self.msg("The animal's exhibit no longer exists, so it was sent to another zoo.".into());
        }
        self.recompute_stats();
        1
    }

    // ---------- Simulation ----------

    pub fn step(&mut self) {
        if self.need_regions {
            self.recompute_regions();
        }
        if self.need_beauty {
            self.recompute_beauty();
        }
        self.time += DT as f64;
        self.sec_acc += DT;
        let second = self.sec_acc >= 1.0;
        if second {
            self.sec_acc -= 1.0;
        }
        self.update_animals(second);
        self.update_people(second);
        if second {
            self.cheat_tick();
            self.spawn_guests();
            self.stat_acc += 1.0;
            if self.stat_acc >= 1.0 {
                self.stat_acc = 0.0;
                self.recompute_stats();
                self.update_fame();
                self.check_goals();
            }
        }
        let m = (self.time / DAY_SECS / DAYS_PER_MONTH) as i64;
        if m != self.month_idx {
            self.month_idx = m;
            self.end_month();
        }
    }

    fn end_month(&mut self) {
        let mut sal = 0.0;
        for p in self.people.iter().filter(|p| !p.free) {
            sal += match p.kind {
                3 => 300.0,
                4 => 200.0,
                _ => 0.0,
            };
        }
        let mut up = 0.0;
        for t in 0..N {
            up += OBJS[self.obj[t] as usize].upkeep as f64;
        }
        let mut an = 0.0;
        for a in self.animals.iter() {
            an += SPECIES[a.sp as usize].cost as f64 * 0.02;
        }
        self.cash -= sal + up + an;
        self.ledger.salaries += sal;
        self.ledger.upkeep += up;
        self.ledger.animals += an;
        self.last_ledger = std::mem::take(&mut self.ledger);
        let l = &self.last_ledger;
        let inc = l.admissions + l.concessions + l.donations + l.gifts;
        let exp = l.salaries + l.upkeep + l.construction + l.animals;
        self.msg(format!("Monthly report: income ${:.0}, expenses ${:.0}, profit ${:.0}.", inc, exp, inc - exp));
    }

    fn update_fame(&mut self) {
        let guests: Vec<&Person> = self.people.iter().filter(|p| p.kind == 2).collect();
        self.guest_h = if guests.is_empty() { 0.5 } else { guests.iter().map(|p| p.happy).sum::<f32>() / guests.len() as f32 };
        self.animal_h = if self.animals.is_empty() { 0.0 } else { self.animals.iter().map(|a| a.happy).sum::<f32>() / self.animals.len() as f32 };
        let species = self.species_count();
        self.variety = (species as f32 / 10.0).min(1.0);
        let count_f = (self.animals.len() as f32 / 15.0).min(1.0);
        let mut rating = 0.25 * self.guest_h + 0.30 * self.animal_h * count_f + 0.45 * self.variety;
        if self.animals.is_empty() {
            rating *= 0.3;
        }
        let target = (rating * 5.5).min(5.0);
        let rate = if target > self.fame { 0.004 } else { 0.008 };
        self.fame += (target - self.fame) * rate;
        self.fame = self.fame.clamp(0.0, 5.0);
    }

    pub fn species_count(&self) -> u32 {
        let mut mask = 0u32;
        for a in self.animals.iter() {
            if !a.escaped {
                mask |= 1 << a.sp;
            }
        }
        mask.count_ones()
    }

    fn check_goals(&mut self) {
        let guests = self.people.iter().filter(|p| p.kind == 2).count();
        let keepers = self.people.iter().filter(|p| p.kind == 3).count();
        let sc = self.species_count();
        let conds = [
            !self.animals.is_empty(),
            keepers > 0,
            guests >= 40,
            sc >= 3,
            self.fame >= 2.0,
            self.baby_born,
            self.animals.len() >= 20,
            self.fame >= 3.5,
            sc >= 7,
            self.fame >= 4.95,
            self.cash >= 150000.0,
        ];
        for (i, &c) in conds.iter().enumerate() {
            if c && self.goals_done & (1 << i) == 0 {
                self.goals_done |= 1 << i;
                let (name, reward) = GOALS[i];
                self.cash += reward as f64;
                if reward > 0 {
                    self.msg(format!("Goal complete: {}! Award: ${}", name, reward));
                } else {
                    self.msg(format!("Goal complete: {}!", name));
                }
            }
        }
    }

    fn update_animals(&mut self, second: bool) {
        let mut broke: Vec<(usize, u8, usize)> = vec![];
        let mut babies: Vec<(u8, f32, f32)> = vec![];
        let mut escaped_msgs: Vec<String> = vec![];
        let n = self.animals.len();
        for i in 0..n {
            let mut a = self.animals[i].clone();
            let sp = a.sp as usize;
            let s = &SPECIES[sp];
            let t = self.tile_of(a.x, a.y);
            let r = self.region[t] as usize;
            a.hunger = (a.hunger - DT / 160.0).max(0.0);
            a.thirst = (a.thirst - DT / 120.0).max(0.0);
            a.age += DT;
            a.breed_cd -= DT;
            let outside = r == self.outside as usize;
            if outside && !a.escaped {
                a.escaped = true;
                escaped_msgs.push(format!("ALERT: {} the {} has escaped!", ANIMAL_NAMES[a.name as usize], s.name));
            } else if !outside && a.escaped {
                a.escaped = false;
                a.home = t as u32;
            }
            if second {
                if !outside {
                    let e = &self.encs[r];
                    if a.hunger < 0.5 && e.food >= 1.0 {
                        self.encs[r].food -= 1.0;
                        a.hunger = 1.0;
                        a.action = 2;
                        a.action_t = 3.0;
                        a.moving = false;
                    } else if a.thirst < 0.55 && self.encs[r].has_water {
                        a.thirst = 1.0;
                        a.action = 3;
                        a.action_t = 2.0;
                        a.moving = false;
                    }
                }
                let su = self.suit(r, sp);
                let needs = a.hunger.min(a.thirst);
                let need_f = 0.3 + 0.7 * clamp01(needs * 2.0);
                let target = clamp01(su.total * need_f * 1.08);
                a.happy += (target - a.happy) * 0.06;
                // breeding
                if !outside && !a.male && a.age >= ADULT_AGE && a.breed_cd <= 0.0 && a.happy > 0.7 {
                    let e = &self.encs[r];
                    if e.adults_m[sp] > 0 && e.sp_count[sp] < s.group_max + 2 && su.space > 0.7 && self.rng.chance(1.0 / 150.0) {
                        babies.push((a.sp, a.x, a.y));
                        a.breed_cd = 1500.0;
                    }
                }
                // dung
                if !outside && self.rng.chance(1.0 / 70.0) && self.dung[t] < 9 {
                    self.dung[t] += 1;
                }
            }
            // actions / movement
            if a.action_t > 0.0 {
                a.action_t -= DT;
                if a.action_t <= 0.0 {
                    a.action = 0;
                }
            } else if a.moving {
                let (dx, dy) = (a.tx - a.x, a.ty - a.y);
                let d = (dx * dx + dy * dy).sqrt();
                let spd = s.speed * if a.age < ADULT_AGE { 0.7 } else { 1.0 } * if a.escaped { 1.5 } else { 1.0 };
                let stepd = spd * DT;
                if d <= stepd {
                    a.x = a.tx;
                    a.y = a.ty;
                    a.moving = false;
                    a.idle = 0.5 + self.rng.f() * if a.happy < 0.3 { 1.0 } else { 4.0 };
                    a.action = 0;
                } else {
                    a.x += dx / d * stepd;
                    a.y += dy / d * stepd;
                    if dx.abs() > 0.01 {
                        a.face = dx.signum();
                    }
                    a.action = 1;
                }
            } else {
                a.idle -= DT;
                if a.idle <= 0.0 {
                    // Rest sometimes, play if toy.
                    let roll = self.rng.f();
                    if roll < 0.12 {
                        a.action = 4;
                        a.action_t = 4.0 + self.rng.f() * 6.0;
                    } else if roll < 0.18 && !outside && self.encs[r].objc[O_TOY as usize] > 0 {
                        a.action = 5;
                        a.action_t = 3.0;
                    } else {
                        // pick a random neighbor direction (with momentum, 2-3 tiles)
                        let d = self.rng.range(4) as u8;
                        if let Some(nt) = self.neighbor(t, d) {
                            let e = self.edge(t, d);
                            if e != 0 {
                                // fence: unhappy animals or ones that are too strong may break it
                                let strength = e & 3;
                                let mut p = 0.0;
                                if a.happy < 0.25 {
                                    p += 0.04;
                                }
                                if strength < s.strength {
                                    p += 0.015 * (s.strength - strength) as f32 * if a.happy < 0.5 { 2.0 } else { 0.5 };
                                }
                                if p > 0.0 && self.rng.chance(p) {
                                    broke.push((t, d, i));
                                }
                                a.idle = 0.5;
                            } else if self.animal_walkable(nt, sp) {
                                a.tx = tx(nt) as f32 + 0.2 + self.rng.f() * 0.6;
                                a.ty = ty(nt) as f32 + 0.2 + self.rng.f() * 0.6;
                                // keep the line within the two tiles
                                if d == 0 || d == 2 {
                                    a.tx = a.tx.clamp(tx(t) as f32 + 0.15, tx(t) as f32 + 0.85);
                                } else {
                                    a.ty = a.ty.clamp(ty(t) as f32 + 0.15, ty(t) as f32 + 0.85);
                                }
                                a.moving = true;
                            } else {
                                a.idle = 0.3;
                            }
                        }
                    }
                }
            }
            // non-swimmer in water: push to a dry neighbor
            if self.terrain[t] == T_WATER && !s.swims {
                for d in 0..4u8 {
                    if let Some(nt) = self.neighbor(t, d) {
                        if self.edge(t, d) == 0 && self.terrain[nt] != T_WATER && !self.blocks(nt) {
                            a.x = tx(nt) as f32 + 0.5;
                            a.y = ty(nt) as f32 + 0.5;
                            a.moving = false;
                            break;
                        }
                    }
                }
            }
            self.animals[i] = a;
        }
        for m in escaped_msgs {
            self.msg(m);
        }
        let mut changed = false;
        for (t, d, i) in broke {
            if self.edge(t, d) != 0 {
                let old = self.edge(t, d);
                self.broken.push((t as u32, d, old));
                self.animal_breaks.push((t as u32, d, old));
                self.set_edge(t, d, 0);
                changed = true;
                let a = &self.animals[i];
                let s = &SPECIES[a.sp as usize];
                let reason = if a.happy < 0.25 { "unhappy" } else { "too strong for the fence" };
                self.messages.push(format!("{} the {} broke through a fence! ({})", ANIMAL_NAMES[a.name as usize], s.name, reason));
            }
        }
        if changed {
            self.recompute_regions();
        }
        for (sp, x, y) in babies {
            let male = self.rng.chance(0.5);
            let id = self.spawn_animal(sp, x, y, male, 0.0);
            let nm = ANIMAL_NAMES[(id as usize * 7) % ANIMAL_NAMES.len()];
            if let Some(a) = self.animals.last_mut() {
                a.happy = 0.8;
            }
            self.baby_born = true;
            self.msg(format!("A baby {} named {} was born!", SPECIES[sp as usize].name, nm));
        }
    }

    fn spawn_guests(&mut self) {
        let guests = self.people.iter().filter(|p| p.kind == 2).count();
        let has_animals = self.animals.iter().any(|a| !a.escaped);
        let willing = 12.0 + self.fame * 8.0;
        let price_f = (2.0 - self.admission / willing).clamp(0.0, 1.3);
        let mut base = if has_animals { 0.12 + self.fame * 0.22 } else { 0.03 };
        let mut cap = 60.0 + self.fame * 70.0;
        if self.fx_wonderland {
            base *= 1.6;
            cap *= 1.4;
        }
        if guests as f32 >= cap {
            return;
        }
        self.guest_acc += base * price_f;
        while self.guest_acc >= 1.0 {
            self.guest_acc -= 1.0;
            let e = self.entrance as usize;
            let id = self.next_id;
            self.next_id += 1;
            let mut p = Person {
                id,
                kind: 2,
                x: tx(e) as f32 + 0.5,
                y: ty(e) as f32 + 0.5,
                target: -1,
                target_obj: -1,
                hunger: 0.6 + self.rng.f() * 0.4,
                thirst: 0.6 + self.rng.f() * 0.4,
                bladder: 0.6 + self.rng.f() * 0.4,
                energy: 0.8 + self.rng.f() * 0.2,
                fun: 0.5,
                happy: 0.7,
                cash: 40.0 + self.rng.f() * 100.0,
                patience: 140.0 + self.rng.f() * 160.0,
                variant: self.rng.range(8) as u8,
                ox: (self.rng.f() - 0.5) * 0.4,
                oy: (self.rng.f() - 0.5) * 0.4,
                prev_tile: e as u32,
                face: 1.0,
                ..Default::default()
            };
            let fee = (self.admission * if self.fx_double_adm { 2.0 } else { 1.0 }).min(p.cash);
            p.cash -= fee;
            p.spent += fee;
            self.cash += fee as f64;
            self.ledger.admissions += fee as f64;
            p.name = format!("{} {}", FIRST_NAMES[self.rng.range(40) as usize], LAST_NAMES[self.rng.range(26) as usize]);
            if self.admission > willing * 1.3 {
                p.thoughts.push(8);
                p.happy -= 0.15;
            }
            self.total_visitors += 1;
            self.people.push(p);
        }
    }

    fn near_obj(&self, t: usize, k: u8, rad: i32) -> bool {
        let (x, y) = (tx(t) as i32, ty(t) as i32);
        for dy in -rad..=rad {
            for dx in -rad..=rad {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && nx < W as i32 && ny < H as i32 && self.obj[tidx(nx as usize, ny as usize)] == k {
                    return true;
                }
            }
        }
        false
    }

    fn adjacent_obj(&self, t: usize, k: u8) -> Option<usize> {
        if self.obj[t] == k {
            return Some(t);
        }
        for d in 0..4u8 {
            if let Some(n) = self.neighbor(t, d) {
                if self.obj[n] == k && self.edge(t, d) == 0 {
                    return Some(n);
                }
            }
        }
        None
    }

    fn update_people(&mut self, second: bool) {
        let n = self.people.len();
        let mut remove = vec![];
        // Escaped animal positions for panic
        let escaped: Vec<(f32, f32, u8)> = self.animals.iter().filter(|a| a.escaped).map(|a| (a.x, a.y, a.sp)).collect();
        for i in 0..n {
            let mut p = std::mem::take(&mut self.people[i]);
            let done = match p.kind {
                2 => self.update_guest(&mut p, second, &escaped),
                3 => {
                    self.update_keeper(&mut p);
                    false
                }
                _ => {
                    self.update_maint(&mut p);
                    false
                }
            };
            self.people[i] = p;
            if done {
                remove.push(i);
            }
        }
        for &i in remove.iter().rev() {
            self.people.swap_remove(i);
        }
    }

    /// Move along path. Returns true when path finished.
    fn follow(&self, p: &mut Person, speed: f32) -> bool {
        if p.pi >= p.path.len() {
            return true;
        }
        let t = p.path[p.pi] as usize;
        let (gx, gy) = (tx(t) as f32 + 0.5 + p.ox, ty(t) as f32 + 0.5 + p.oy);
        let (dx, dy) = (gx - p.x, gy - p.y);
        let d = (dx * dx + dy * dy).sqrt();
        let st = speed * DT;
        if d <= st {
            p.x = gx;
            p.y = gy;
            p.prev_tile = self.tile_of(p.x, p.y) as u32;
            p.pi += 1;
            return p.pi >= p.path.len();
        }
        p.x += dx / d * st;
        p.y += dy / d * st;
        if dx.abs() > 0.01 {
            p.face = dx.signum();
        }
        false
    }

    fn think(p: &mut Person, th: u8) {
        if p.thoughts.last() == Some(&th) {
            return;
        }
        p.thoughts.push(th);
        if p.thoughts.len() > 6 {
            p.thoughts.remove(0);
        }
    }

    fn update_guest(&mut self, p: &mut Person, second: bool, escaped: &[(f32, f32, u8)]) -> bool {
        p.visit += DT;
        p.hunger = (p.hunger - DT / 110.0).max(0.0);
        p.thirst = (p.thirst - DT / 90.0).max(0.0);
        p.bladder = (p.bladder - DT / 130.0).max(0.0);
        p.energy = (p.energy - DT / 200.0).max(0.0);
        p.fun = (p.fun - DT / 120.0).max(0.0);
        let t = self.tile_of(p.x, p.y);
        if p.trash > 0.0 {
            p.trash += DT;
            if self.near_obj(t, O_TRASH, 2) {
                p.trash = 0.0;
            } else if p.trash > 25.0 {
                if self.terrain[t] == T_PATH && self.litter[t] < 5 {
                    self.litter[t] += 1;
                }
                p.trash = 0.0;
            }
        }
        if second {
            let needs = (p.hunger + p.thirst + p.bladder + p.energy) / 4.0;
            let mut env = self.beauty[t] * 0.15;
            let mut lit = 0u32;
            for d in 0..4u8 {
                if let Some(nn) = self.neighbor(t, d) {
                    lit += self.litter[nn] as u32;
                }
            }
            lit += self.litter[t] as u32;
            if lit > 2 {
                env -= 0.15;
                if self.rng.chance(0.1) {
                    Self::think(p, 7);
                }
            }
            let mut scared = false;
            for &(ax, ay, _) in escaped {
                if (ax - p.x).abs() < 5.0 && (ay - p.y).abs() < 5.0 {
                    scared = true;
                }
            }
            if scared {
                env -= 0.4;
                Self::think(p, 9);
            }
            if self.beauty[t] > 0.5 && self.rng.chance(0.03) {
                Self::think(p, 15);
            }
            let target = clamp01(0.45 * needs + 0.45 * p.fun + 0.1 + env);
            p.happy += (target - p.happy) * 0.08;
            if scared && p.state != 4 && self.rng.chance(0.2) {
                self.guest_leave(p);
            }
        }
        match p.state {
            1 => {
                // heading to building
                if self.follow(p, 1.3) {
                    let k = p.target_obj as u8;
                    let tt = self.tile_of(p.x, p.y);
                    if self.adjacent_obj(tt, k).is_some() {
                        p.state = 2;
                        p.timer = if k == O_BENCH { 8.0 } else { 3.0 };
                    } else {
                        p.state = 0;
                    }
                }
            }
            2 => {
                p.timer -= DT;
                if p.timer <= 0.0 {
                    let k = p.target_obj as u8;
                    match k {
                        O_FOOD => {
                            if p.cash >= 6.0 {
                                p.cash -= 6.0;
                                p.spent += 6.0;
                                self.cash += 6.0;
                                self.ledger.concessions += 6.0;
                                p.hunger = 1.0;
                                p.trash = 0.01;
                                Self::think(p, 10);
                            }
                        }
                        O_DRINK => {
                            if p.cash >= 3.0 {
                                p.cash -= 3.0;
                                p.spent += 3.0;
                                self.cash += 3.0;
                                self.ledger.concessions += 3.0;
                                p.thirst = 1.0;
                                p.trash = 0.01;
                                Self::think(p, 11);
                            }
                        }
                        O_RESTROOM => {
                            p.bladder = 1.0;
                            Self::think(p, 12);
                        }
                        O_BENCH => {
                            p.energy = (p.energy + 0.6).min(1.0);
                            Self::think(p, 13);
                        }
                        O_GIFT => {
                            if p.cash >= 12.0 {
                                p.cash -= 12.0;
                                p.spent += 12.0;
                                self.cash += 12.0;
                                self.ledger.gifts += 12.0;
                                p.fun = (p.fun + 0.25).min(1.0);
                                Self::think(p, 14);
                            }
                        }
                        _ => {}
                    }
                    p.state = 0;
                    p.target_obj = -1;
                }
            }
            3 => {
                // viewing
                p.timer -= DT;
                if p.timer <= 0.0 {
                    p.state = 0;
                }
            }
            4 => {
                if self.follow(p, 1.3) {
                    let tt = self.tile_of(p.x, p.y);
                    if tt == self.entrance as usize {
                        return true;
                    }
                    // path got broken, retry
                    let e = self.entrance as usize;
                    match self.bfs(tt, 0, |_, c| c == e) {
                        Some(path) if !path.is_empty() => {
                            p.path = path;
                            p.pi = 0;
                        }
                        _ => return true,
                    }
                }
            }
            _ => {
                // wandering
                if self.follow(p, 1.1) {
                    self.guest_decide(p);
                }
            }
        }
        false
    }

    fn guest_leave(&mut self, p: &mut Person) {
        let t = self.tile_of(p.x, p.y);
        let e = self.entrance as usize;
        p.state = 4;
        p.pi = 0;
        p.path = self.bfs(t, 0, |_, c| c == e).unwrap_or_default();
        if p.path.is_empty() && t != e {
            // stranded off-path: walk directly
            p.path = vec![e as u32];
        }
        if p.happy > 0.7 {
            Self::think(p, 18);
        }
    }

    fn guest_decide(&mut self, p: &mut Person) {
        let t = self.tile_of(p.x, p.y);
        // leave?
        if p.visit > p.patience || p.happy < 0.15 || (p.energy < 0.1) {
            if p.energy < 0.1 {
                Self::think(p, 19);
            } else if p.happy < 0.15 && self.species_count() < 2 {
                Self::think(p, 16);
            }
            self.guest_leave(p);
            return;
        }
        if p.cash < 2.0 && p.visit > 60.0 {
            Self::think(p, 17);
            self.guest_leave(p);
            return;
        }
        // needs
        let wants: [(f32, u8, u8, f32); 5] = [
            (p.hunger, O_FOOD, 1, 0.35),
            (p.thirst, O_DRINK, 2, 0.35),
            (p.bladder, O_RESTROOM, 3, 0.3),
            (p.energy, O_BENCH, 4, 0.35),
            (if p.cash > 30.0 && p.visit > 60.0 { p.fun + 0.2 } else { 1.0 }, O_GIFT, 0, 0.3),
        ];
        let mut best: Option<(f32, u8, u8)> = None;
        for &(v, k, th, thr) in wants.iter() {
            if v < thr && best.map_or(true, |b| v < b.0) {
                best = Some((v, k, th));
            }
        }
        if let Some((v, k, th)) = best {
            if let Some(path) = self.bfs(t, 0, |g, c| g.adjacent_obj(c, k).is_some()) {
                p.path = path;
                p.pi = 0;
                p.state = 1;
                p.target_obj = k as i32;
                return;
            } else if th > 0 {
                Self::think(p, th);
                if v < 0.08 {
                    p.happy -= 0.2;
                    if k == O_RESTROOM {
                        p.bladder = 0.5;
                    }
                    if k == O_FOOD {
                        p.hunger = 0.3;
                    }
                    if k == O_DRINK {
                        p.thirst = 0.3;
                    }
                    if k == O_BENCH {
                        p.energy = 0.3;
                    }
                }
            }
        }
        // view exhibit
        let ve = self.view_enc[t];
        if ve > 0 {
            let r = (ve - 1) as usize;
            if r < self.encs.len() && self.encs[r].n_animals > 0 {
                let now = p.visit;
                let last = p.seen.iter().find(|s| s.0 as usize == r).map(|s| s.1);
                if last.map_or(true, |l| now - l > 70.0) {
                    let e = &self.encs[r];
                    let first = last.is_none();
                    let gain = (0.12 + e.attraction * 0.22).min(0.55) * if first { 1.0 } else { 0.35 };
                    p.fun = (p.fun + gain).min(1.0);
                    let avg = e.avg_happy;
                    let att = e.attraction;
                    let mut mask = 0u32;
                    for s in 0..NS {
                        if e.sp_count[s] > 0 {
                            mask |= 1 << s;
                        }
                    }
                    p.species_seen |= mask;
                    if let Some(s) = p.seen.iter_mut().find(|s| s.0 as usize == r) {
                        s.1 = now;
                    } else {
                        p.seen.push((r as u16, now));
                    }
                    if avg > 0.75 {
                        Self::think(p, if self.rng.chance(0.5) { 5 } else { 21 });
                    } else if avg < 0.4 {
                        Self::think(p, 6);
                        p.happy -= 0.05;
                    }
                    // donation
                    if first && self.near_obj(t, O_DONATION, 3) && p.cash > 5.0 {
                        let amt = (1.0 + att * 3.0 * avg).min(p.cash * 0.3) * if self.fx_double_don { 2.0 } else { 1.0 };
                        p.cash -= amt;
                        p.spent += amt;
                        self.cash += amt as f64;
                        self.ledger.donations += amt as f64;
                        Self::think(p, 22);
                    }
                    p.state = 3;
                    p.timer = 3.0 + self.rng.f() * 3.0;
                    // face the exhibit
                    p.face = if self.encs[r].sx > p.x { 1.0 } else { -1.0 };
                    return;
                }
            }
        }
        if p.visit > 90.0 && p.species_seen.count_ones() < 2 && self.rng.chance(0.05) {
            Self::think(p, 20);
        }
        // wander to neighbor path tile
        let mut opts: [usize; 4] = [0; 4];
        let mut no = 0;
        for d in 0..4u8 {
            if let Some(nn) = self.neighbor(t, d) {
                if self.edge(t, d) == 0 && self.walkable(nn, 0) && nn as u32 != p.prev_tile {
                    opts[no] = nn;
                    no += 1;
                }
            }
        }
        let next = if no > 0 {
            // prefer continuing straight
            opts[self.rng.range(no as u32) as usize]
        } else if self.walkable(p.prev_tile as usize, 0) {
            p.prev_tile as usize
        } else {
            // off-path (e.g. path deleted): head to the nearest path
            match self.bfs(t, 1, |g, c| g.terrain[c] == T_PATH) {
                Some(path) if !path.is_empty() => {
                    p.path = path;
                    p.pi = 0;
                    return;
                }
                _ => {
                    p.visit = p.patience + 1.0;
                    return;
                }
            }
        };
        p.prev_tile = t as u32;
        p.path = vec![next as u32];
        p.pi = 0;
    }

    fn update_keeper(&mut self, p: &mut Person) {
        match p.state {
            0 => {
                p.timer -= DT;
                if p.timer > 0.0 {
                    return;
                }
                p.timer = 1.0;
                let t = self.tile_of(p.x, p.y);
                // pick enclosure with greatest need
                let mut best = -1i32;
                let mut bs = 0.0;
                for (r, e) in self.encs.iter().enumerate() {
                    if e.outside || e.n_animals == 0 || e.claimed >= 0 || e.gates.is_empty() {
                        continue;
                    }
                    let need_food = e.n_animals as f32 * 3.0;
                    let mut score = 0.0;
                    if e.food < need_food * 0.5 {
                        score += 2.0 + (need_food - e.food) / need_food;
                    }
                    if e.dung > 2 {
                        score += e.dung as f32 * 0.1;
                    }
                    if score > bs {
                        bs = score;
                        best = r as i32;
                    }
                }
                if best >= 0 {
                    let rr = best as u16;
                    if let Some(path) = self.bfs(t, 2, |g, c| g.region[c] == rr) {
                        self.encs[best as usize].claimed = p.id as i32;
                        p.target = best;
                        p.path = path;
                        p.pi = 0;
                        p.state = 1;
                        return;
                    }
                }
                // wander a little
                if self.rng.chance(0.3) {
                    let d = self.rng.range(4) as u8;
                    if let Some(nn) = self.neighbor(t, d) {
                        if self.edge(t, d) == 0 && self.walkable(nn, 1) && self.region[nn] == self.outside {
                            p.path = vec![nn as u32];
                            p.pi = 0;
                            p.state = 5;
                        }
                    }
                }
            }
            1 => {
                if self.follow(p, 1.4) {
                    p.state = 2;
                    p.timer = 0.0;
                }
            }
            2 => {
                // inside: find dung
                if p.pi < p.path.len() {
                    self.follow(p, 1.2);
                    return;
                }
                let t = self.tile_of(p.x, p.y);
                if self.dung[t] > 0 {
                    p.timer += DT;
                    if p.timer > 1.2 {
                        self.dung[t] = 0;
                        p.timer = 0.0;
                    }
                    return;
                }
                let rr = p.target as u16;
                if let Some(path) = self.bfs(t, 1, |g, c| g.region[c] == rr && g.dung[c] > 0) {
                    p.path = path;
                    p.pi = 0;
                } else {
                    p.state = 3;
                    p.timer = 3.0;
                }
            }
            3 => {
                p.timer -= DT;
                if p.timer <= 0.0 {
                    let r = p.target as usize;
                    if r < self.encs.len() {
                        self.encs[r].food = self.encs[r].n_animals as f32 * 6.0 + 4.0;
                        self.encs[r].claimed = -1;
                    }
                    let t = self.tile_of(p.x, p.y);
                    let out = self.outside;
                    p.path = self.bfs(t, 2, |g, c| g.region[c] == out && g.terrain[c] == T_PATH)
                        .or_else(|| self.bfs(t, 2, |g, c| g.region[c] == out))
                        .unwrap_or_default();
                    p.pi = 0;
                    p.state = 4;
                }
            }
            4 => {
                if self.follow(p, 1.4) {
                    p.state = 0;
                    p.target = -1;
                }
            }
            _ => {
                if self.follow(p, 0.9) {
                    p.state = 0;
                }
            }
        }
    }

    fn update_maint(&mut self, p: &mut Person) {
        match p.state {
            0 => {
                p.timer -= DT;
                if p.timer > 0.0 {
                    return;
                }
                p.timer = 1.0;
                let t = self.tile_of(p.x, p.y);
                if let Some(path) = self.bfs(t, 1, |g, c| g.litter[c] > 0) {
                    p.path = path;
                    p.pi = 0;
                    p.state = 1;
                    return;
                }
                let d = self.rng.range(4) as u8;
                if let Some(nn) = self.neighbor(t, d) {
                    if self.edge(t, d) == 0 && self.walkable(nn, 0) {
                        p.path = vec![nn as u32];
                        p.pi = 0;
                        p.state = 3;
                    }
                }
            }
            1 => {
                if self.follow(p, 1.3) {
                    p.state = 2;
                    p.timer = 1.5;
                }
            }
            2 => {
                p.timer -= DT;
                if p.timer <= 0.0 {
                    let t = self.tile_of(p.x, p.y);
                    self.litter[t] = 0;
                    p.state = 0;
                }
            }
            _ => {
                if self.follow(p, 0.9) {
                    p.state = 0;
                }
            }
        }
    }

    // ---------- Reporting (JSON) ----------

    pub fn date_string(&self) -> String {
        let day = (self.time / DAY_SECS) as i64;
        let m = day / DAYS_PER_MONTH as i64;
        let d = day % DAYS_PER_MONTH as i64;
        format!("{} {}, Year {}", MONTHS[(m % 12) as usize], d * 5 + 1, m / 12 + 1)
    }

    pub fn stats_json(&mut self) -> String {
        let guests = self.people.iter().filter(|p| p.kind == 2).count();
        let keepers = self.people.iter().filter(|p| p.kind == 3).count();
        let maint = self.people.iter().filter(|p| p.kind == 4).count();
        let mut s = String::with_capacity(1024);
        let _ = write!(
            s,
            "{{\"cash\":{:.0},\"fame\":{:.3},\"guests\":{},\"animals\":{},\"species\":{},\"keepers\":{},\"maint\":{},\"date\":\"{}\",\"admission\":{:.0},\"guestH\":{:.3},\"animalH\":{:.3},\"variety\":{:.3},\"visitors\":{},\"goals\":{},\"dayFrac\":{:.4},\"unlocked\":{},\"tint\":{},\"birds\":{:.1},\"own\":[{},{},{},{}],\"expand\":[{},{},{}],",
            self.cash, self.fame, guests, self.animals.len(), self.species_count(), keepers, maint, self.date_string(),
            self.admission, self.guest_h, self.animal_h, self.variety, self.total_visitors, self.goals_done,
            (self.time / DAY_SECS).fract(), self.unlocked, self.tint, self.birds,
            self.own[0], self.own[1], self.own[2], self.own[3], self.expand_cost(0), self.expand_cost(1), self.expand_cost(2)
        );
        let l = &self.last_ledger;
        let c = &self.ledger;
        let _ = write!(
            s,
            "\"ledger\":[{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0}],\"cur\":[{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0}],",
            l.admissions, l.concessions, l.donations, l.gifts, l.salaries, l.upkeep, l.construction, l.animals,
            c.admissions, c.concessions, c.donations, c.gifts, c.salaries, c.upkeep, c.construction, c.animals
        );
        s.push_str("\"msgs\":[");
        let msgs = std::mem::take(&mut self.messages);
        for (i, m) in msgs.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            push_str_json(&mut s, m);
        }
        s.push_str("]}");
        s
    }

    pub fn defs_json(&self) -> String {
        let mut s = String::from("{\"terrain\":[");
        for i in 0..NT {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{{\"name\":\"{}\",\"cost\":{}}}", TERRAIN_NAMES[i], TERRAIN_COST[i]);
        }
        s.push_str("],\"objects\":[");
        for (i, o) in OBJS.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{{\"name\":\"{}\",\"cost\":{},\"cat\":{},\"biome\":{},\"blocks\":{}}}", o.name, o.cost, o.cat, o.biome, o.blocks);
        }
        s.push_str("],\"species\":[");
        for (i, sp) in SPECIES.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(
                s,
                "{{\"hidden\":{},\"name\":\"{}\",\"plural\":\"{}\",\"cost\":{},\"fame\":{},\"biome\":\"{}\",\"foliage\":{},\"strength\":{},\"swims\":{},\"predator\":{},\"group\":[{},{}],\"space\":{},\"terrain\":[",
                i >= NS_PUBLIC, sp.name, sp.plural, sp.cost, sp.fame, sp.biome_label, sp.foliage, sp.strength, sp.swims, sp.predator, sp.group_min, sp.group_max, sp.space
            );
            for j in 0..NT {
                if j > 0 {
                    s.push(',');
                }
                let _ = write!(s, "{}", sp.terrain[j]);
            }
            s.push_str("]}");
        }
        s.push_str("],\"fences\":[");
        for i in 0..4 {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{{\"name\":\"{}\",\"cost\":{}}}", FENCE_NAMES[i], FENCE_COST[i]);
        }
        s.push_str("],\"goals\":[");
        for (i, (g, r)) in GOALS.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{{\"name\":\"{}\",\"reward\":{}}}", g, r);
        }
        s.push_str("]}");
        s
    }

    pub fn pick(&self, fx: f32, fy: f32) -> Option<u32> {
        let mut best = None;
        let mut bd = 0.55f32;
        for a in self.animals.iter() {
            let d = ((a.x - fx).powi(2) + (a.y - fy).powi(2)).sqrt();
            if d < bd {
                bd = d;
                best = Some(a.id);
            }
        }
        for p in self.people.iter() {
            let d = ((p.x - fx).powi(2) + (p.y - fy).powi(2)).sqrt();
            if d < bd * 0.8 {
                bd = d;
                best = Some(p.id);
            }
        }
        best
    }

    pub fn info_entity(&self, id: u32) -> String {
        let mut s = String::with_capacity(512);
        if let Some(a) = self.animals.iter().find(|a| a.id == id) {
            let sp = &SPECIES[a.sp as usize];
            let t = self.tile_of(a.x, a.y);
            let r = self.region[t] as usize;
            let su = self.suit(r, a.sp as usize);
            let e = &self.encs[r];
            let _ = write!(
                s,
                "{{\"type\":\"animal\",\"id\":{},\"name\":\"{}\",\"species\":\"{}\",\"sp\":{},\"male\":{},\"adult\":{},\"age\":{:.1},\"happy\":{:.3},\"hunger\":{:.3},\"thirst\":{:.3},\"escaped\":{},\"x\":{:.2},\"y\":{:.2},",
                a.id, ANIMAL_NAMES[a.name as usize], sp.name, a.sp, a.male, a.age >= ADULT_AGE, a.age / (DAY_SECS as f32), a.happy, a.hunger, a.thirst, a.escaped, a.x, a.y
            );
            let _ = write!(
                s,
                "\"suit\":{{\"terrain\":{:.2},\"foliage\":{:.2},\"space\":{:.2},\"social\":{:.2},\"shelter\":{:.2},\"toys\":{:.2},\"clean\":{:.2},\"water\":{:.2},\"safety\":{:.2},\"total\":{:.2}}},",
                su.terrain, su.foliage, su.space, su.social, su.shelter, su.toys, su.clean, su.water, su.safety, su.total
            );
            // tips
            s.push_str("\"tips\":[");
            let mut tips: Vec<String> = vec![];
            if a.escaped {
                tips.push("ESCAPED! Recapture this animal before guests panic.".into());
            } else {
                if su.terrain < 0.75 {
                    let area = e.area.max(1) as f32;
                    let mut worst = 0usize;
                    let mut wd = 0.0;
                    for i in 0..NT {
                        let d = sp.terrain[i] - e.terr[i] as f32 / area;
                        if d > wd {
                            wd = d;
                            worst = i;
                        }
                    }
                    let mut over = 0usize;
                    let mut od = 0.0;
                    for i in 0..NT {
                        let d = e.terr[i] as f32 / area - sp.terrain[i];
                        if d > od {
                            od = d;
                            over = i;
                        }
                    }
                    if wd > 0.08 {
                        tips.push(format!("Wants more {} terrain ({:.0}% ideal).", TERRAIN_NAMES[worst], sp.terrain[worst] * 100.0));
                    }
                    if od > 0.15 {
                        tips.push(format!("Too much {} in the exhibit.", TERRAIN_NAMES[over]));
                    }
                }
                if su.foliage < 0.7 {
                    tips.push(format!("Wants more {} foliage.", biome_names(sp.foliage)));
                }
                if su.space < 0.8 {
                    tips.push("The exhibit is too crowded. Make it bigger.".into());
                }
                let c = e.sp_count[a.sp as usize];
                if c < sp.group_min {
                    tips.push(format!("Lonely. Prefers a group of {}-{}.", sp.group_min, sp.group_max));
                } else if c > sp.group_max {
                    tips.push(format!("Too many {} in one exhibit (max {}).", sp.plural, sp.group_max));
                }
                if su.water < 1.0 {
                    tips.push("Needs fresh water or a water trough.".into());
                }
                if su.shelter < 1.0 {
                    tips.push("Would like an animal shelter.".into());
                }
                if su.toys < 1.0 {
                    tips.push("Bored. Add an enrichment toy.".into());
                }
                if su.clean < 0.7 {
                    tips.push("The exhibit is dirty. Hire zookeepers.".into());
                }
                if su.safety < 1.0 {
                    tips.push("Terrified of the predators in this exhibit!".into());
                }
                if a.hunger < 0.3 {
                    tips.push("Hungry! Zookeepers need to bring food.".into());
                }
                if e.min_fence < sp.strength {
                    tips.push(format!("Fence too weak. Needs {}.", FENCE_NAMES[sp.strength as usize]));
                }
                if e.gates.is_empty() {
                    tips.push("This exhibit has no gate. Keepers can't get in.".into());
                }
                if tips.is_empty() {
                    tips.push("This animal is content.".into());
                }
            }
            for (i, tp) in tips.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                push_str_json(&mut s, tp);
            }
            s.push_str("]}");
            return s;
        }
        if let Some(p) = self.people.iter().find(|p| p.id == id) {
            if p.kind == 2 {
                let _ = write!(
                    s,
                    "{{\"type\":\"guest\",\"name\":\"{}\",\"sick\":{},\"id\":{},\"happy\":{:.3},\"hunger\":{:.3},\"thirst\":{:.3},\"bladder\":{:.3},\"energy\":{:.3},\"fun\":{:.3},\"cash\":{:.0},\"spent\":{:.0},\"seen\":{},\"leaving\":{},\"x\":{:.2},\"y\":{:.2},\"thoughts\":[",
                    esc(&p.name), p.sick, p.id, p.happy, p.hunger, p.thirst, p.bladder, p.energy, p.fun, p.cash, p.spent, p.species_seen.count_ones(), p.state == 4, p.x, p.y
                );
                for (i, th) in p.thoughts.iter().rev().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    push_str_json(&mut s, GUEST_THOUGHTS[*th as usize]);
                }
                s.push_str("]}");
            } else {
                let task = if p.kind == 3 {
                    match p.state {
                        0 => "Looking for work",
                        1 => "Walking to an exhibit",
                        2 => "Cleaning up dung",
                        3 => "Putting out food",
                        4 => "Leaving the exhibit",
                        _ => "Strolling",
                    }
                } else {
                    match p.state {
                        0 => "Looking for litter",
                        1 => "Heading to some litter",
                        2 => "Sweeping",
                        _ => "Patrolling the paths",
                    }
                };
                let _ = write!(
                    s,
                    "{{\"type\":\"staff\",\"name\":\"{}\",\"free\":{},\"id\":{},\"role\":\"{}\",\"task\":\"{}\",\"salary\":{},\"x\":{:.2},\"y\":{:.2}}}",
                    esc(&p.name), p.free, p.id, if p.kind == 3 { "Zookeeper" } else { "Maintenance Worker" }, task, if p.free { 0 } else if p.kind == 3 { 300 } else { 200 }, p.x, p.y
                );
            }
            return s;
        }
        "null".into()
    }

    pub fn info_tile(&self, fx: f32, fy: f32) -> String {
        if fx < 0.0 || fy < 0.0 || fx >= W as f32 || fy >= H as f32 {
            return "null".into();
        }
        let t = self.tile_of(fx, fy);
        let r = self.region[t] as usize;
        let e = &self.encs[r];
        let mut s = String::new();
        let _ = write!(
            s,
            "{{\"type\":\"tile\",\"owned\":{},\"x\":{},\"y\":{},\"terrain\":\"{}\",\"object\":\"{}\",\"objId\":{},\"dung\":{},\"litter\":{},\"enclosure\":",
            self.owned(t), tx(t), ty(t), TERRAIN_NAMES[self.terrain[t] as usize], OBJS[self.obj[t] as usize].name, self.obj[t], self.dung[t], self.litter[t]
        );
        if e.outside {
            s.push_str("null}");
            return s;
        }
        let _ = write!(
            s,
            "{{\"id\":{},\"tile\":{},\"name\":\"{}\",\"area\":{},\"animals\":{},\"food\":{:.0},\"dung\":{},\"fence\":\"{}\",\"gates\":{},\"happy\":{:.3},\"attraction\":{:.2},\"water\":{},\"terrain\":[",
            r, e.first_tile, esc(&e.name), e.area, e.n_animals, e.food, e.dung, FENCE_NAMES[e.min_fence as usize], e.gates.len(), e.avg_happy, e.attraction, e.has_water
        );
        for i in 0..NT {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{:.3}", e.terr[i] as f32 / e.area.max(1) as f32);
        }
        s.push_str("],\"species\":[");
        let mut first = true;
        for i in 0..NS {
            if e.sp_count[i] > 0 {
                if !first {
                    s.push(',');
                }
                first = false;
                let _ = write!(s, "[{},{},{:.2}]", i, e.sp_count[i], e.sp_happy[i]);
            }
        }
        // suitability preview for every species
        s.push_str("],\"suit\":[");
        for i in 0..NS {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "{:.2}", self.suit(r, i).total);
        }
        s.push_str("]}}");
        s
    }

    /// Fill entity buffer: 10 floats each.
    pub fn fill_entities(&self, buf: &mut Vec<f32>) {
        buf.clear();
        for a in self.animals.iter() {
            let grow = if a.age >= ADULT_AGE { 1.0 } else { 0.45 + 0.55 * a.age / ADULT_AGE };
            buf.extend_from_slice(&[1.0, a.sp as f32, a.x, a.y, a.face, grow, a.action as f32, a.id as f32, a.happy, if a.escaped { 1.0 } else { 0.0 }]);
        }
        for p in self.people.iter() {
            let walking = match (p.kind, p.state) {
                (2, 2) | (2, 3) => 0.0,
                _ => if p.pi < p.path.len() { 1.0 } else { 0.0 },
            };
            let variant = if p.sick { 8.0 } else { p.variant as f32 };
            buf.extend_from_slice(&[p.kind as f32, variant, p.x, p.y, p.face, 1.0, walking, p.id as f32, p.happy, p.state as f32]);
        }
    }
}

pub fn biome_names(mask: u8) -> String {
    let names = [(B_TEMP, "temperate"), (B_BOREAL, "boreal"), (B_SAV, "savanna"), (B_TROP, "tropical"), (B_DES, "desert"), (B_ASIA, "bamboo/Asian"), (B_POLAR, "tundra")];
    let v: Vec<&str> = names.iter().filter(|(b, _)| mask & b != 0).map(|(_, n)| *n).collect();
    v.join(" or ")
}

pub fn push_str_json(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            c => s.push(c),
        }
    }
    s.push('"');
}

pub fn esc(v: &str) -> String {
    let mut s = String::new();
    push_str_json(&mut s, v);
    s[1..s.len() - 1].to_string()
}
