//! Static game data: terrain, objects, species.

/// The whole world grid. Only the owned rectangle is playable; the rest is land for sale.
pub const W: usize = 128;
pub const H: usize = 128;
/// Size of the map used by version 1/2 saves (before land ownership).
pub const OLD_W: usize = 56;
pub const OLD_H: usize = 56;
/// Starting zoo sizes (width, height) for Small / Medium / Large.
pub const START_SIZES: [(usize, usize); 3] = [(56, 56), (80, 80), (112, 112)];
pub const LAND_STEP: usize = 8;
pub const LAND_PRICE: i32 = 20;
pub const N: usize = W * H;

// ---- Terrain ----
pub const T_GRASS: u8 = 0;
pub const T_SAVANNA: u8 = 1;
pub const T_DIRT: u8 = 2;
pub const T_SAND: u8 = 3;
pub const T_SNOW: u8 = 4;
pub const T_ROCK: u8 = 5;
pub const T_WATER: u8 = 6;
pub const T_PATH: u8 = 7;
pub const T_JUNGLE: u8 = 8;
pub const NT: usize = 9;

pub const TERRAIN_NAMES: [&str; NT] = [
    "Grass", "Savanna Grass", "Dirt", "Sand", "Snow", "Rock", "Fresh Water", "Path", "Rainforest Floor",
];
pub const TERRAIN_COST: [i32; NT] = [5, 6, 4, 5, 8, 8, 15, 10, 7];

// ---- Biome tags (for foliage preference) ----
pub const B_TEMP: u8 = 1;
pub const B_BOREAL: u8 = 2;
pub const B_SAV: u8 = 4;
pub const B_TROP: u8 = 8;
pub const B_DES: u8 = 16;
pub const B_ASIA: u8 = 32;
pub const B_POLAR: u8 = 64;

// ---- Object categories ----
pub const C_FOLIAGE: u8 = 0;
pub const C_ANIMAL_ITEM: u8 = 1;
pub const C_GUEST: u8 = 2;
pub const C_SCENERY: u8 = 3;
pub const C_SPECIAL: u8 = 4;

pub struct ObjDef {
    pub name: &'static str,
    pub cost: i32,
    pub cat: u8,
    pub biome: u8,
    pub blocks: bool,
    pub upkeep: i32,
    pub beauty: f32,
}

macro_rules! o {
    ($n:expr, $c:expr, $cat:expr, $b:expr, $bl:expr, $u:expr, $be:expr) => {
        ObjDef { name: $n, cost: $c, cat: $cat, biome: $b, blocks: $bl, upkeep: $u, beauty: $be }
    };
}

pub const O_SHELTER: u8 = 14;
pub const O_TOY: u8 = 15;
pub const O_TROUGH: u8 = 16;
pub const O_FOOD: u8 = 17;
pub const O_DRINK: u8 = 18;
pub const O_RESTROOM: u8 = 19;
pub const O_BENCH: u8 = 20;
pub const O_TRASH: u8 = 21;
pub const O_GIFT: u8 = 22;
pub const O_DONATION: u8 = 23;
pub const O_ENTRANCE: u8 = 27;

