//! "Zoo Walk" first-person view: a software raycaster rendering into an RGBA buffer.

use crate::defs::*;
use crate::sim::{Game, ADULT_AGE, tidx};

pub const RW: usize = 400;
pub const RH: usize = 240;
const EYE: f32 = 0.42;
const MAXD: f32 = 26.0;

pub struct Walk {
    pub w: usize,
    pub h: usize,
    pub x: f32,
    pub y: f32,
    pub a: f32,
    pub buf: Vec<u32>,
    pub time: f32,
}

#[derive(Clone, Copy)]
struct Slice {
    dist: f32,
    kind: u8, // 0 fence/wall, 1 sprite
    sub: u16,
    u: f32,
    h: f32,
    face: f32,
    extra: f32,
}

impl Walk {
    pub fn new() -> Walk {
        Walk { w: RW, h: RH, x: W as f32 / 2.0 + 0.5, y: H as f32 - 2.0, a: -std::f32::consts::FRAC_PI_2, buf: vec![0; RW * RH], time: 0.0 }
    }

    /// Change the render resolution (e.g. portrait phones get a tall frame).
    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w.clamp(160, 640);
        self.h = h.clamp(120, 720);
        self.buf = vec![0xFF00_0000; self.w * self.h];
    }

    pub fn movement(&mut self, g: &Game, fwd: f32, strafe: f32, turn: f32, dt: f32) {
        self.a += turn * dt * 2.2;
        let (c, s) = (self.a.cos(), self.a.sin());
        let sp = 2.2 * dt;
        let dx = (c * fwd - s * strafe) * sp;
        let dy = (s * fwd + c * strafe) * sp;
        let nx = self.x + dx;
        if self.can_move(g, self.x, self.y, nx, self.y) {
            self.x = nx;
        }
        let ny = self.y + dy;
        if self.can_move(g, self.x, self.y, self.x, ny) {
            self.y = ny;
        }
    }

    fn can_move(&self, g: &Game, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        let m = 0.15;
        if x1 < m || y1 < m || x1 > W as f32 - m || y1 > H as f32 - m {
            return false;
        }
        let (ax, ay) = (x0 as usize, y0 as usize);
        // check with margin in movement direction
        let px = x1 + if x1 > x0 { m } else if x1 < x0 { -m } else { 0.0 };
        let py = y1 + if y1 > y0 { m } else if y1 < y0 { -m } else { 0.0 };
        let (bx, by) = (px.clamp(0.0, W as f32 - 0.01) as usize, py.clamp(0.0, H as f32 - 0.01) as usize);
        let bt = tidx(bx, by);
        if g.terrain[bt] == T_WATER || g.blocks(bt) || !g.owned(bt) {
            return false;
        }
        let at = tidx(ax, ay);
        if bx != ax {
            let d = if bx > ax { 1 } else { 3 };
            let e = g.edge(at, d);
            if e != 0 && e & GATE_BIT == 0 {
                return false;
            }
        }
        if by != ay {
            let d = if by > ay { 2 } else { 0 };
            let e = g.edge(at, d);
            if e != 0 && e & GATE_BIT == 0 {
                return false;
            }
        }
        true
    }

    pub fn render(&mut self, g: &Game, dt: f32) {
        let (rw, rh) = (self.w, self.h);
        self.time += dt;
        let (dirx, diry) = (self.a.cos(), self.a.sin());
        let fov = 0.66f32;
        let (plx, ply) = (-diry * fov, dirx * fov);
        let proj = rw as f32 * 0.5 / fov; // square pixels
        let hor = rh as f32 * 0.5;
        let sky_top = (88, 148, 214);
        let sky_bot = (196, 222, 240);
        let fog = (190, 214, 222);
        let day = (g.time / 60.0).fract() as f32;
        let light = 0.85 + 0.15 * (day * std::f32::consts::TAU).cos().abs();
        // Sky
        for y in 0..(hor as usize) {
            let t = y as f32 / hor;
            let r = lerp(sky_top.0 as f32, sky_bot.0 as f32, t);
            let gg = lerp(sky_top.1 as f32, sky_bot.1 as f32, t);
            let b = lerp(sky_top.2 as f32, sky_bot.2 as f32, t);
            for x in 0..rw {
                // simple clouds
                let ang = self.a + (x as f32 / rw as f32 - 0.5) * 1.2;
                let c = ((ang * 3.0 + self.time * 0.01).sin() * (ang * 7.3).cos() + (y as f32 * 0.09 + ang * 2.0).sin() * 0.5) * (1.0 - t);
                let cl = if c > 0.55 { ((c - 0.55) * 2.5).min(0.7) } else { 0.0 };
                self.buf[y * rw + x] = rgb(lerp(r, 255.0, cl), lerp(gg, 255.0, cl), lerp(b, 255.0, cl));
            }
        }
        // Distant treeline silhouette
        for x in 0..rw {
            let ang = self.a + (x as f32 / rw as f32 - 0.5) * 1.2;
            let hgt = 6.0 + 5.0 * ((ang * 9.0).sin() * 0.5 + (ang * 23.0).sin() * 0.3 + (ang * 51.0).sin() * 0.2).abs();
            let top = (hor - hgt) as usize;
            for y in top..(hor as usize) {
                self.buf[y * rw + x] = rgb(96.0, 128.0, 104.0);
            }
        }
        // Floor
        for y in (hor as usize)..rh {
            let p = y as f32 - hor + 0.5;
            let row_dist = EYE * proj / p;
            let fogf = (row_dist / MAXD).min(1.0).powf(1.3);
            let (rdx0, rdy0) = (dirx - plx, diry - ply);
            let (rdx1, rdy1) = (dirx + plx, diry + ply);
            let stepx = row_dist * (rdx1 - rdx0) / rw as f32;
            let stepy = row_dist * (rdy1 - rdy0) / rw as f32;
            let mut fx = self.x + row_dist * rdx0;
            let mut fy = self.y + row_dist * rdy0;
            for x in 0..rw {
                let (r, gg, b) = if fx >= 0.0 && fy >= 0.0 && fx < W as f32 && fy < H as f32 {
                    let t = tidx(fx as usize, fy as usize);
                    let c = ground_color(g, t, fx.fract(), fy.fract(), self.time);
                    if g.owned(t) { c } else { (c.0 * 0.72, c.1 * 0.78, c.2 * 0.72) }
                } else {
                    (86.0, 140.0, 70.0)
                };
                self.buf[y * rw + x] = rgb(
                    lerp(r * light, fog.0 as f32, fogf),
                    lerp(gg * light, fog.1 as f32, fogf),
                    lerp(b * light, fog.2 as f32, fogf),
                );
                fx += stepx;
                fy += stepy;
            }
        }
        // Sprites list
        struct Spr {
            x: f32,
            y: f32,
            kind: u16,
            w: f32,
            h: f32,
            face: f32,
            extra: f32,
        }
        let mut sprites: Vec<Spr> = vec![];
        let r2 = MAXD * MAXD;
        let x0 = (self.x - MAXD).max(0.0) as usize;
        let x1 = ((self.x + MAXD) as usize).min(W - 1);
        let y0 = (self.y - MAXD).max(0.0) as usize;
        let y1 = ((self.y + MAXD) as usize).min(H - 1);
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let t = tidx(tx, ty);
                let o = g.obj[t];
                if o == 0 || is_box(o) {
                    continue;
                }
                let (cx, cy) = (tx as f32 + 0.5, ty as f32 + 0.5);
                if (cx - self.x).powi(2) + (cy - self.y).powi(2) > r2 {
                    continue;
                }
                let (w, h) = obj_size(o);
                sprites.push(Spr { x: cx, y: cy, kind: o as u16, w, h, face: 1.0, extra: (t % 7) as f32 });
            }
        }
        for a in g.animals.iter() {
            let grow = if a.age >= ADULT_AGE { 1.0 } else { 0.45 + 0.55 * a.age / ADULT_AGE };
            let (w, h) = animal_size(a.sp);
            sprites.push(Spr { x: a.x, y: a.y, kind: 100 + a.sp as u16, w: w * grow, h: h * grow, face: a.face, extra: a.action as f32 });
        }
        for p in g.people.iter() {
            sprites.push(Spr { x: p.x, y: p.y, kind: 200 + p.kind as u16 * 10 + p.variant as u16, w: 0.22, h: 0.36, face: 1.0, extra: 0.0 });
        }
        let inv = 1.0 / (plx * diry - dirx * ply);
        // Per-column slices
        let mut cols: Vec<Vec<Slice>> = vec![Vec::with_capacity(8); rw];
        for (si, s) in sprites.iter().enumerate() {
            let (sx, sy) = (s.x - self.x, s.y - self.y);
            let tx = inv * (diry * sx - dirx * sy);
            let tz = inv * (-ply * sx + plx * sy);
            if tz < 0.35 {
                continue;
            }
            let scx = (rw as f32 / 2.0) * (1.0 + tx / tz);
            let pw = s.w * proj / tz;
            let l = (scx - pw / 2.0).max(0.0) as i32;
            let r = (scx + pw / 2.0).min(rw as f32 - 1.0) as i32;
            if r < 0 || l >= rw as i32 {
                continue;
            }
            // Facing relative to camera: determine flip by sign of world dx vs camera right vector
            let face = if s.face * plx >= 0.0 { 1.0 } else { -1.0 };
            for c in l.max(0)..=r.min(rw as i32 - 1) {
                let u = (c as f32 - (scx - pw / 2.0)) / pw;
                cols[c as usize].push(Slice { dist: tz, kind: 1, sub: si as u16, u, h: s.h, face, extra: s.extra });
            }
        }
        // Raycast fences & buildings
        for c in 0..rw {
            let camx = 2.0 * c as f32 / rw as f32 - 1.0;
            let rdx = dirx + plx * camx;
            let rdy = diry + ply * camx;
            let mut mx = self.x as i32;
            let mut my = self.y as i32;
            let ddx = if rdx == 0.0 { 1e30 } else { (1.0 / rdx).abs() };
            let ddy = if rdy == 0.0 { 1e30 } else { (1.0 / rdy).abs() };
            let (stx, mut sdx) = if rdx < 0.0 { (-1, (self.x - mx as f32) * ddx) } else { (1, (mx as f32 + 1.0 - self.x) * ddx) };
            let (sty, mut sdy) = if rdy < 0.0 { (-1, (self.y - my as f32) * ddy) } else { (1, (my as f32 + 1.0 - self.y) * ddy) };
            for _ in 0..64 {
                let (side, dist);
                if sdx < sdy {
                    dist = sdx;
                    sdx += ddx;
                    side = 0;
                } else {
                    dist = sdy;
                    sdy += ddy;
                    side = 1;
                }
                if dist > MAXD {
                    break;
                }
                let (pmx, pmy) = (mx, my);
                if side == 0 { mx += stx; } else { my += sty; }
                let inb = |x: i32, y: i32| x >= 0 && y >= 0 && x < W as i32 && y < H as i32;
                // edge crossed
                let ev = if inb(pmx, pmy) {
                    let t = tidx(pmx as usize, pmy as usize);
                    let d = if side == 0 { if stx > 0 { 1 } else { 3 } } else if sty > 0 { 2 } else { 0 };
                    g.edge(t, d)
                } else if inb(mx, my) {
                    let t = tidx(mx as usize, my as usize);
                    let d = if side == 0 { if stx > 0 { 3 } else { 1 } } else if sty > 0 { 0 } else { 2 };
                    g.edge(t, d)
                } else {
                    0
                };
                let hx = self.x + rdx * dist;
                let hy = self.y + rdy * dist;
                let u = if side == 0 { hy.fract() } else { hx.fract() };
                if ev != 0 {
                    let h = match ev & 3 { 1 => 0.26, 2 => 0.3, _ => 0.36 };
                    cols[c].push(Slice { dist, kind: 0, sub: ev as u16 + if side == 1 { 16 } else { 0 }, u, h, face: 0.0, extra: 0.0 });
                }
                if !inb(mx, my) {
                    if mx < -1 || my < -1 || mx > W as i32 || my > H as i32 {
                        break;
                    }
                    continue;
                }
                let t = tidx(mx as usize, my as usize);
                let o = g.obj[t];
                if is_box(o) {
                    let (h, _) = box_style(o);
                    cols[c].push(Slice { dist, kind: 2, sub: o as u16 + if side == 1 { 256 } else { 0 }, u, h, face: 0.0, extra: 0.0 });
                }
            }
        }
        // Draw slices far to near
        for c in 0..rw {
            let col = &mut cols[c];
            col.sort_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap_or(std::cmp::Ordering::Equal));
            for s in col.iter() {
                let base = hor + EYE * proj / s.dist;
                let top = hor + (EYE - s.h) * proj / s.dist;
                let fogf = (s.dist / MAXD).min(1.0).powf(1.3);
                let y0 = top.max(0.0) as i32;
                let y1 = base.min(rh as f32 - 1.0) as i32;
                let span = (base - top).max(0.001);
                for y in y0..=y1 {
                    let v = (y as f32 - top) / span;
                    let col = match s.kind {
                        0 => fence_px(s.sub, s.u, v, s.dist),
                        2 => box_px(s.sub, s.u, v),
                        _ => {
                            let sp = &sprites[s.sub as usize];
                            let u = if s.face < 0.0 { 1.0 - s.u } else { s.u };
                            sprite_px(sp.kind, u, v, s.extra, self.time)
                        }
                    };
                    if let Some((r, gg, b)) = col {
                        self.buf[y as usize * rw + c] = rgb(
                            lerp(r * light, fog.0 as f32, fogf),
                            lerp(gg * light, fog.1 as f32, fogf),
                            lerp(b * light, fog.2 as f32, fogf),
                        );
                    }
                }
            }
        }
        // crosshair-free; small compass marker
        let _ = sky_bot;
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn rgb(r: f32, g: f32, b: f32) -> u32 {
    let r = r.clamp(0.0, 255.0) as u32;
    let g = g.clamp(0.0, 255.0) as u32;
    let b = b.clamp(0.0, 255.0) as u32;
    r | (g << 8) | (b << 16) | 0xFF00_0000
}
fn hash(a: u32) -> f32 {
    let mut x = a.wrapping_mul(0x9E3779B1);
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EBCA77);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

