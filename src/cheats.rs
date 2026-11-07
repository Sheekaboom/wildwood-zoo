//! Classic Zoo Tycoon-style cheats: Shift+$ money, and secret names for
//! guests, staff and exhibits. Also the command terminal.

use crate::defs::*;
use crate::sim::*;
use std::fmt::Write as _;

/// Who must carry a name for the cheat to fire.
#[derive(Clone, Copy, PartialEq)]
pub enum Who {
    Guest,
    Staff,
    Maint,
    Exhibit,
}

pub struct NameCheat {
    pub name: &'static str,
    pub who: Who,
    pub desc: &'static str,
    pub origin: &'static str,
}

pub const NAME_CHEATS: [NameCheat; 18] = [
    NameCheat { name: "Rosalie", who: Who::Staff, desc: "That staff member works for free", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. Blue", who: Who::Guest, desc: "Clothes and buildings turn blue", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. Brown", who: Who::Guest, desc: "Clothes and buildings turn brown", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. Orange", who: Who::Guest, desc: "Clothes and buildings turn orange", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. Pink", who: Who::Guest, desc: "Clothes and buildings turn pink", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. White", who: Who::Guest, desc: "Clothes and buildings turn white", origin: "Zoo Tycoon" },
    NameCheat { name: "Mr. Blonde", who: Who::Guest, desc: "Clothes and buildings turn yellow", origin: "Zoo Tycoon" },
    NameCheat { name: "Russell C", who: Who::Guest, desc: "Every fence in the zoo breaks!", origin: "Zoo Tycoon" },
    NameCheat { name: "Alfred H", who: Who::Guest, desc: "Birds attack your guests", origin: "Zoo Tycoon" },
    NameCheat { name: "Zeta Psi", who: Who::Guest, desc: "Some guests turn yellow and get sick", origin: "Zoo Tycoon" },
    NameCheat { name: "Blue Fang", who: Who::Exhibit, desc: "Guests pay double admission without noticing", origin: "Zoo Tycoon" },
    NameCheat { name: "Microsoft", who: Who::Exhibit, desc: "Double donations", origin: "Zoo Tycoon" },
    NameCheat { name: "Wonderland", who: Who::Exhibit, desc: "Bonus visitors", origin: "Zoo Tycoon" },
    NameCheat { name: "Xanadu", who: Who::Exhibit, desc: "Unlocks the Unicorn", origin: "Dinosaur Digs" },
    NameCheat { name: "Cretaceous Corral", who: Who::Exhibit, desc: "Unlocks the Triceratops", origin: "Dinosaur Digs" },
    NameCheat { name: "Copralite Corner", who: Who::Exhibit, desc: "Dung in this exhibit turns to rocks", origin: "Dinosaur Digs" },
    NameCheat { name: "Bob V", who: Who::Maint, desc: "All broken fences are repaired", origin: "Dinosaur Digs" },
    NameCheat { name: "George W", who: Who::Maint, desc: "All foliage removed, and you get $3,000", origin: "Dinosaur Digs" },
];

pub fn norm(s: &str) -> String {
    let mut out = String::new();
    let mut sp = false;
    for c in s.trim().chars() {
        if c == '.' || c == '\'' {
            continue;
        }
        if c.is_whitespace() {
            sp = true;
            continue;
        }
        if sp && !out.is_empty() {
            out.push(' ');
        }
        sp = false;
        out.extend(c.to_lowercase());
    }
    out
}

pub fn find_cheat(name: &str) -> Option<usize> {
    let n = norm(name);
    NAME_CHEATS.iter().position(|c| norm(c.name) == n)
}

impl Game {
    /// Shift + $ : +$10,000. As in the original, using it wears down fences.
    pub fn money_cheat(&mut self) {
        self.cash += 10000.0;
        self.msg("Cheat: +$10,000!".into());
        if self.rng.chance(0.15) {
            let mut edges = vec![];
            for t in 0..N {
                for d in [1u8, 2u8] {
                    let e = self.edge(t, d);
                    if e != 0 && self.neighbor(t, d).is_some() {
                        edges.push((t, d, e));
                    }
                }
            }
            if !edges.is_empty() {
                let (t, d, e) = edges[self.rng.range(edges.len() as u32) as usize];
                self.broken.push((t as u32, d, e));
                self.set_edge(t, d, 0);
                self.need_regions = true;
                self.msg("Uh oh... a fence has worn through!".into());
            }
        }
    }

    pub fn refresh_name_effects(&mut self) {
        self.fx_double_adm = false;
        self.fx_double_don = false;
        self.fx_wonderland = false;
        self.fx_copralite.clear();
        for (i, e) in self.encs.iter().enumerate() {
            if e.outside || e.name.is_empty() {
                continue;
            }
            match find_cheat(&e.name).map(|c| NAME_CHEATS[c].name) {
                Some("Blue Fang") => self.fx_double_adm = true,
                Some("Microsoft") => self.fx_double_don = true,
                Some("Wonderland") => self.fx_wonderland = true,
                Some("Copralite Corner") => self.fx_copralite.push(i as u16),
                _ => {}
            }
        }
    }

    pub fn cheat_tick(&mut self) {
        if self.birds > 0.0 {
            self.birds -= 1.0;
            for i in 0..self.people.len() {
                if self.people[i].kind == 2 && self.rng.chance(0.25) {
                    let p = &mut self.people[i];
                    p.happy = (p.happy - 0.05).max(0.0);
                    if p.thoughts.last() != Some(&24) {
                        p.thoughts.push(24);
                    }
                }
            }
        }
        // Sick guests
        for i in 0..self.people.len() {
            if self.people[i].sick && self.rng.chance(0.03) {
                let t = self.tile_of(self.people[i].x, self.people[i].y);
                if self.terrain[t] == T_PATH && self.litter[t] < 5 {
                    self.litter[t] += 1;
                }
                let p = &mut self.people[i];
                p.happy = (p.happy - 0.03).max(0.0);
                if p.thoughts.last() != Some(&25) {
                    p.thoughts.push(25);
                }
            }
        }
        // Copralite Corner: dung becomes rocks
        if !self.fx_copralite.is_empty() {
            let mut changed = false;
            for t in 0..N {
                if self.dung[t] > 0 && self.fx_copralite.contains(&self.region[t]) {
                    self.dung[t] = 0;
                    if self.obj[t] == 0 && self.terrain[t] != T_WATER && !self.animals.iter().any(|a| self.tile_of(a.x, a.y) == t) {
                        self.obj[t] = 9;
                        changed = true;
                    }
                }
            }
            if changed {
                self.need_beauty = true;
            }
        }
    }

    /// Apply the effect of a secret name. Returns a description if it fired.
    fn fire_cheat(&mut self, ci: usize, person: Option<usize>, region: Option<usize>) -> String {
        let c = &NAME_CHEATS[ci];
        let label = c.name;
        match label {
            "Rosalie" => {
                if let Some(i) = person {
                    self.people[i].free = true;
                }
            }
            "Mr. Blue" => self.tint = 1,
            "Mr. Brown" => self.tint = 2,
            "Mr. Orange" => self.tint = 3,
            "Mr. Pink" => self.tint = 4,
            "Mr. White" => self.tint = 5,
            "Mr. Blonde" => self.tint = 6,
            "Russell C" => {
                let mut n = 0;
                for t in 0..N {
                    for d in 0..4u8 {
                        let e = self.edge(t, d);
                        if e != 0 {
                            self.broken.push((t as u32, d, e));
                            self.set_edge(t, d, 0);
                            n += 1;
                        }
                    }
                }
                self.recompute_regions();
                if n == 0 {
                    return "Russell C: there are no fences to break (yet).".into();
                }
            }
            "Alfred H" => self.birds = 45.0,
            "Zeta Psi" => {
                for i in 0..self.people.len() {
                    if self.people[i].kind == 2 && self.rng.chance(0.35) {
                        self.people[i].sick = true;
                    }
                }
            }
            "Xanadu" => self.unlocked |= 1 << 12,
            "Cretaceous Corral" => self.unlocked |= 1 << 13,
            "Bob V" => {
                let list = std::mem::take(&mut self.broken);
                self.animal_breaks.clear();
                let mut n = 0;
                for (t, d, v) in list {
                    let t = t as usize;
                    if t < N && self.edge(t, d) == 0 {
                        self.set_edge(t, d, v);
                        n += 1;
                    }
                }
                self.recompute_regions();
                return format!("Cheat activated: Bob V repaired {} fence segment{}.", n, if n == 1 { "" } else { "s" });
            }
            "George W" => {
                for t in 0..N {
                    if OBJS[self.obj[t] as usize].cat == C_FOLIAGE {
                        self.obj[t] = 0;
                    }
                }
                self.cash += 3000.0;
                self.need_beauty = true;
                self.recompute_stats();
            }
            _ => {}
        }
        let _ = region;
        self.refresh_name_effects();
        format!("Cheat activated: {}! {}{}", label, c.desc, if c.desc.ends_with('!') { "" } else { "." })
    }

    /// Rename a guest or staff member; fires a cheat if the name matches.
    pub fn rename_person(&mut self, id: u32, name: &str) -> String {
        let Some(i) = self.people.iter().position(|p| p.id == id) else { return "Nobody to rename.".into() };
        let name: String = name.trim().chars().take(32).collect();
        if name.is_empty() {
            return "Names can't be blank.".into();
        }
        self.people[i].name = name.clone();
        let kind = self.people[i].kind;
        if let Some(ci) = find_cheat(&name) {
            let ok = match NAME_CHEATS[ci].who {
                Who::Guest => kind == 2,
                Who::Staff => kind != 2,
                Who::Maint => kind == 4,
                Who::Exhibit => false,
            };
            if ok {
                let m = self.fire_cheat(ci, Some(i), None);
                self.msg(m.clone());
                return m;
            }
        }
        format!("Renamed to {}.", name)
    }

    pub fn rename_exhibit(&mut self, tile: usize, name: &str) -> String {
        if tile >= N {
            return "No exhibit there.".into();
        }
        let r = self.region[tile] as usize;
        if self.encs[r].outside {
            return "That isn't inside an exhibit.".into();
        }
        let name: String = name.trim().chars().take(32).collect();
        self.encs[r].name = name.clone();
        if let Some(ci) = find_cheat(&name) {
            if NAME_CHEATS[ci].who == Who::Exhibit {
                let m = self.fire_cheat(ci, None, Some(r));
                self.msg(m.clone());
                return m;
            }
        }
        self.refresh_name_effects();
        if name.is_empty() { "Exhibit name cleared.".into() } else { format!("Exhibit renamed to {}.", name) }
    }

    // ------------------------------------------------------------------
    // Terminal
    // ------------------------------------------------------------------

    /// Run a terminal command. `sel_ent` = selected entity id (or -1),
    /// `sel_tile` = selected tile (or -1). Returns text output.
    pub fn terminal(&mut self, line: &str, sel_ent: i32, sel_tile: i32) -> String {
        let line = line.trim();
        if line.is_empty() {
            return String::new();
        }
        let (cmd, rest) = match line.find(' ') {
            Some(i) => (&line[..i], line[i + 1..].trim()),
            None => (line, ""),
        };
        let mut o = String::new();
        match cmd.to_lowercase().as_str() {
            "help" | "?" => {
                o.push_str("WILDWOOD ZOO TERMINAL v1.0\n\
                    \n  help                 this list\
                    \n  cheats               list every secret name\
                    \n  money                +$10,000 (same as Shift+$)\
                    \n  rename <name>        rename the selected guest, staff or exhibit\
                    \n  name guest <name>    give a random guest a name\
                    \n  name staff <name>    give a staff member a name (hires one if needed)\
                    \n  name maint <name>    give a maintenance worker a name\
                    \n  name exhibit <name>  name the selected exhibit (or the first one)\
                    \n  <secret name>        shortcut: type a cheat name on its own\
                    \n  status               zoo summary\
                    \n  reset                undo color tint, sickness and birds\
                    \n  clear                clear the screen\
                    \n  exit                 close the terminal");
            }
            "cheats" | "codes" => {
                o.push_str("KEY COMBO\n  Shift + $ (Shift+4)   +$10,000  (fences may wear out!)\n");
                for (title, w) in [("GUEST NAMES", Who::Guest), ("STAFF NAMES", Who::Staff), ("MAINTENANCE WORKER NAMES", Who::Maint), ("EXHIBIT NAMES", Who::Exhibit)] {
                    let _ = write!(o, "\n{}\n", title);
                    for c in NAME_CHEATS.iter().filter(|c| c.who == w) {
                        let _ = writeln!(o, "  {:<18} {}  [{}]", c.name, c.desc, c.origin);
                    }
                }
                o.push_str("\nNote: Zoo Tycoon 2 itself shipped without cheat codes; these are the classic ones from the first game.");
            }
            "money" | "$" => {
                self.money_cheat();
                let _ = write!(o, "Granted $10,000. Balance: ${:.0}", self.cash);
            }
            "status" => {
                let guests = self.people.iter().filter(|p| p.kind == 2).count();
                let _ = write!(
                    o,
                    "Date: {}\nZoo size: {} x {} tiles\nCash: ${:.0}\nFame: {:.2} stars\nGuests: {}\nAnimals: {} ({} species)\nStaff: {}\nBroken fences on record: {}",
                    self.date_string(), self.own[2] - self.own[0], self.own[3] - self.own[1], self.cash, self.fame, guests, self.animals.len(), self.species_count(),
                    self.people.len() - guests, self.broken.len()
                );
                let mut fx = vec![];
                if self.fx_double_adm { fx.push("Blue Fang"); }
                if self.fx_double_don { fx.push("Microsoft"); }
                if self.fx_wonderland { fx.push("Wonderland"); }
                if !self.fx_copralite.is_empty() { fx.push("Copralite Corner"); }
                if self.unlocked & (1 << 12) != 0 { fx.push("Xanadu"); }
                if self.unlocked & (1 << 13) != 0 { fx.push("Cretaceous Corral"); }
                if !fx.is_empty() {
                    let _ = write!(o, "\nActive exhibit cheats: {}", fx.join(", "));
                }
            }
            "reset" => {
                self.tint = 0;
                self.birds = 0.0;
                for p in self.people.iter_mut() {
                    p.sick = false;
                }
                o.push_str("Colors restored, birds shooed away, guests feel better.");
            }
            "rename" => {
                if rest.is_empty() {
                    o.push_str("Usage: rename <name>   (select a guest, staff member or exhibit first)");
                } else if sel_ent >= 0 && self.people.iter().any(|p| p.id == sel_ent as u32) {
                    o = self.rename_person(sel_ent as u32, rest);
                } else if sel_tile >= 0 && self.region[sel_tile as usize] != self.outside {
                    o = self.rename_exhibit(sel_tile as usize, rest);
                } else {
                    o.push_str("Select a guest, staff member or exhibit first (or use: name guest|staff|maint|exhibit <name>).");
                }
            }
            "name" => {
                let (who, nm) = match rest.find(' ') {
                    Some(i) => (rest[..i].to_lowercase(), rest[i + 1..].trim()),
                    None => (rest.to_lowercase(), ""),
                };
                if nm.is_empty() {
                    o.push_str("Usage: name guest|staff|maint|exhibit <name>");
                } else {
                    o = self.name_someone(&who, nm, sel_tile);
                }
            }
            _ => {
                // Bare secret name?
                if let Some(ci) = find_cheat(line) {
                    let who = match NAME_CHEATS[ci].who {
                        Who::Guest => "guest",
                        Who::Staff => "staff",
                        Who::Maint => "maint",
                        Who::Exhibit => "exhibit",
                    };
                    o = self.name_someone(who, NAME_CHEATS[ci].name, sel_tile);
                } else {
                    let _ = write!(o, "Unknown command '{}'. Type 'help'.", cmd);
                }
            }
        }
        o
    }

    fn name_someone(&mut self, who: &str, nm: &str, sel_tile: i32) -> String {
        match who {
            "guest" => {
                let guests: Vec<u32> = self.people.iter().filter(|p| p.kind == 2).map(|p| p.id).collect();
                if guests.is_empty() {
                    return "There are no guests in the zoo to rename.".into();
                }
                let id = guests[self.rng.range(guests.len() as u32) as usize];
                let old = self.people.iter().find(|p| p.id == id).map(|p| p.name.clone()).unwrap_or_default();
                let r = self.rename_person(id, nm);
                format!("Guest {} is now {}.\n{}", old, nm, r)
            }
            "staff" | "keeper" | "maint" | "maintenance" => {
                let kind = if who.starts_with('m') { 4 } else { 3 };
                let id = match self.people.iter().find(|p| p.kind == kind || (kind == 3 && p.kind == 4 && who == "staff")) {
                    Some(p) => p.id,
                    None => {
                        let id = self.hire(kind);
                        if id < 0 {
                            return "No staff to rename, and you can't afford to hire one.".into();
                        }
                        id as u32
                    }
                };
                let r = self.rename_person(id, nm);
                format!("Staff member is now {}.\n{}", nm, r)
            }
            "exhibit" => {
                let mut tile = if sel_tile >= 0 && self.region[sel_tile as usize] != self.outside { Some(sel_tile as usize) } else { None };
                if tile.is_none() {
                    tile = self.encs.iter().find(|e| !e.outside && e.area > 0).map(|e| e.first_tile as usize);
                }
                match tile {
                    Some(t) => self.rename_exhibit(t, nm),
                    None => "Build an exhibit first, then name it.".into(),
                }
            }
            _ => "Usage: name guest|staff|maint|exhibit <name>".into(),
        }
    }
}
