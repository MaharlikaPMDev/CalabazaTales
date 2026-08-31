mod app;
mod model;
mod storage;
mod ui;

use app::{App, UiIntent, normalize_id};
use model::{LastHit, ObjectiveKind};
use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, Server,
    boss_bar::{BossBar, BossBarColor, BossBarDivision},
    command::{Command, CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    events::{
        BedrockFormResponseEvent, BlockBreakEvent, BlockPlaceEvent, EntityDamageByEntityEvent,
        EntityDeathEvent, EntityPickupItemEvent, EntitySpawnEvent, EventData, EventHandler,
        EventPriority, InventoryClickEvent, PlayerJoinEvent, PlayerLeaveEvent,
    },
    forms::FormResponse,
    permission::{Permission, PermissionDefault, PermissionLevel},
    permissions,
    scheduler::SchedulerExt,
    text::TextComponent,
};
use std::{path::PathBuf, sync::Arc};

const PERMISSION_TALES: &str = "CalabazaTales:command.tales";
const PERMISSION_ADMIN: &str = "CalabazaTales:command.admin";

struct CalabazaTales {
    app: Option<Arc<App>>,
}

impl Plugin for CalabazaTales {
    fn new() -> Self {
        Self { app: None }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "CalabazaTales".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["MaharlikaPMDev".into()],
            description: "A cross-edition MMORPG foundation: quests, levels, attributes, safe zones, target HUD, and Dragon Seeds.".into(),
            dependencies: vec![],
            permissions: vec![permissions::FS_READ_DATA.into(), permissions::FS_WRITE_DATA.into()],
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        let app = Arc::new(App::load(PathBuf::from(context.get_data_folder()))?);
        register_permissions(&context)?;
        register_commands(&context, app.clone());

        context.register_event_handler(JoinHandler(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(LeaveHandler(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(BreakHandler(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(PlaceHandler(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(PickupHandler(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(SpawnHandler(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(DamageHandler(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(DeathHandler(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(ClickHandler(app.clone()), EventPriority::Highest, true)?;
        context.register_event_handler(FormHandler(app.clone()), EventPriority::Normal, true)?;
        let ui_app = app.clone();
        context.schedule_repeating_task(1, 1, move |server| process_ui_intents(&ui_app, &server));
        self.app = Some(app);
        tracing::info!("CalabazaTales loaded with callback-safe deferred UI processing");
        Ok(())
    }

    fn handle_ipc_message(&mut self, _sender: String, message: Vec<u8>) -> Result<Vec<u8>, String> {
        let app = self.app.as_ref().ok_or("plugin is not loaded")?;
        let request: serde_json::Value =
            serde_json::from_slice(&message).map_err(|e| format!("invalid request: {e}"))?;
        if request.get("schema").and_then(serde_json::Value::as_str) != Some("calabazatales.ipc")
            || request.get("version").and_then(serde_json::Value::as_u64) != Some(1)
        {
            return Err("unsupported CalabazaTales IPC schema".into());
        }
        let action = request
            .get("action")
            .and_then(serde_json::Value::as_str)
            .ok_or("missing action")?;
        let response = match action {
            "capabilities" => {
                serde_json::json!({"schema":"calabazatales.ipc","version":1,"actions":["capabilities","active_quest"]})
            }
            "active_quest" => {
                let player = request
                    .get("player")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("missing player")?;
                let active_state = app
                    .players
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(player)
                    .and_then(|state| {
                        state
                            .active
                            .iter()
                            .next()
                            .map(|(id, progress)| (id.clone(), *progress))
                    });
                let quests = app.quests.lock().unwrap_or_else(|e| e.into_inner());
                let active = active_state.as_ref().and_then(|(id, progress)| {
                    quests
                        .iter()
                        .find(|quest| &quest.id == id)
                        .map(|quest| (quest, *progress))
                });
                serde_json::json!({"schema":"calabazatales.ipc","version":1,"quest_id":active.map(|(q,_)|q.id.as_str()),"quest_name":active.map(|(q,_)|q.title.as_str()),"quest_progress":active.map(|(_,p)|p)})
            }
            _ => return Err("unsupported action".into()),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
    }
}

pumpkin_plugin_api::register_plugin!(CalabazaTales);

fn register_permissions(context: &Context) -> pumpkin_plugin_api::Result<()> {
    context.register_permission(&Permission {
        node: PERMISSION_TALES.into(),
        description: "Open the CalabazaTales menus".into(),
        default: PermissionDefault::Allow,
        children: vec![],
    })?;
    context.register_permission(&Permission {
        node: PERMISSION_ADMIN.into(),
        description: "Reload and validate CalabazaTales configuration".into(),
        default: PermissionDefault::Op(PermissionLevel::Three),
        children: vec![],
    })
}

fn register_commands(context: &Context, app: Arc<App>) {
    let command = Command::new(
        &["tales".into(), "quests".into(), "attributes".into()],
        "Open CalabazaTales quests and character progression.",
    )
    .execute(TalesCommand {
        app: app.clone(),
        action: CommandAction::Quests,
    })
    .then(CommandNode::literal("quests").execute(TalesCommand {
        app: app.clone(),
        action: CommandAction::Quests,
    }))
    .then(CommandNode::literal("attributes").execute(TalesCommand {
        app: app.clone(),
        action: CommandAction::Attributes,
    }))
    .then(CommandNode::literal("stats").execute(TalesCommand {
        app: app.clone(),
        action: CommandAction::Attributes,
    }))
    .then(CommandNode::literal("reload").execute(TalesCommand {
        app,
        action: CommandAction::Reload,
    }));
    context.register_command(command, PERMISSION_TALES);
}

#[derive(Clone, Copy)]
enum CommandAction {
    Quests,
    Attributes,
    Reload,
}
struct TalesCommand {
    app: Arc<App>,
    action: CommandAction,
}
impl CommandHandler for TalesCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if matches!(self.action, CommandAction::Reload) {
            if !sender.has_permission(&server, PERMISSION_ADMIN) {
                return Err(CommandError::PermissionDenied);
            }
            return match self.app.reload() {
                Ok(()) => {
                    sender.send_message(TextComponent::text(
                        "CalabazaTales configuration reloaded and validated.",
                    ));
                    Ok(1)
                }
                Err(error) => Err(CommandError::CommandFailed(TextComponent::text(&error))),
            };
        }
        let Some(player) = sender.as_player() else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "This menu can only be opened by a player.",
            )));
        };
        match self.action {
            CommandAction::Quests => ui::open_main(&self.app, &player, 0),
            CommandAction::Attributes => ui::open_attributes(&self.app, &player),
            CommandAction::Reload => unreachable!(),
        }
        Ok(1)
    }
}

struct JoinHandler(Arc<App>);
impl EventHandler<PlayerJoinEvent> for JoinHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<PlayerJoinEvent>,
    ) -> EventData<PlayerJoinEvent> {
        self.0.ensure_player(&event.player);
        let id = App::player_id(&event.player);
        let state = self.0.snapshot(&id);
        let symbol = self
            .0
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .currency
            .symbol
            .clone();
        event.player.send_system_message(
            TextComponent::text(&format!(
                "Welcome to Calabaza Tales • Level {} • {} {} • /tales",
                state.level, state.dragon_seeds, symbol
            )),
            false,
        );
        event
    }
}

struct LeaveHandler(Arc<App>);
impl EventHandler<PlayerLeaveEvent> for LeaveHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<PlayerLeaveEvent>,
    ) -> EventData<PlayerLeaveEvent> {
        let id = App::player_id(&event.player);
        self.0.save(&id);
        self.0
            .gui_views
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        self.0
            .ui_intents
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|intent| match intent {
                UiIntent::JavaClick { player_id, .. } | UiIntent::BedrockForm { player_id, .. } => {
                    player_id != &id
                }
            });
        if let Some(bar) = self
            .0
            .target_bars
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id)
        {
            bar.remove_all();
        }
        event
    }
}

struct BreakHandler(Arc<App>);
impl EventHandler<BlockBreakEvent> for BreakHandler {
    fn handle(
        &self,
        _server: Server,
        mut event: EventData<BlockBreakEvent>,
    ) -> EventData<BlockBreakEvent> {
        if let Some(player) = &event.player {
            if let Some(zone) = self.0.safe_zone_for(player).filter(|z| z.block_break) {
                event.cancelled = true;
                player.send_system_message(
                    TextComponent::text(&format!(
                        "You cannot break blocks inside {} ({}).",
                        zone.display_name, zone.id
                    )),
                    false,
                );
            } else if !event.cancelled {
                self.0
                    .progress(player, ObjectiveKind::Mine, &event.block, 1);
                self.0.award_xp(player, 2);
            }
        }
        event
    }
}

struct PlaceHandler(Arc<App>);
impl EventHandler<BlockPlaceEvent> for PlaceHandler {
    fn handle(
        &self,
        _server: Server,
        mut event: EventData<BlockPlaceEvent>,
    ) -> EventData<BlockPlaceEvent> {
        if let Some(zone) = self
            .0
            .safe_zone_for(&event.player)
            .filter(|z| z.block_place)
        {
            event.cancelled = true;
            event.player.send_system_message(
                TextComponent::text(&format!(
                    "You cannot place blocks inside {} ({}).",
                    zone.display_name, zone.id
                )),
                false,
            );
        }
        event
    }
}

struct PickupHandler(Arc<App>);
impl EventHandler<EntityPickupItemEvent> for PickupHandler {
    fn handle(
        &self,
        server: Server,
        event: EventData<EntityPickupItemEvent>,
    ) -> EventData<EntityPickupItemEvent> {
        if !event.cancelled
            && let Some(player) = self.0.player_by_entity_id(&server, event.entity_id)
        {
            self.0.progress(
                &player,
                ObjectiveKind::Collect,
                &event.item_name,
                u64::from(event.count),
            );
            self.0.award_xp(&player, u64::from(event.count).min(5));
        }
        event
    }
}

struct SpawnHandler(Arc<App>);
impl EventHandler<EntitySpawnEvent> for SpawnHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<EntitySpawnEvent>,
    ) -> EventData<EntitySpawnEvent> {
        self.0
            .spawned_types
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(event.entity_id, normalize_id(&event.entity_type));
        event
    }
}

struct DamageHandler(Arc<App>);
impl EventHandler<EntityDamageByEntityEvent> for DamageHandler {
    fn handle(
        &self,
        server: Server,
        mut event: EventData<EntityDamageByEntityEvent>,
    ) -> EventData<EntityDamageByEntityEvent> {
        let attacker = self.0.player_by_entity_id(&server, event.damager_id);
        let victim = self.0.player_by_entity_id(&server, event.entity_id);
        if attacker
            .as_ref()
            .is_some_and(|p| self.0.safe_zone_for(p).is_some_and(|z| z.pvp))
            || victim
                .as_ref()
                .is_some_and(|p| self.0.safe_zone_for(p).is_some_and(|z| z.pvp))
        {
            event.cancelled = true;
            if let Some(player) = attacker {
                player.send_system_message(
                    TextComponent::text("Combat is disabled inside this safe zone."),
                    false,
                );
            }
            return event;
        }
        if let Some(player) = attacker {
            let id = App::player_id(&player);
            event.damage *= self.0.damage_multiplier(&id);
            let detected = player
                .get_target_entity(
                    self.0
                        .config
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .target_hud
                        .max_distance,
                )
                .filter(|entity| entity.get_id() as i32 == event.entity_id)
                .map(|entity| normalize_id(&format!("{:?}", entity.get_type())));
            let target = self
                .0
                .spawned_types
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&event.entity_id)
                .cloned()
                .or(detected)
                .unwrap_or_else(|| {
                    if victim.is_some() {
                        "minecraft:player".into()
                    } else {
                        "minecraft:entity".into()
                    }
                });
            self.0
                .last_hits
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(
                    event.entity_id,
                    LastHit {
                        player_id: id,
                        target,
                    },
                );
            self.show_target_hud(player, victim.as_ref().map_or(0, estimate_player_armor));
        }
        if let Some(player) = &victim {
            event.damage *= self.0.defense_multiplier(&App::player_id(player));
        }
        event
    }
}

impl DamageHandler {
    fn show_target_hud(&self, player: pumpkin_plugin_api::Player, armor: u32) {
        let cfg = self
            .0
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if !cfg.target_hud.enabled {
            return;
        }
        let Some(target) = player.get_target_entity(cfg.target_hud.max_distance) else {
            return;
        };
        let Some(living) = target.as_living() else {
            return;
        };
        let health = living.get_health().max(0.0);
        let max_health = living.get_max_health().max(1.0);
        let name = format!("{:?}", target.get_type()).replace('_', " ");
        let title = if cfg.target_hud.show_armor {
            format!("{name}  ❤ {health:.1}/{max_health:.1}  🛡 {armor}")
        } else {
            format!("{name}  ❤ {health:.1}/{max_health:.1}")
        };
        let id = App::player_id(&player);
        let mut bars = self.0.target_bars.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(bar) = bars.get(&id) {
            bar.set_title(TextComponent::text(&title));
            bar.set_health((health / max_health).clamp(0.0, 1.0));
        } else {
            let bar = BossBar::new(
                TextComponent::text(&title),
                BossBarColor::Red,
                BossBarDivision::Notches10,
            );
            bar.set_health((health / max_health).clamp(0.0, 1.0));
            bar.add_player(player);
            bars.insert(id, bar);
        }
    }
}

fn estimate_player_armor(player: &pumpkin_plugin_api::Player) -> u32 {
    let inventory = player.get_inventory();
    [
        inventory.get_helmet(),
        inventory.get_chestplate(),
        inventory.get_leggings(),
        inventory.get_boots(),
    ]
    .into_iter()
    .flatten()
    .map(|item| armor_points(&item.get_registry_key()))
    .sum()
}

fn armor_points(id: &str) -> u32 {
    let material = if id.contains("netherite") || id.contains("diamond") {
        4
    } else if id.contains("iron") {
        3
    } else if id.contains("chainmail") || id.contains("golden") {
        2
    } else if id.contains("leather") {
        1
    } else {
        0
    };
    let slot = if id.contains("chestplate") { 2 } else { 1 };
    material * slot
}

struct DeathHandler(Arc<App>);
impl EventHandler<EntityDeathEvent> for DeathHandler {
    fn handle(
        &self,
        server: Server,
        event: EventData<EntityDeathEvent>,
    ) -> EventData<EntityDeathEvent> {
        if let Some(hit) = self
            .0
            .last_hits
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&event.entity_id)
            && let Some(player) = server
                .get_all_players()
                .into_iter()
                .find(|p| App::player_id(p) == hit.player_id)
        {
            self.0
                .progress(&player, ObjectiveKind::Kill, &hit.target, 1);
            self.0.award_xp(&player, 10);
        }
        self.0
            .spawned_types
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&event.entity_id);
        event
    }
}