fn ground_color(g: &Game, t: usize, u: f32, v: f32, time: f32) -> (f32, f32, f32) {
    let n = hash(t as u32) * 0.12 + hash(((u * 8.0) as u32) * 31 + (v * 8.0) as u32 + t as u32 * 7) * 0.08;
    let (r, gg, b) = match g.terrain[t] {
        T_GRASS => (92.0, 158.0, 64.0),
        T_SAVANNA => (196.0, 176.0, 96.0),
        T_DIRT => (146.0, 112.0, 74.0),
        T_SAND => (226.0, 206.0, 150.0),
        T_SNOW => (236.0, 242.0, 248.0),
        T_ROCK => (140.0, 136.0, 130.0),
        T_WATER => {
            let w = ((u * 6.0 + time * 1.5).sin() * (v * 5.0 - time).cos()) * 12.0;
            return (60.0 + w, 120.0 + w, 186.0 + w);
        }
        T_PATH => {
            let edge = u < 0.05 || v < 0.05;
            if edge { (170.0, 160.0, 146.0) } else { (206.0, 196.0, 178.0) }
        }
        _ => (78.0, 94.0, 48.0),
    };
    let mut k = 1.0 - n;
    if g.dung[t] > 0 && ((u - 0.4).powi(2) + (v - 0.6).powi(2)) < 0.01 * g.dung[t].min(4) as f32 {
        return (90.0, 64.0, 34.0);
    }
    if g.litter[t] > 0 && ((u - 0.7).powi(2) + (v - 0.3).powi(2)) < 0.006 {
        return (230.0, 230.0, 220.0);
    }
    if g.terrain[t] == T_SAVANNA && hash((u * 20.0) as u32 + t as u32 * 13) > 0.7 {
        k *= 0.85;
    }
    (r * k, gg * k, b * k)
}