pub const OBJS: [ObjDef; 28] = [
    o!("None", 0, C_SPECIAL, 0, false, 0, 0.0),
    o!("Oak Tree", 60, C_FOLIAGE, B_TEMP, true, 0, 0.15),
    o!("Pine Tree", 60, C_FOLIAGE, B_BOREAL, true, 0, 0.15),
    o!("Acacia Tree", 80, C_FOLIAGE, B_SAV, true, 0, 0.15),
    o!("Palm Tree", 80, C_FOLIAGE, B_TROP | B_DES, true, 0, 0.2),
    o!("Bamboo", 50, C_FOLIAGE, B_ASIA, false, 0, 0.1),
    o!("Saguaro Cactus", 40, C_FOLIAGE, B_DES, true, 0, 0.1),
    o!("Shrub", 30, C_FOLIAGE, B_TEMP | B_BOREAL | B_SAV | B_TROP | B_ASIA, false, 0, 0.1),
    o!("Kapok Tree", 90, C_FOLIAGE, B_TROP | B_ASIA, true, 0, 0.2),
    o!("Boulder", 40, C_FOLIAGE, 0, true, 0, 0.05),
    o!("Elephant Grass", 20, C_FOLIAGE, B_SAV | B_ASIA, false, 0, 0.05),
    o!("Tree Fern", 25, C_FOLIAGE, B_TROP | B_TEMP, false, 0, 0.1),
    o!("Ice Rock", 50, C_FOLIAGE, B_POLAR, true, 0, 0.05),
    o!("Birch Tree", 60, C_FOLIAGE, B_TEMP | B_BOREAL, true, 0, 0.15),
    o!("Animal Shelter", 600, C_ANIMAL_ITEM, 0, true, 5, 0.0),
    o!("Enrichment Toy", 150, C_ANIMAL_ITEM, 0, false, 0, 0.0),
    o!("Water Trough", 200, C_ANIMAL_ITEM, 0, false, 2, 0.0),
    o!("Burger Stand", 1500, C_GUEST, 0, true, 40, 0.0),
    o!("Soda Stand", 1000, C_GUEST, 0, true, 30, 0.0),
    o!("Restroom", 1200, C_GUEST, 0, true, 30, 0.0),
    o!("Bench", 120, C_GUEST, 0, false, 0, 0.05),
    o!("Trash Can", 80, C_GUEST, 0, false, 0, 0.0),
    o!("Gift Shop", 3000, C_GUEST, 0, true, 60, 0.05),
    o!("Donation Box", 200, C_GUEST, 0, false, 0, 0.0),
    o!("Flower Bed", 60, C_SCENERY, 0, false, 0, 0.35),
    o!("Fountain", 900, C_SCENERY, 0, true, 5, 0.8),
    o!("Lamp Post", 100, C_SCENERY, 0, false, 0, 0.15),
    o!("Zoo Entrance", 0, C_SPECIAL, 0, true, 0, 0.3),
];
pub const NOBJ: usize = 28;

// ---- Fences ----
pub const FENCE_NAMES: [&str; 4] = ["None", "Wooden Fence", "Chain-Link Fence", "Concrete Wall"];
pub const FENCE_COST: [i32; 4] = [0, 25, 40, 75];
pub const GATE_BIT: u8 = 4;

// ---- Species ----
pub struct Species {
    pub name: &'static str,
    pub plural: &'static str,
    /// fraction preferences indexed by terrain id
    pub terrain: [f32; NT],
    pub foliage: u8,
    pub density: f32,
    pub space: f32,
    pub group_min: u16,
    pub group_max: u16,
    pub strength: u8,
    pub swims: bool,
    pub predator: bool,
    pub cost: i32,
    pub fame: f32,
    pub speed: f32,
    pub biome_label: &'static str,
}

const fn tp(g: f32, sv: f32, d: f32, sa: f32, sn: f32, r: f32, wa: f32, j: f32) -> [f32; NT] {
    [g, sv, d, sa, sn, r, wa, 0.0, j]
}

