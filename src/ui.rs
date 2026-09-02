use crate::{app::App, model::MenuView};
use pumpkin_plugin_api::{Player, forms::SimpleFormBuilder, text::TextComponent};

const QUESTS_PER_PAGE: usize = 10;

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
    let max_page = quests.len().saturating_sub(1) / QUESTS_PER_PAGE;
    let page = page.min(max_page);
    player.send_system_message(
        TextComponent::from_legacy_string(&format!(
            "§6§l{} §r§7({}/{})",
            cfg.quests.menu_title,
            page + 1,
            max_page + 1
        )),
        false,
    );
    player.send_system_message(
        TextComponent::from_legacy_string(&format!(
            "§7Level {} • {} {} • {} attribute points",
            state.level, state.dragon_seeds, cfg.currency.symbol, state.unspent_points
        )),
        false,
    );
    for quest in quests
        .iter()
        .skip(page * QUESTS_PER_PAGE)
        .take(QUESTS_PER_PAGE)
    {
        let progress = state.active.get(&quest.id).copied().unwrap_or(0);
        let unlocked = state.completed.contains(&quest.id)
            || state.active.contains_key(&quest.id)
            || (state.level >= quest.required_level
                && quest
                    .prerequisite
                    .as_ref()
                    .is_none_or(|required| state.completed.contains(required)));
        let status = quest_status(&state, quest, progress);
        let text = if unlocked {
            format!("§f[{}] §7{} • {status}", quest.title, quest.difficulty)
        } else {
            format!("§8[{}] {} • {status}", quest.title, quest.difficulty)
        };
        let mut line = TextComponent::from_legacy_string(&text);
        if unlocked {
            line = line
                .click_run_command(&format!("/tales quest {}", quest.id))
                .hover_show_text(TextComponent::text("Click to view quest details"));
        }
        player.send_system_message(line, false);
    }
    let mut navigation = TextComponent::text("");
    if page > 0 {
        navigation = navigation.add_child(
            TextComponent::from_legacy_string("§e[Previous]")
                .click_run_command(&format!("/tales page {}", page)),
        );
    }
    navigation = navigation.add_child(
        TextComponent::from_legacy_string(" §b[Attributes]").click_run_command("/tales attributes"),
    );
    if page < max_page {
        navigation = navigation.add_child(
            TextComponent::from_legacy_string(" §e[Next]")
                .click_run_command(&format!("/tales page {}", page + 2)),
        );
    }
    player.send_system_message(navigation, false);
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
    let start = page * QUESTS_PER_PAGE;
    for quest in quests.iter().skip(start).take(QUESTS_PER_PAGE) {
        let progress = state.active.get(&quest.id).copied().unwrap_or(0);
        form = form.button(
            TextComponent::text(&format!(
                "{}\n{} • {progress}/{}",
                quest.title,
                quest_status(&state, quest, progress),
                quest.objective.amount
            )),
            None,
        );
    }
    form = form.button(TextComponent::text("Character Attributes"), None);
    if page > 0 {
        form = form.button(TextComponent::text("Previous Page"), None);
    }
    if start + QUESTS_PER_PAGE < quests.len() {
        form = form.button(TextComponent::text("Next Page"), None);
    }
    let form_id = bedrock.open_form(form.build());
    app.bedrock_forms
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(form_id, (id, MenuView::Quests(page)));
}