fn is_box(o: u8) -> bool {
    matches!(o, O_SHELTER | O_FOOD | O_DRINK | O_RESTROOM | O_GIFT | O_ENTRANCE)
}

fn box_style(o: u8) -> (f32, (f32, f32, f32)) {
    match o {
        O_SHELTER => (0.45, (128.0, 96.0, 64.0)),
        O_FOOD => (0.55, (214.0, 82.0, 52.0)),
        O_DRINK => (0.55, (52.0, 132.0, 214.0)),
        O_RESTROOM => (0.6, (120.0, 160.0, 150.0)),
        O_GIFT => (0.7, (176.0, 92.0, 176.0)),
        _ => (0.9, (110.0, 86.0, 52.0)),
    }
}

fn box_px(sub: u16, u: f32, v: f32) -> Option<(f32, f32, f32)> {
    let o = (sub & 255) as u8;
    let shade = if sub >= 256 { 0.82 } else { 1.0 };
    let (_, c) = box_style(o);
    let mut col = c;
    if v < 0.18 {
        // roof band
        col = (col.0 * 0.6 + 60.0, col.1 * 0.6 + 50.0, col.2 * 0.6 + 40.0);
        if ((u * 8.0) as i32) % 2 == 0 && o != O_SHELTER && o != O_ENTRANCE {
            col = (240.0, 236.0, 220.0);
        }
    } else if o != O_SHELTER && o != O_ENTRANCE && v > 0.35 && v < 0.6 && u > 0.15 && u < 0.85 {
        col = (70.0, 60.0, 50.0); // service window
    } else if o == O_ENTRANCE && (u < 0.15 || u > 0.85) {
        col = (150.0, 120.0, 80.0);
    }
    Some((col.0 * shade, col.1 * shade, col.2 * shade))
}