pub const NS: usize = 14;
/// Species at or above this index are secret and must be unlocked by cheats.
pub const NS_PUBLIC: usize = 12;
pub const SPECIES: [Species; NS] = [
    Species { name: "Lion", plural: "Lions", terrain: tp(0.1, 0.6, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0), foliage: B_SAV, density: 0.06, space: 12.0, group_min: 2, group_max: 6, strength: 2, swims: false, predator: true, cost: 3000, fame: 8.0, speed: 1.1, biome_label: "Savanna" },
    Species { name: "Plains Zebra", plural: "Zebras", terrain: tp(0.2, 0.7, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0), foliage: B_SAV, density: 0.05, space: 10.0, group_min: 3, group_max: 10, strength: 1, swims: false, predator: false, cost: 1200, fame: 4.0, speed: 1.4, biome_label: "Savanna" },
    Species { name: "Reticulated Giraffe", plural: "Giraffes", terrain: tp(0.1, 0.6, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0), foliage: B_SAV, density: 0.08, space: 16.0, group_min: 2, group_max: 8, strength: 1, swims: false, predator: false, cost: 2500, fame: 6.0, speed: 1.0, biome_label: "Savanna" },
    Species { name: "African Elephant", plural: "Elephants", terrain: tp(0.1, 0.4, 0.4, 0.0, 0.0, 0.0, 0.1, 0.0), foliage: B_SAV, density: 0.05, space: 28.0, group_min: 2, group_max: 6, strength: 3, swims: false, predator: false, cost: 6000, fame: 10.0, speed: 0.7, biome_label: "Savanna" },
    Species { name: "Grizzly Bear", plural: "Grizzly Bears", terrain: tp(0.45, 0.0, 0.2, 0.0, 0.0, 0.2, 0.15, 0.0), foliage: B_TEMP | B_BOREAL, density: 0.15, space: 15.0, group_min: 1, group_max: 3, strength: 2, swims: true, predator: true, cost: 3500, fame: 8.0, speed: 0.9, biome_label: "Temperate Forest" },
    Species { name: "Giant Panda", plural: "Giant Pandas", terrain: tp(0.6, 0.0, 0.2, 0.0, 0.0, 0.2, 0.0, 0.0), foliage: B_ASIA, density: 0.25, space: 12.0, group_min: 1, group_max: 2, strength: 1, swims: false, predator: false, cost: 8000, fame: 12.0, speed: 0.5, biome_label: "Bamboo Forest" },
    Species { name: "Emperor Penguin", plural: "Penguins", terrain: tp(0.0, 0.0, 0.0, 0.0, 0.6, 0.1, 0.3, 0.0), foliage: B_POLAR, density: 0.0, space: 3.0, group_min: 6, group_max: 20, strength: 1, swims: true, predator: false, cost: 900, fame: 5.0, speed: 0.6, biome_label: "Tundra" },
    Species { name: "Polar Bear", plural: "Polar Bears", terrain: tp(0.0, 0.0, 0.0, 0.0, 0.6, 0.15, 0.25, 0.0), foliage: B_POLAR, density: 0.03, space: 18.0, group_min: 1, group_max: 3, strength: 3, swims: true, predator: true, cost: 5000, fame: 10.0, speed: 0.9, biome_label: "Tundra" },
    Species { name: "Bengal Tiger", plural: "Bengal Tigers", terrain: tp(0.4, 0.0, 0.2, 0.0, 0.0, 0.0, 0.1, 0.3), foliage: B_ASIA | B_TROP, density: 0.2, space: 15.0, group_min: 1, group_max: 2, strength: 3, swims: true, predator: true, cost: 5500, fame: 11.0, speed: 1.1, biome_label: "Tropical Forest" },
    Species { name: "Western Gorilla", plural: "Gorillas", terrain: tp(0.2, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.7), foliage: B_TROP, density: 0.3, space: 14.0, group_min: 3, group_max: 8, strength: 2, swims: false, predator: false, cost: 6000, fame: 10.0, speed: 0.7, biome_label: "Tropical Rainforest" },
    Species { name: "Dromedary Camel", plural: "Camels", terrain: tp(0.0, 0.0, 0.2, 0.8, 0.0, 0.0, 0.0, 0.0), foliage: B_DES, density: 0.04, space: 12.0, group_min: 2, group_max: 6, strength: 1, swims: false, predator: false, cost: 1500, fame: 4.0, speed: 0.9, biome_label: "Desert" },
    Species { name: "Hippopotamus", plural: "Hippos", terrain: tp(0.3, 0.0, 0.3, 0.0, 0.0, 0.0, 0.4, 0.0), foliage: B_SAV | B_TROP, density: 0.05, space: 18.0, group_min: 2, group_max: 8, strength: 2, swims: true, predator: false, cost: 4000, fame: 7.0, speed: 0.6, biome_label: "Savanna Wetland" },
    Species { name: "Unicorn", plural: "Unicorns", terrain: tp(0.7, 0.0, 0.1, 0.0, 0.0, 0.0, 0.2, 0.0), foliage: B_TEMP, density: 0.15, space: 14.0, group_min: 1, group_max: 4, strength: 2, swims: false, predator: false, cost: 10000, fame: 16.0, speed: 1.3, biome_label: "Enchanted Forest" },
    Species { name: "Triceratops", plural: "Triceratops", terrain: tp(0.4, 0.0, 0.4, 0.0, 0.0, 0.2, 0.0, 0.0), foliage: B_TROP | B_TEMP, density: 0.12, space: 30.0, group_min: 2, group_max: 6, strength: 3, swims: false, predator: false, cost: 12000, fame: 15.0, speed: 0.6, biome_label: "Cretaceous Forest" },
];

