use crate::{app::App, model::MenuView};
use pumpkin_plugin_api::{
    ItemStack, Player, Screen, forms::SimpleFormBuilder, gui::Gui, text::TextComponent,
};

fn menu_item(id: &str, name: String, lore: Vec<String>) -> ItemStack {
    let item = ItemStack::new(id, 1);
    item.set_custom_name(Some(TextComponent::text(&name)));
    item.set_lore(
        lore.into_iter()
            .map(|line| TextComponent::text(&line))
            .collect(),
    );
    item
}

pub fn open_main(app: &App, player: &Player, page: usize) {
    app.ensure_player(player);
    if let Some(bedrock) = player.as_bedrock() {
        open_bedrock_quests(app, player, bedrock, page);
    } else {
        open_java_quests(app, player, page);
    }
}

fn open_java_quests(app: &App, player: &Player, page: usize) {
    let id = App::player_id(player);
    let quests = app.quests.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let state = app.snapshot(&id);
    let cfg = app.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let gui = Gui::new(
        Screen::Generic9x6,
        TextComponent::text(&cfg.quests.menu_title),
    );
    gui.set_allow_grab_items(false);
    gui.set_allow_put_items(false);
    let start = page * 45;
    for (slot, quest) in quests.iter().skip(start).take(45).enumerate() {
        let progress = state.active.get(&quest.id).copied().unwrap_or(0);
        let (icon, status) = if state.completed.contains(&quest.id) {
            ("minecraft:lime_dye", "COMPLETED")
        } else if state.active.contains_key(&quest.id) && progress >= quest.objective.amount {
            ("minecraft:emerald", "READY TO CLAIM")
        } else if state.active.contains_key(&quest.id) {
            ("minecraft:writable_book", "ACTIVE")
        } else if state.level < quest.required_level {
            ("minecraft:gray_dye", "LEVEL LOCKED")
        } else {
            ("minecraft:book", "AVAILABLE")
        };
        gui.set_item(
            slot as u32,
            menu_item(
                icon,
                format!("{} [{}]", quest.title, quest.difficulty),
                vec![
                    quest.description.clone(),
                    format!("Objective: {} / {}", progress, quest.objective.amount),
                    format!("Requires level: {}", quest.required_level),
                    format!(
                        "Reward: {} XP + {} {}",
                        quest.reward_xp, quest.reward_ds, cfg.currency.symbol
                    ),
                    format!("Status: {status}"),
                    "Click to accept, inspect, or claim.".into(),
                ],
            ),
        );
    }
    if page > 0 {
        gui.set_item(
            45,
            menu_item("minecraft:arrow", "Previous Page".into(), vec![]),
        );
    }
    gui.set_item(
        49,
        menu_item(
            "minecraft:nether_star",
            "Character Attributes".into(),
            vec![format!(
                "Level {} • {} {}",
                state.level, state.dragon_seeds, cfg.currency.symbol
            )],
        ),
    );
    if start + 45 < quests.len() {
        gui.set_item(53, menu_item("minecraft:arrow", "Next Page".into(), vec![]));
    }
    app.gui_views
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(id, MenuView::Quests(page));
    player.open_gui(gui);
}

fn open_bedrock_quests(
    app: &App,
    player: &Player,
    bedrock: pumpkin_plugin_api::player::BedrockPlayer,
    page: usize,
) {
    let id = App::player_id(player);
    let quests = app.quests.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let state = app.snapshot(&id);
    let cfg = app.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut form = SimpleFormBuilder::new(
        TextComponent::text(&cfg.quests.menu_title),
        TextComponent::text(&format!(
            "Level {}  •  {} {}  •  {} points",
            state.level, state.dragon_seeds, cfg.currency.symbol, state.unspent_points
        )),
    );
    let start = page * 10;
    for quest in quests.iter().skip(start).take(10) {
        let progress = state.active.get(&quest.id).copied().unwrap_or(0);
        let status = if state.completed.contains(&quest.id) {
            "✓"
        } else if progress >= quest.objective.amount && state.active.contains_key(&quest.id) {
            "★"
        } else if state.active.contains_key(&quest.id) {
            "◈"
        } else {
            "◇"
        };
        form = form.button(
            TextComponent::text(&format!(
                "{status} {}\n{progress}/{} • {} XP • {} Ds",
                quest.title, quest.objective.amount, quest.reward_xp, quest.reward_ds
            )),
            None,
        );
    }
    form = form.button(TextComponent::text("Character Attributes"), None);
    if page > 0 {
        form = form.button(TextComponent::text("Previous Page"), None);
    }
    if start + 10 < quests.len() {
        form = form.button(TextComponent::text("Next Page"), None);
    }
    let form_id = bedrock.open_form(form.build());
    app.bedrock_forms
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(form_id, (id, MenuView::Quests(page)));
}