fn fence_px(sub: u16, u: f32, v: f32, dist: f32) -> Option<(f32, f32, f32)> {
    let ev = (sub & 15) as u8;
    let shade = if sub >= 16 { 0.85 } else { 1.0 };
    let kind = ev & 3;
    let gate = ev & GATE_BIT != 0;
    let post = u < 0.06 || u > 0.94;
    let c = match kind {
        1 => {
            // wood: posts and three rails
            let rail = (v > 0.1 && v < 0.22) || (v > 0.45 && v < 0.57) || (v > 0.8 && v < 0.9);
            if post || rail || (gate && (u - v).abs() < 0.05) { Some((150.0, 108.0, 62.0)) } else { None }
        }
        2 => {
            if post || v < 0.06 {
                Some((150.0, 154.0, 160.0))
            } else {
                let m = 24.0 / dist.max(1.0).sqrt();
                let a = ((u + v) * m).fract();
                let b = ((u - v + 4.0) * m).fract();
                if a < 0.14 || b < 0.14 { Some((176.0, 180.0, 186.0)) } else { None }
            }
        }
        _ => {
            let line = (v * 4.0).fract() < 0.05 || ((u * 3.0 + if ((v * 4.0) as i32) % 2 == 0 { 0.5 } else { 0.0 }).fract() < 0.03);
            if line { Some((150.0, 148.0, 140.0)) } else { Some((190.0, 186.0, 176.0)) }
        }
    };
    let c = if gate && kind != 1 {
        if post || (v > 0.1 && v < 0.2) { Some((120.0, 90.0, 60.0)) } else { c }
    } else { c };
    c.map(|(r, g, b)| (r * shade, g * shade, b * shade))
}

