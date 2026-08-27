use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub progression: ProgressionConfig,
    pub attributes: AttributeConfig,
    pub currency: CurrencyConfig,
    pub quests: QuestConfig,
    pub target_hud: TargetHudConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProgressionConfig {
    pub base_xp: u64,
    pub growth: f64,
    pub max_level: u32,
    pub attribute_points_per_level: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AttributeConfig {
    pub damage_per_point: f32,
    pub defense_per_point: f32,
    pub speed_per_point: f32,
    pub health_per_point: f32,
    pub max_damage_bonus: f32,
    pub max_defense_reduction: f32,
    pub max_walk_speed: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CurrencyConfig {
    pub name: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestConfig {
    pub max_active: usize,
    pub menu_title: String,
    pub attributes_title: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TargetHudConfig {
    pub enabled: bool,
    pub max_distance: f64,
    pub show_armor: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestBook {
    pub quests: Vec<Quest>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub objective: Objective,
    pub required_level: u32,
    #[serde(default)]
    pub prerequisite: Option<String>,
    pub reward_xp: u64,
    pub reward_ds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub target: String,
    pub amount: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveKind {
    Mine,
    Collect,
    Kill,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SafeZoneBook {
    pub zones: Vec<SafeZone>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SafeZone {
    pub id: String,
    pub display_name: String,
    pub world: String,
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub min_z: i32,
    pub max_z: i32,
    pub block_break: bool,
    pub block_place: bool,
    pub pvp: bool,
}

impl SafeZone {
    pub fn contains(&self, world: &str, x: f64, y: f64, z: f64) -> bool {
        self.world == world
            && x >= f64::from(self.min_x)
            && x <= f64::from(self.max_x)
            && y >= f64::from(self.min_y)
            && y <= f64::from(self.max_y)
            && z >= f64::from(self.min_z)
            && z <= f64::from(self.max_z)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attributes {
    pub damage: u32,
    pub defense: u32,
    pub speed: u32,
    pub vitality: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub level: u32,
    pub xp: u64,
    pub dragon_seeds: u64,
    pub unspent_points: u32,
    pub attributes: Attributes,
    pub active: HashMap<String, u64>,
    pub completed: HashSet<String>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            dragon_seeds: 0,
            unspent_points: 0,
            attributes: Attributes::default(),
            active: HashMap::new(),
            completed: HashSet::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MenuView {
    Quests(usize),
    Attributes,
}

#[derive(Clone, Debug)]
pub struct LastHit {
    pub player_id: String,
    pub target: String,
}