pub fn open_attributes(app: &App, player: &Player) {
    let id = App::player_id(player);
    let state = app.snapshot(&id);
    let cfg = app.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(bedrock) = player.as_bedrock() {
        let form = SimpleFormBuilder::new(
            TextComponent::text(&cfg.quests.attributes_title),
            TextComponent::text(&format!(
                "Level {} • XP {}/{} • {} {} ({}) • {} points available",
                state.level,
                state.xp,
                app.xp_needed(state.level),
                state.dragon_seeds,
                cfg.currency.symbol,
                cfg.currency.name,
                state.unspent_points
            )),
        )
        .button(
            TextComponent::text(&format!(
                "⚔ Damage {}\n+{:.1}% outgoing damage",
                state.attributes.damage,
                state.attributes.damage as f32 * cfg.attributes.damage_per_point * 100.0
            )),
            None,
        )
        .button(
            TextComponent::text(&format!(
                "🛡 Defense {}\n-{:.1}% incoming damage",
                state.attributes.defense,
                state.attributes.defense as f32 * cfg.attributes.defense_per_point * 100.0
            )),
            None,
        )
        .button(
            TextComponent::text(&format!(
                "➤ Speed {}\n+{:.1}% movement",
                state.attributes.speed,
                state.attributes.speed as f32 * cfg.attributes.speed_per_point * 100.0
            )),
            None,
        )
        .button(
            TextComponent::text(&format!(
                "❤ Vitality {}\n+{:.1} max health",
                state.attributes.vitality,
                state.attributes.vitality as f32 * cfg.attributes.health_per_point
            )),
            None,
        )
        .button(TextComponent::text("Back to Quest Journal"), None)
        .build();
        let form_id = bedrock.open_form(form);
        app.bedrock_forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(form_id, (id, MenuView::Attributes));
    } else {
        let gui = Gui::new(
            Screen::Generic9x3,
            TextComponent::text(&cfg.quests.attributes_title),
        );
        gui.set_allow_grab_items(false);
        gui.set_allow_put_items(false);
        gui.set_item(
            4,
            menu_item(
                "minecraft:nether_star",
                format!("Level {}", state.level),
                vec![
                    format!("XP: {}/{}", state.xp, app.xp_needed(state.level)),
                    format!("Balance: {} {}", state.dragon_seeds, cfg.currency.symbol),
                    format!("Unspent points: {}", state.unspent_points),
                ],
            ),
        );
        gui.set_item(
            10,
            menu_item(
                "minecraft:iron_sword",
                format!("Damage • {}", state.attributes.damage),
                vec!["Click to spend 1 point.".into()],
            ),
        );
        gui.set_item(
            12,
            menu_item(
                "minecraft:shield",
                format!("Defense • {}", state.attributes.defense),
                vec!["Click to spend 1 point.".into()],
            ),
        );
        gui.set_item(
            14,
            menu_item(
                "minecraft:rabbit_foot",
                format!("Speed • {}", state.attributes.speed),
                vec!["Click to spend 1 point.".into()],
            ),
        );
        gui.set_item(
            16,
            menu_item(
                "minecraft:glistering_melon_slice",
                format!("Vitality • {}", state.attributes.vitality),
                vec!["Click to spend 1 point.".into()],
            ),
        );
        gui.set_item(
            22,
            menu_item("minecraft:book", "Back to Quest Journal".into(), vec![]),
        );
        app.gui_views
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, MenuView::Attributes);
        player.open_gui(gui);
    }
}

pub fn handle_java_click(app: &App, player: &Player, slot: i16) {
    if slot < 0 {
        return;
    }
    let id = App::player_id(player);
    let view = app
        .gui_views
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&id)
        .copied();
    match view {
        Some(MenuView::Quests(page)) => match slot {
            0..=44 => {
                let index = page * 45 + slot as usize;
                player.send_system_message(
                    TextComponent::text(&app.activate_or_claim(player, index)),
                    false,
                );
                open_main(app, player, page);
            }
            45 if page > 0 => open_main(app, player, page - 1),
            49 => open_attributes(app, player),
            53 => open_main(app, player, page + 1),
            _ => {}
        },
        Some(MenuView::Attributes) => match slot {
            10 => spend_and_refresh(app, player, 0),
            12 => spend_and_refresh(app, player, 1),
            14 => spend_and_refresh(app, player, 2),
            16 => spend_and_refresh(app, player, 3),
            22 => open_main(app, player, 0),
            _ => {}
        },
        None => {}
    }
}

fn spend_and_refresh(app: &App, player: &Player, index: u32) {
    player.send_system_message(
        TextComponent::text(&app.spend_attribute(player, index)),
        false,
    );
    open_attributes(app, player);
}

pub fn handle_bedrock_response(
    app: &App,
    player: &Player,
    form_id: u32,
    response: pumpkin_plugin_api::forms::FormResponse,
) {
    let Some((expected_id, view)) = app
        .bedrock_forms
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&form_id)
    else {
        return;
    };
    if expected_id != App::player_id(player) {
        return;
    }
    let pumpkin_plugin_api::forms::FormResponse::Simple(button) = response else {
        return;
    };
    match view {
        MenuView::Quests(page) => {
            let quests_len = app.quests.lock().unwrap_or_else(|e| e.into_inner()).len();
            let page_count = quests_len.saturating_sub(page * 10).min(10);
            let button = button as usize;
            if button < page_count {
                player.send_system_message(
                    TextComponent::text(&app.activate_or_claim(player, page * 10 + button)),
                    false,
                );
                open_main(app, player, page);
            } else if button == page_count {
                open_attributes(app, player);
            } else {
                let mut cursor = page_count + 1;
                if page > 0 {
                    if button == cursor {
                        open_main(app, player, page - 1);
                        return;
                    }
                    cursor += 1;
                }
                if page * 10 + 10 < quests_len && button == cursor {
                    open_main(app, player, page + 1);
                }
            }
        }
        MenuView::Attributes => match button {
            0..=3 => spend_and_refresh(app, player, button),
            4 => open_main(app, player, 0),
            _ => {}
        },
    }
}