fn obj_size(o: u8) -> (f32, f32) {
    match o {
        1 => (1.1, 1.3),
        2 => (0.8, 1.5),
        3 => (1.3, 1.1),
        4 => (0.9, 1.4),
        5 => (0.7, 0.9),
        6 => (0.5, 0.8),
        7 => (0.6, 0.35),
        8 => (1.4, 1.7),
        9 => (0.6, 0.35),
        10 => (0.7, 0.35),
        11 => (0.6, 0.45),
        12 => (0.7, 0.4),
        13 => (0.7, 1.2),
        15 => (0.25, 0.25),
        16 => (0.6, 0.16),
        20 => (0.5, 0.18),
        21 => (0.2, 0.22),
        23 => (0.25, 0.3),
        24 => (0.6, 0.14),
        25 => (0.7, 0.5),
        26 => (0.12, 0.6),
        _ => (0.5, 0.5),
    }
}

fn animal_size(sp: u8) -> (f32, f32) {
    match sp {
        0 => (0.6, 0.32),
        1 => (0.6, 0.38),
        2 => (0.7, 1.05),
        3 => (1.1, 0.8),
        4 => (0.65, 0.4),
        5 => (0.5, 0.3),
        6 => (0.16, 0.26),
        7 => (0.7, 0.4),
        8 => (0.65, 0.3),
        9 => (0.5, 0.38),
        10 => (0.7, 0.55),
        12 => (0.6, 0.5),
        13 => (1.0, 0.55),
        _ => (0.8, 0.4),
    }
}