pub const ANIMAL_NAMES: [&str; 48] = [
    "Kibo", "Nala", "Zuri", "Tumbo", "Asha", "Bongo", "Juma", "Lulu", "Mosi", "Pili", "Rafiki", "Sefu",
    "Tatu", "Wanjiru", "Yaro", "Zola", "Amara", "Biscuit", "Clover", "Dumpling", "Ember", "Fig", "Ginger",
    "Hazel", "Indigo", "Juniper", "Kona", "Lotus", "Mango", "Nutmeg", "Olive", "Pepper", "Quill", "Rosie",
    "Sage", "Toffee", "Umber", "Violet", "Willow", "Xena", "Yuki", "Ziggy", "Bao", "Mei", "Tenzing", "Sora",
    "Pip", "Moose",
];

pub const GUEST_THOUGHTS: [&str; 26] = [
    "",
    "I'm starving, but I can't find any food!",
    "I'm so thirsty and there's nothing to drink.",
    "I really need to find a restroom...",
    "My feet hurt. I wish there were more benches.",
    "Wow! These animals are amazing!",
    "Those animals look really unhappy.",
    "This zoo is filthy! Look at all this litter.",
    "The admission price here is a rip-off.",
    "AAAH! An animal has escaped!",
    "That burger hit the spot.",
    "Mmm, refreshing soda.",
    "Ahh, much better.",
    "Nice to sit down for a bit.",
    "I bought a souvenir!",
    "This place is beautiful.",
    "I'm bored. There's not much to see here.",
    "I'm out of money. Time to go home.",
    "What a great day at the zoo!",
    "I'm tired, time to head home.",
    "I wish I could see more kinds of animals.",
    "The exhibits here are so natural!",
    "I donated to help the animals.",
    "Where do all these paths go?",
    "AAAH! The birds are attacking me!",
    "Ugh... I don't feel so good.",
];

pub const FIRST_NAMES: [&str; 40] = [
    "Alex", "Beth", "Carlos", "Dana", "Eli", "Fiona", "Greg", "Hana", "Ivan", "Jade", "Kyle", "Lena", "Marco",
    "Nina", "Oscar", "Priya", "Quinn", "Rosa", "Sam", "Tara", "Uma", "Victor", "Wendy", "Yusuf", "Zoe", "Ben",
    "Chloe", "Diego", "Emma", "Frank", "Grace", "Hector", "Iris", "Jon", "Kara", "Leo", "Maya", "Noah", "Olga", "Pete",
];
pub const LAST_NAMES: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
    "W", "X", "Y", "Z",
];

pub const GOALS: [(&str, i32); 11] = [
    ("Adopt your first animal", 1000),
    ("Hire a zookeeper", 500),
    ("Have 40 guests in the zoo at once", 2000),
    ("Exhibit 3 different species", 3000),
    ("Reach 2-star zoo fame", 3000),
    ("Have a baby animal born", 5000),
    ("Own 20 animals", 5000),
    ("Reach 3.5-star zoo fame", 10000),
    ("Exhibit 7 different species", 10000),
    ("Reach 5-star zoo fame", 20000),
    ("Have $150,000 in the bank", 0),
];

pub const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
    "November", "December",
];
