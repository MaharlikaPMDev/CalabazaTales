use crate::{model::*, storage};
use pumpkin_plugin_api::{Player, Server, boss_bar::BossBar, text::TextComponent};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Mutex,
};

pub struct App {
    pub data_dir: PathBuf,
    pub config: Mutex<Config>,
    pub quests: Mutex<Vec<Quest>>,
    pub zones: Mutex<Vec<SafeZone>>,
    pub players: Mutex<HashMap<String, PlayerState>>,
    pub gui_views: Mutex<HashMap<String, MenuView>>,
    pub gui_refresh_pending: Mutex<HashSet<String>>,
    pub bedrock_forms: Mutex<HashMap<u32, (String, MenuView)>>,
    pub spawned_types: Mutex<HashMap<i32, String>>,
    pub last_hits: Mutex<HashMap<i32, LastHit>>,
    pub target_bars: Mutex<HashMap<String, BossBar>>,
}

impl App {
    pub fn load(data_dir: PathBuf) -> Result<Self, String> {
        storage::ensure_data_layout(&data_dir)?;
        let config_path = storage::seed_file(
            &data_dir,
            "config.toml",
            include_str!("../config/config.toml"),
        )?;
        let quests_path = storage::seed_file(
            &data_dir,
            "quests.toml",
            include_str!("../config/quests.toml"),
        )?;
        let zones_path = storage::seed_file(
            &data_dir,
            "safe_zones.toml",
            include_str!("../config/safe_zones.toml"),
        )?;
        let config =
            toml::from_str(&std::fs::read_to_string(config_path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("invalid config.toml: {e}"))?;
        let quests: QuestBook =
            toml::from_str(&std::fs::read_to_string(quests_path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("invalid quests.toml: {e}"))?;
        let zones: SafeZoneBook =
            toml::from_str(&std::fs::read_to_string(zones_path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("invalid safe_zones.toml: {e}"))?;
        Self::validate_quests(&quests.quests)?;
        Ok(Self {
            data_dir,
            config: Mutex::new(config),
            quests: Mutex::new(quests.quests),
            zones: Mutex::new(zones.zones),
            players: Mutex::new(HashMap::new()),
            gui_views: Mutex::new(HashMap::new()),
            gui_refresh_pending: Mutex::new(HashSet::new()),
            bedrock_forms: Mutex::new(HashMap::new()),
            spawned_types: Mutex::new(HashMap::new()),
            last_hits: Mutex::new(HashMap::new()),
            target_bars: Mutex::new(HashMap::new()),
        })
    }

    pub fn reload(&self) -> Result<(), String> {
        let config: Config = toml::from_str(
            &std::fs::read_to_string(self.data_dir.join("config.toml"))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("invalid config.toml: {e}"))?;
        let quests: QuestBook = toml::from_str(
            &std::fs::read_to_string(self.data_dir.join("quests.toml"))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("invalid quests.toml: {e}"))?;
        let zones: SafeZoneBook = toml::from_str(
            &std::fs::read_to_string(self.data_dir.join("safe_zones.toml"))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("invalid safe_zones.toml: {e}"))?;
        Self::validate_quests(&quests.quests)?;
        *self.config.lock().map_err(|e| e.to_string())? = config;
        *self.quests.lock().map_err(|e| e.to_string())? = quests.quests;
        *self.zones.lock().map_err(|e| e.to_string())? = zones.zones;
        Ok(())
    }

    fn validate_quests(quests: &[Quest]) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        for q in quests {
            if q.objective.amount == 0 {
                return Err(format!("quest {} has zero objective amount", q.id));
            }
            if !ids.insert(q.id.clone()) {
                return Err(format!("duplicate quest id: {}", q.id));
            }
        }
        for q in quests {
            if let Some(required) = &q.prerequisite
                && !ids.contains(required)
            {
                return Err(format!(
                    "quest {} references missing prerequisite {required}",
                    q.id
                ));
            }
        }
        Ok(())
    }

    pub fn player_id(player: &Player) -> String {
        player.get_id().to_string()
    }

    pub fn ensure_player(&self, player: &Player) {
        let id = Self::player_id(player);
        let mut players = self.players.lock().unwrap_or_else(|e| e.into_inner());
        players
            .entry(id.clone())
            .or_insert_with(|| storage::load_player(&self.data_dir, &id));
        drop(players);
        self.apply_attributes(player);
    }

    pub fn save(&self, id: &str) {
        let state = self
            .players
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned();
        if let Some(state) = state
            && let Err(error) = storage::save_player(&self.data_dir, id, &state)
        {
            tracing::error!("failed to save player {id}: {error}");
        }
    }

    pub fn snapshot(&self, id: &str) -> PlayerState {
        self.players
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn xp_needed(&self, level: u32) -> u64 {
        let cfg = self.config.lock().unwrap_or_else(|e| e.into_inner());
        ((cfg.progression.base_xp as f64)
            * cfg.progression.growth.powi(level.saturating_sub(1) as i32))
        .round() as u64
    }

    pub fn award_xp(&self, player: &Player, amount: u64) {
        let id = Self::player_id(player);
        self.ensure_player(player);
        let cfg = self
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut level_messages = Vec::new();
        {
            let mut players = self.players.lock().unwrap_or_else(|e| e.into_inner());
            let state = players.get_mut(&id).expect("player state exists");
            state.xp = state.xp.saturating_add(amount);
            while state.level < cfg.progression.max_level {
                let needed = ((cfg.progression.base_xp as f64)
                    * cfg
                        .progression
                        .growth
                        .powi(state.level.saturating_sub(1) as i32))
                .round() as u64;
                if state.xp < needed {
                    break;
                }
                state.xp -= needed;
                state.level += 1;
                state.unspent_points = state
                    .unspent_points
                    .saturating_add(cfg.progression.attribute_points_per_level);
                level_messages.push(state.level);
            }
        }
        self.save(&id);
        for level in level_messages {
            player.send_system_message(
                TextComponent::text(&format!(
                    "✦ Level {level}! You gained {} attribute points.",
                    cfg.progression.attribute_points_per_level
                )),
                false,
            );
        }
    }

    pub fn progress(&self, player: &Player, kind: ObjectiveKind, raw_target: &str, amount: u64) {
        let id = Self::player_id(player);
        self.ensure_player(player);
        let target = normalize_id(raw_target);
        let quests = self
            .quests
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut finished = Vec::new();
        {
            let mut players = self.players.lock().unwrap_or_else(|e| e.into_inner());
            let state = players.get_mut(&id).expect("player state exists");
            for (quest_id, progress) in &mut state.active {
                if let Some(quest) = quests.iter().find(|q| &q.id == quest_id)
                    && quest.objective.kind == kind
                    && normalize_id(&quest.objective.target) == target
                {
                    *progress = progress.saturating_add(amount).min(quest.objective.amount);
                    if *progress == quest.objective.amount {
                        finished.push(quest.title.clone());
                    }
                }
            }
        }
        if !finished.is_empty() {
            self.save(&id);
            for title in finished {
                player.send_system_message(
                    TextComponent::text(&format!(
                        "✓ Quest objective complete: {title}. Open /tales to claim."
                    )),
                    false,
                );
            }
        }
    }

    pub fn activate_or_claim(&self, player: &Player, quest_index: usize) -> String {
        let id = Self::player_id(player);
        self.ensure_player(player);
        let quests = self
            .quests
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(quest) = quests.get(quest_index) else {
            return "That quest no longer exists.".into();
        };
        let max_active = self
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .quests
            .max_active;
        let mut reward = None;
        let message;
        {
            let mut players = self.players.lock().unwrap_or_else(|e| e.into_inner());
            let state = players.get_mut(&id).expect("player state exists");
            if state.completed.contains(&quest.id) {
                return "You have already completed that quest.".into();
            }
            if let Some(progress) = state.active.get(&quest.id).copied() {
                if progress < quest.objective.amount {
                    return format!("{}: {progress}/{}", quest.title, quest.objective.amount);
                }
                state.active.remove(&quest.id);
                state.completed.insert(quest.id.clone());
                state.dragon_seeds = state.dragon_seeds.saturating_add(quest.reward_ds);
                reward = Some(quest.reward_xp);
                message = format!(
                    "Claimed {} — +{} XP, +{} Ds",
                    quest.title, quest.reward_xp, quest.reward_ds
                );
            } else if state.level < quest.required_level {
                return format!("Requires level {}.", quest.required_level);
            } else if quest
                .prerequisite
                .as_ref()
                .is_some_and(|q| !state.completed.contains(q))
            {
                return "Complete the preceding quest first.".into();
            } else if state.active.len() >= max_active {
                return format!("You may only track {max_active} quests at once.");
            } else {
                state.active.insert(quest.id.clone(), 0);
                message = format!("Quest accepted: {}", quest.title);
            }
        }
        self.save(&id);
        if let Some(xp) = reward {
            self.award_xp(player, xp);
        }
        message
    }

    pub fn spend_attribute(&self, player: &Player, index: u32) -> String {
        let id = Self::player_id(player);
        self.ensure_player(player);
        let label;
        {
            let mut players = self.players.lock().unwrap_or_else(|e| e.into_inner());
            let state = players.get_mut(&id).expect("player state exists");
            if state.unspent_points == 0 {
                return "You have no unspent attribute points.".into();
            }
            let stat = match index {
                0 => {
                    label = "Damage";
                    &mut state.attributes.damage
                }
                1 => {
                    label = "Defense";
                    &mut state.attributes.defense
                }
                2 => {
                    label = "Speed";
                    &mut state.attributes.speed
                }
                3 => {
                    label = "Vitality";
                    &mut state.attributes.vitality
                }
                _ => return "Unknown attribute.".into(),
            };
            *stat = stat.saturating_add(1);
            state.unspent_points -= 1;
        }
        self.save(&id);
        self.apply_attributes(player);
        format!("{label} increased by 1.")
    }

    pub fn apply_attributes(&self, player: &Player) {
        let id = Self::player_id(player);
        let state = self.snapshot(&id);
        let cfg = self.config.lock().unwrap_or_else(|e| e.into_inner());
        player.set_max_health(
            20.0 + state.attributes.vitality as f32 * cfg.attributes.health_per_point,
        );
        let speed = (0.1 + state.attributes.speed as f32 * cfg.attributes.speed_per_point)
            .min(cfg.attributes.max_walk_speed);
        player.set_walk_speed(speed);
    }

    pub fn damage_multiplier(&self, id: &str) -> f32 {
        let state = self.snapshot(id);
        let cfg = self.config.lock().unwrap_or_else(|e| e.into_inner());
        1.0 + (state.attributes.damage as f32 * cfg.attributes.damage_per_point)
            .min(cfg.attributes.max_damage_bonus)
    }

    pub fn defense_multiplier(&self, id: &str) -> f32 {
        let state = self.snapshot(id);
        let cfg = self.config.lock().unwrap_or_else(|e| e.into_inner());
        1.0 - (state.attributes.defense as f32 * cfg.attributes.defense_per_point)
            .min(cfg.attributes.max_defense_reduction)
    }

    pub fn safe_zone_for(&self, player: &Player) -> Option<SafeZone> {
        let world = player.get_world().get_id();
        let (x, y, z) = player.get_position();
        self.zones
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|zone| zone.contains(&world, x, y, z))
            .cloned()
    }

    pub fn player_by_entity_id(&self, server: &Server, entity_id: i32) -> Option<Player> {
        server
            .get_all_players()
            .into_iter()
            .find(|p| p.as_entity().get_id() as i32 == entity_id)
    }
}

pub fn normalize_id(value: &str) -> String {
    let lower = value.trim().to_ascii_lowercase().replace(' ', "_");
    if lower.contains(':') {
        lower
    } else {
        format!("minecraft:{lower}")
    }
}