pub fn open_quest_detail(app: &App, player: &Player, index: usize, page: usize) {
    app.ensure_player(player);
    let id = App::player_id(player);
    let quests = app.quests.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let Some(quest) = quests.get(index) else {
        player.send_system_message(TextComponent::text("That quest no longer exists."), false);
        return;
    };
    let state = app.snapshot(&id);
    let progress = state.active.get(&quest.id).copied().unwrap_or(0);
    let cfg = app.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let details = format!(
        "{}\n\n{}\n\nDifficulty: {}\nObjective: {:?} {} — {progress}/{}\nRequired level: {}\nReward: {} XP + {} {}\nStatus: {}",
        quest.title,
        quest.description,
        quest.difficulty,
        quest.objective.kind,
        quest.objective.target,
        quest.objective.amount,
        quest.required_level,
        quest.reward_xp,
        quest.reward_ds,
        cfg.currency.symbol,
        quest_status(&state, quest, progress)
    );
    if let Some(bedrock) = player.as_bedrock() {
        let mut form = SimpleFormBuilder::new(
            TextComponent::text(&quest.title),
            TextComponent::text(&details),
        );
        if state.active.contains_key(&quest.id) {
            if progress >= quest.objective.amount {
                form = form.button(TextComponent::text("Claim Reward"), None);
            } else {
                form = form.button(TextComponent::text("Cancel Quest"), None);
            }
        } else if quest_is_unlocked(&state, quest) && !state.completed.contains(&quest.id) {
            form = form.button(TextComponent::text("Accept Quest"), None);
        }
        form = form.button(TextComponent::text("Back"), None);
        let form_id = bedrock.open_form(form.build());
        app.bedrock_forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(form_id, (id, MenuView::QuestDetail { page, index }));
        return;
    }

    player.send_system_message(
        TextComponent::from_legacy_string(&format!("§6§l{}", quest.title)),
        false,
    );
    for line in details.lines().skip(1) {
        if !line.is_empty() {
            player.send_system_message(
                TextComponent::from_legacy_string(&format!("§7{line}")),
                false,
            );
        }
    }
    let mut actions = TextComponent::text("");
    if state.active.contains_key(&quest.id) {
        if progress >= quest.objective.amount {
            actions = actions.add_child(
                TextComponent::from_legacy_string("§a[Claim Reward]")
                    .click_run_command(&format!("/tales claim {}", quest.id)),
            );
        } else {
            actions = actions.add_child(
                TextComponent::from_legacy_string("§c[Cancel Quest]")
                    .click_run_command(&format!("/tales cancel {}", quest.id)),
            );
        }
    } else if quest_is_unlocked(&state, quest) && !state.completed.contains(&quest.id) {
        actions = actions
            .add_child(
                TextComponent::from_legacy_string("§a[Accept]")
                    .click_run_command(&format!("/tales accept {}", quest.id)),
            )
            .add_child(
                TextComponent::from_legacy_string(" §c[Decline]")
                    .click_run_command(&format!("/tales page {}", page + 1)),
            );
    }
    actions = actions.add_child(
        TextComponent::from_legacy_string(" §e[Back]")
            .click_run_command(&format!("/tales page {}", page + 1)),
    );
    player.send_system_message(actions, false);
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
            TextComponent::text(&format!("Damage {}", state.attributes.damage)),
            None,
        )
        .button(
            TextComponent::text(&format!("Defense {}", state.attributes.defense)),
            None,
        )
        .button(
            TextComponent::text(&format!("Speed {}", state.attributes.speed)),
            None,
        )
        .button(
            TextComponent::text(&format!("Vitality {}", state.attributes.vitality)),
            None,
        )
        .button(TextComponent::text("Back to Quest Journal"), None)
        .build();
        let form_id = bedrock.open_form(form);
        app.bedrock_forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(form_id, (id, MenuView::Attributes));
        return;
    }
    player.send_system_message(
        TextComponent::from_legacy_string(&format!(
            "§6§l{} §r§7• Level {} • XP {}/{} • {} points",
            cfg.quests.attributes_title,
            state.level,
            state.xp,
            app.xp_needed(state.level),
            state.unspent_points
        )),
        false,
    );
    for (index, label, value) in [
        (0, "Damage", state.attributes.damage),
        (1, "Defense", state.attributes.defense),
        (2, "Speed", state.attributes.speed),
        (3, "Vitality", state.attributes.vitality),
    ] {
        player.send_system_message(
            TextComponent::from_legacy_string(&format!(
                "§b[{label} {value}] §7Click to spend 1 point"
            ))
            .click_run_command(&format!("/tales spend {index}")),
            false,
        );
    }
    player.send_system_message(
        TextComponent::from_legacy_string("§e[Back to Quest Journal]").click_run_command("/tales"),
        false,
    );
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
            let page_count = quests_len
                .saturating_sub(page * QUESTS_PER_PAGE)
                .min(QUESTS_PER_PAGE);
            let button = button as usize;
            if button < page_count {
                open_quest_detail(app, player, page * QUESTS_PER_PAGE + button, page);
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
                if page * QUESTS_PER_PAGE + QUESTS_PER_PAGE < quests_len && button == cursor {
                    open_main(app, player, page + 1);
                }
            }
        }
        MenuView::QuestDetail { page, index } => {
            let quests = app.quests.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let Some(quest) = quests.get(index) else {
                open_main(app, player, page);
                return;
            };
            let state = app.snapshot(&App::player_id(player));
            let progress = state.active.get(&quest.id).copied().unwrap_or(0);
            let has_action = (state.active.contains_key(&quest.id)
                || (quest_is_unlocked(&state, quest) && !state.completed.contains(&quest.id)))
                && !state.completed.contains(&quest.id);
            if button == 0 && has_action {
                let message = if state.active.contains_key(&quest.id) {
                    if progress >= quest.objective.amount {
                        app.claim_quest(player, index)
                    } else {
                        app.cancel_quest(player, index)
                    }
                } else {
                    app.accept_quest(player, index)
                };
                player.send_system_message(TextComponent::text(&message), false);
                open_quest_detail(app, player, index, page);
            } else {
                open_main(app, player, page);
            }
        }
        MenuView::Attributes => match button {
            0..=3 => spend_and_refresh(app, player, button),
            4 => open_main(app, player, 0),
            _ => {}
        },
    }
}

fn quest_is_unlocked(state: &crate::model::PlayerState, quest: &crate::model::Quest) -> bool {
    state.level >= quest.required_level
        && quest
            .prerequisite
            .as_ref()
            .is_none_or(|required| state.completed.contains(required))
}

fn quest_status(
    state: &crate::model::PlayerState,
    quest: &crate::model::Quest,
    progress: u64,
) -> &'static str {
    if state.completed.contains(&quest.id) {
        "COMPLETED"
    } else if state.active.contains_key(&quest.id) && progress >= quest.objective.amount {
        "READY TO CLAIM"
    } else if state.active.contains_key(&quest.id) {
        "ACTIVE"
    } else if quest_is_unlocked(state, quest) {
        "AVAILABLE"
    } else {
        "LOCKED"
    }
}