fn circ(u: f32, v: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> bool {
    let dx = (u - cx) / rx;
    let dy = (v - cy) / ry;
    dx * dx + dy * dy < 1.0
}

fn sprite_px(kind: u16, u: f32, v: f32, extra: f32, time: f32) -> Option<(f32, f32, f32)> {
    let leaf = |base: (f32, f32, f32)| {
        let n = hash(((u * 14.0) as u32) * 57 + (v * 14.0) as u32 * 13 + extra as u32) * 0.25;
        let s = 1.0 - v * 0.3 - n;
        Some((base.0 * s, base.1 * s, base.2 * s))
    };
    let trunk = Some((110.0, 78.0, 48.0));
    match kind {
        1 => {
            if circ(u, v, 0.5, 0.35, 0.45, 0.3) || circ(u, v, 0.28, 0.5, 0.25, 0.18) || circ(u, v, 0.72, 0.5, 0.25, 0.18) {
                leaf((70.0, 140.0, 50.0))
            } else if (u - 0.5).abs() < 0.06 && v > 0.5 { trunk } else { None }
        }
        2 => {
            let w = (v - 0.02) * 0.48;
            if v < 0.88 && (u - 0.5).abs() < w && ((v * 5.0).fract() > 0.15 || (u - 0.5).abs() < w * 0.8) {
                leaf((40.0, 100.0, 60.0))
            } else if (u - 0.5).abs() < 0.05 && v >= 0.8 { trunk } else { None }
        }
        3 => {
            if circ(u, v, 0.5, 0.22, 0.48, 0.13) {
                leaf((110.0, 140.0, 50.0))
            } else if v > 0.25 && ((u - 0.5 - (0.6 - v) * 0.3).abs() < 0.03 || (u - 0.5 + (0.6 - v) * 0.3).abs() < 0.03) && v < 0.6 || ((u - 0.5).abs() < 0.035 && v >= 0.55) { trunk } else { None }
        }
        4 => {
            let tx = 0.5 + (1.0 - v) * 0.12;
            let fr = v < 0.3 && ((u - 0.5).abs() * 0.6 - (0.18 - (v - 0.15).abs())).abs() < 0.05;
            if fr || circ(u, v, 0.55, 0.12, 0.4, 0.06) { leaf((70.0, 150.0, 60.0)) } else if (u - tx).abs() < 0.04 && v > 0.12 { Some((150.0, 120.0, 80.0)) } else { None }
        }
        5 => {
            let stalks = [0.2, 0.35, 0.5, 0.62, 0.8];
            for (i, s) in stalks.iter().enumerate() {
                if (u - s).abs() < 0.03 && v > 0.05 + i as f32 * 0.04 {
                    return if (v * 9.0).fract() < 0.08 { Some((90.0, 120.0, 40.0)) } else { Some((120.0, 170.0, 60.0)) };
                }
                if circ(u, v, *s + 0.08, 0.15 + i as f32 * 0.1, 0.1, 0.04) {
                    return leaf((80.0, 160.0, 60.0));
                }
            }
            None
        }
        6 => {
            if (u - 0.5).abs() < 0.12 || (v > 0.3 && v < 0.5 && u < 0.3 && u > 0.12) || (u < 0.25 && u > 0.12 && v > 0.15 && v < 0.5) || (v > 0.4 && v < 0.55 && u > 0.7 && u < 0.88) || (u > 0.75 && u < 0.88 && v > 0.25 && v < 0.55) {
                if v < 0.03 { None } else { Some((80.0, 140.0, 80.0)) }
            } else { None }
        }
        7 => if circ(u, v, 0.5, 0.6, 0.5, 0.42) { leaf((60.0, 130.0, 50.0)) } else { None },
        8 => {
            if circ(u, v, 0.5, 0.25, 0.5, 0.22) || circ(u, v, 0.3, 0.35, 0.3, 0.12) { leaf((40.0, 120.0, 40.0)) }
            else if ((u - 0.5).abs() < 0.07 && v > 0.3) || (v > 0.85 && (u - 0.5).abs() < 0.2 - (1.0 - v)) { Some((150.0, 130.0, 110.0)) } else { None }
        }
        9 => if circ(u, v, 0.5, 0.6, 0.5, 0.42) { let s = 1.0 - hash((u * 9.0) as u32 * 7 + (v * 9.0) as u32) * 0.2 - v * 0.2; Some((140.0 * s, 136.0 * s, 128.0 * s)) } else { None },
        10 => {
            let blade = ((u * 9.0).fract() - 0.5).abs() < 0.2 && v > ((u * 9.0) as i32 % 3) as f32 * 0.12;
            if blade { Some((180.0, 170.0, 90.0)) } else { None }
        }
        11 => {
            let dx = u - 0.5;
            if v > 0.25 && (dx.abs() < 0.04 || ((v - 0.3) - dx.abs() * 0.8).abs() < 0.06 && v < 0.7) { Some((60.0, 140.0, 60.0)) } else if circ(u, v, 0.5, 0.2, 0.45, 0.1) { leaf((70.0, 150.0, 70.0)) } else { None }
        }
        12 => if circ(u, v, 0.5, 0.6, 0.5, 0.42) { Some((210.0 - v * 30.0, 228.0 - v * 20.0, 245.0)) } else { None },
        13 => {
            if circ(u, v, 0.5, 0.3, 0.4, 0.28) { leaf((130.0, 170.0, 60.0)) }
            else if (u - 0.5).abs() < 0.05 && v > 0.4 { if (v * 12.0).fract() < 0.15 { Some((40.0, 40.0, 40.0)) } else { Some((236.0, 236.0, 230.0)) } } else { None }
        }
        15 => if circ(u, v, 0.5, 0.5, 0.5, 0.5) { Some(if u < 0.5 { (220.0, 50.0, 50.0) } else { (240.0, 220.0, 60.0) }) } else { None },
        16 => if v > 0.2 { Some(if v < 0.4 { (80.0, 140.0, 210.0) } else { (130.0, 130.0, 130.0) }) } else { None },
        20 => if (v < 0.3) || (v > 0.45 && v < 0.65) || ((u < 0.1 || u > 0.9) && v > 0.5) { Some((140.0, 100.0, 60.0)) } else { None },
        21 => if v > 0.1 { Some((60.0, 110.0, 70.0)) } else { None },
        23 => if v > 0.4 && (u - 0.5).abs() < 0.4 { Some((220.0, 190.0, 60.0)) } else if (u - 0.5).abs() < 0.08 { Some((100.0, 100.0, 100.0)) } else { None },
        24 => if v > 0.3 { let h = hash((u * 28.0) as u32 * 3 + (v * 14.0) as u32 * 101); Some(if h > 0.6 { (230.0, 80.0, 120.0) } else if h > 0.35 { (250.0, 220.0, 80.0) } else { (60.0, 130.0, 50.0) }) } else { None },
        25 => {
            if v > 0.7 { Some((170.0, 170.0, 165.0)) }
            else if (u - 0.5).abs() < 0.1 + (v - 0.1) * 0.25 && v > 0.1 { let w = ((v * 20.0 - time * 8.0).sin()) * 20.0; Some((150.0 + w, 200.0 + w, 240.0)) } else { None }
        }
        26 => if circ(u, v, 0.5, 0.08, 0.5, 0.08) { Some((255.0, 240.0, 180.0)) } else if (u - 0.5).abs() < 0.2 { Some((50.0, 50.0, 55.0)) } else { None },
        100..=199 => animal_px((kind - 100) as u8, u, v, extra, time),
        _ => person_px(kind, u, v),
    }
}

fn person_px(kind: u16, u: f32, v: f32) -> Option<(f32, f32, f32)> {
    let role = (kind - 200) / 10;
    let var = (kind % 10) as usize;
    let shirts = [(220.0, 60.0, 60.0), (60.0, 120.0, 220.0), (240.0, 200.0, 60.0), (80.0, 170.0, 90.0), (200.0, 90.0, 200.0), (240.0, 140.0, 50.0), (240.0, 240.0, 240.0), (60.0, 190.0, 190.0), (230.0, 230.0, 40.0)];
    let skin = [(240.0, 200.0, 160.0), (200.0, 150.0, 110.0), (150.0, 100.0, 70.0), (110.0, 75.0, 50.0)][var % 4];
    let shirt = match role { 3 => (150.0, 130.0, 80.0), 4 => (60.0, 80.0, 140.0), _ => shirts[var % 9] };
    if circ(u, v, 0.5, 0.12, 0.28, 0.11) {
        if role == 3 && v < 0.08 { return Some((120.0, 100.0, 60.0)); }
        return Some(skin);
    }
    if role == 3 && v > 0.06 && v < 0.1 && (u - 0.5).abs() < 0.45 { return Some((120.0, 100.0, 60.0)); }
    if v > 0.22 && v < 0.58 && (u - 0.5).abs() < 0.32 { return Some(shirt); }
    if v >= 0.58 && ((u - 0.35).abs() < 0.11 || (u - 0.65).abs() < 0.11) {
        return Some(if role == 2 { (60.0, 70.0, 110.0) } else { (90.0, 80.0, 60.0) });
    }
    None
}

fn animal_px(sp: u8, u: f32, v: f32, action: f32, time: f32) -> Option<(f32, f32, f32)> {
    let resting = action as u8 == 4;
    let walk = action as u8 == 1;
    let lp = if walk { (time * 9.0).sin() * 0.04 } else { 0.0 };
    let quad = |body: (f32, f32, f32), by: f32, bh: f32, hx: f32, hy: f32, hr: f32, leg: f32| -> Option<(f32, f32, f32)> {
        if circ(u, v, 0.48, by, 0.36, bh) { return Some(body); }
        if circ(u, v, hx, hy, hr, hr * 1.0) { return Some(body); }
        if !resting && v > by && v < 1.0 {
            for (i, lx) in [0.22f32, 0.32, 0.62, 0.72].iter().enumerate() {
                let off = if i % 2 == 0 { lp } else { -lp };
                if (u - lx - off).abs() < leg { return Some((body.0 * 0.8, body.1 * 0.8, body.2 * 0.8)); }
            }
        }
        None
    };
    let rv = if resting { v * 0.8 + 0.2 } else { v };
    let v = rv;
    match sp {
        0 => {
            // lion
            if circ(u, v, 0.83, 0.4, 0.15, 0.2) && !(circ(u, v, 0.86, 0.42, 0.08, 0.1)) { return Some((150.0, 90.0, 40.0)); }
            if circ(u, v, 0.86, 0.42, 0.08, 0.1) { return Some((210.0, 160.0, 90.0)); }
            if u < 0.12 && (v - 0.45 - (0.12 - u)).abs() < 0.03 { return Some((180.0, 130.0, 70.0)); }
            quad((214.0, 168.0, 98.0), 0.55, 0.2, 0.86, 0.42, 0.1, 0.04)
        }
        1 => {
            let r = quad((240.0, 240.0, 240.0), 0.5, 0.2, 0.86, 0.32, 0.09, 0.035);
            if r.is_some() && (u * 22.0 + v * 6.0).sin() > 0.2 { return Some((25.0, 25.0, 25.0)); }
            if (u - 0.8).abs() < 0.04 && v > 0.3 && v < 0.45 { return Some((30.0, 30.0, 30.0)); }
            r
        }
        2 => {
            // giraffe
            let neck = (u - 0.72 - (0.5 - v) * 0.15).abs() < 0.05 && v > 0.08 && v < 0.55;
            let body = circ(u, v, 0.45, 0.55, 0.3, 0.12);
            let head = circ(u, v, 0.82, 0.07, 0.08, 0.05);
            let legs = v > 0.55 && [0.25f32, 0.33, 0.58, 0.66].iter().any(|l| (u - l).abs() < 0.025);
            if neck || body || head || legs {
                let spot = (u * 30.0).sin() * (v * 40.0).sin() > 0.3;
                return Some(if spot { (150.0, 90.0, 40.0) } else { (236.0, 200.0, 120.0) });
            }
            None
        }
        3 => {
            // elephant
            let g = (128.0, 126.0, 124.0);
            if circ(u, v, 0.86, 0.38, 0.12, 0.18) || ((u - 0.95).abs() < 0.04 && v > 0.4 && v < 0.85) { return Some(g); }
            if circ(u, v, 0.74, 0.4, 0.1, 0.2) { return Some((110.0, 108.0, 108.0)); }
            if circ(u, v, 0.45, 0.42, 0.4, 0.3) { return Some(g); }
            if v > 0.6 && [0.2f32, 0.33, 0.55, 0.68].iter().any(|l| (u - l).abs() < 0.06) { return Some((116.0, 114.0, 112.0)); }
            None
        }
        4 => quad((120.0, 82.0, 50.0), 0.5, 0.28, 0.85, 0.42, 0.12, 0.06),
        5 => {
            let r = quad((240.0, 240.0, 236.0), 0.5, 0.28, 0.84, 0.4, 0.13, 0.06);
            if r.is_some() && (v > 0.7 || (u > 0.3 && u < 0.45) || circ(u, v, 0.84, 0.32, 0.04, 0.04) || circ(u, v, 0.88, 0.4, 0.035, 0.04)) { return Some((30.0, 30.0, 30.0)); }
            r
        }
        6 => {
            // penguin
            if circ(u, v, 0.5, 0.5, 0.42, 0.42) {
                return Some(if circ(u, v, 0.6, 0.55, 0.22, 0.32) { (245.0, 245.0, 240.0) } else { (30.0, 30.0, 40.0) });
            }
            if circ(u, v, 0.55, 0.12, 0.25, 0.12) { return Some(if u > 0.68 { (240.0, 160.0, 40.0) } else { (30.0, 30.0, 40.0) }); }
            None
        }
        7 => quad((244.0, 240.0, 222.0), 0.5, 0.26, 0.86, 0.42, 0.12, 0.06),
        8 => {
            let r = quad((230.0, 130.0, 40.0), 0.55, 0.2, 0.86, 0.45, 0.1, 0.04);
            if r.is_some() && (u * 26.0).sin() > 0.6 { return Some((30.0, 20.0, 20.0)); }
            if r.is_some() && v > 0.65 && u > 0.2 && u < 0.75 { return Some((240.0, 230.0, 220.0)); }
            r
        }
        9 => {
            if circ(u, v, 0.45, 0.45, 0.32, 0.3) || circ(u, v, 0.72, 0.25, 0.14, 0.14) { return Some((50.0, 48.0, 52.0)); }
            if v > 0.6 && [0.25f32, 0.62].iter().any(|l| (u - l).abs() < 0.08) { return Some((40.0, 38.0, 42.0)); }
            if (u - 0.8).abs() < 0.06 && v > 0.35 && v < 0.95 { return Some((40.0, 38.0, 42.0)); }
            None
        }
        10 => {
            if circ(u, v, 0.5, 0.28, 0.14, 0.12) { return Some((200.0, 160.0, 100.0)); }
            quad((200.0, 160.0, 100.0), 0.45, 0.17, 0.86, 0.38, 0.08, 0.035)
        }
        11 => quad((140.0, 110.0, 120.0), 0.6, 0.25, 0.86, 0.55, 0.13, 0.08),
        12 => {
            // unicorn: white horse with a golden horn and rainbow mane
            if (u - 0.9 - (0.22 - v) * 0.5).abs() < 0.025 && v > 0.04 && v < 0.24 { return Some((250.0, 210.0, 80.0)); }
            if circ(u, v, 0.74, 0.3, 0.06, 0.14) && u < 0.76 { let k = ((v * 12.0) as i32 % 3) as usize; return Some([(240.0, 120.0, 200.0), (140.0, 180.0, 250.0), (250.0, 230.0, 120.0)][k]); }
            if (u - 0.8).abs() < 0.06 && v > 0.22 && v < 0.5 { return Some((250.0, 250.0, 255.0)); }
            quad((250.0, 250.0, 255.0), 0.52, 0.17, 0.88, 0.27, 0.08, 0.035)
        }
        13 => {
            // triceratops
            if circ(u, v, 0.8, 0.35, 0.09, 0.22) { return Some((120.0, 150.0, 70.0)); }
            if (v > 0.32 && v < 0.38 && u > 0.88 && u < 1.0) || (v > 0.47 && v < 0.52 && u > 0.92) { return Some((240.0, 235.0, 210.0)); }
            let r = quad((96.0, 136.0, 72.0), 0.55, 0.25, 0.9, 0.5, 0.1, 0.06);
            if r.is_some() && v < 0.45 && (u * 18.0).sin() > 0.7 { return Some((80.0, 112.0, 60.0)); }
            r
        }
        _ => None,
    }
}