struct ClickHandler(Arc<App>);
impl EventHandler<InventoryClickEvent> for ClickHandler {
    fn handle(
        &self,
        _server: Server,
        mut event: EventData<InventoryClickEvent>,
    ) -> EventData<InventoryClickEvent> {
        let id = App::player_id(&event.player);
        let has_open_view = self
            .0
            .gui_views
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(&id);
        if has_open_view
            && event
                .clicked_item
                .as_ref()
                .is_some_and(ui::is_java_menu_item)
        {
            event.cancelled = true;
            self.0.enqueue_ui(UiIntent::JavaClick {
                player_id: id,
                slot: event.raw_slot,
            });
        }
        event
    }
}

struct FormHandler(Arc<App>);
impl EventHandler<BedrockFormResponseEvent> for FormHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<BedrockFormResponseEvent>,
    ) -> EventData<BedrockFormResponseEvent> {
        self.0.enqueue_ui(UiIntent::BedrockForm {
            player_id: App::player_id(&event.player),
            form_id: event.form_id,
            response_data: event.response_data.clone(),
        });
        event
    }
}

fn process_ui_intents(app: &App, server: &Server) {
    let intents = {
        let mut queue = app.ui_intents.lock().unwrap_or_else(|e| e.into_inner());
        let count = queue.len().min(64);
        queue.drain(..count).collect::<Vec<_>>()
    };
    for intent in intents {
        let player_id = match &intent {
            UiIntent::JavaClick { player_id, .. } | UiIntent::BedrockForm { player_id, .. } => {
                player_id
            }
        };
        let Some(player) = server
            .get_all_players()
            .into_iter()
            .find(|p| App::player_id(p) == *player_id)
        else {
            continue;
        };
        match intent {
            UiIntent::JavaClick { slot, .. } => {
                if let Some(next) = ui::handle_java_click(app, &player, slot) {
                    ui::open_java_view(app, &player, next);
                }
            }
            UiIntent::BedrockForm {
                form_id,
                response_data,
                ..
            } => ui::handle_bedrock_response(
                app,
                &player,
                form_id,
                FormResponse::parse(response_data),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalogue_has_fifty_unique_valid_quests() {
        let book: model::QuestBook = toml::from_str(include_str!("../config/quests.toml")).unwrap();
        assert_eq!(book.quests.len(), 50);
        let ids: std::collections::HashSet<_> = book.quests.iter().map(|q| &q.id).collect();
        assert_eq!(ids.len(), 50);
        assert!(book.quests.iter().all(|q| q.objective.amount > 0));
        assert!(
            book.quests
                .windows(2)
                .all(|pair| pair[1].required_level >= pair[0].required_level)
        );
    }

    #[test]
    fn safe_zone_includes_xyz_boundaries() {
        let book: model::SafeZoneBook =
            toml::from_str(include_str!("../config/safe_zones.toml")).unwrap();
        let zone = &book.zones[0];
        assert!(zone.contains("minecraft:overworld", -64.0, -64.0, 64.0));
        assert!(zone.contains("minecraft:overworld", 64.0, 320.0, -64.0));
        assert!(!zone.contains("minecraft:overworld", 65.0, 100.0, 0.0));
        assert!(!zone.contains("minecraft:the_nether", 0.0, 64.0, 0.0));
    }

    #[test]
    fn identifiers_are_normalized() {
        assert_eq!(normalize_id("Zombie"), "minecraft:zombie");
        assert_eq!(normalize_id("minecraft:Iron_Ore"), "minecraft:iron_ore");
    }

    #[test]
    fn permission_nodes_use_the_exact_plugin_namespace() {
        assert!(PERMISSION_TALES.starts_with("CalabazaTales:"));
        assert!(PERMISSION_ADMIN.starts_with("CalabazaTales:"));
    }

    #[test]
    fn java_pack_metadata_targets_resource_pack_88_0() {
        let metadata: serde_json::Value =
            serde_json::from_str(include_str!("../resource-pack-java/pack.mcmeta")).unwrap();
        assert_eq!(metadata["pack"]["min_format"], serde_json::json!([88, 0]));
        assert_eq!(metadata["pack"]["max_format"], serde_json::json!([88, 0]));
        assert!(metadata["pack"].get("pack_format").is_none());
    }
}
