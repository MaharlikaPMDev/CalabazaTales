# CalabazaTales Roadmap

## TODO — SAO-style client HUD (future session)

- Prototype an optional client-side Java mod (Fabric/NeoForge decision pending) for the animated Sword Art Online-style health, armor, target HP, level, XP, and Dragon Seed HUD.
- Keep the mod server-optional: it must connect to Pumpkin without requiring server-side mod installation, using plugin-provided packets/scoreboard or boss-bar data where available.
- Define a versioned bridge protocol between the CalabazaTales WASM plugin and the client mod, with a vanilla/resource-pack fallback when the mod is absent.
- Verify compatibility against Minecraft Java 26.2 and document the exact loader/API requirements.
- Do not promise equivalent arbitrary HUD overlays for Bedrock; maintain the Bedrock forms/resource-pack experience unless a supported Bedrock scripting/UI path is added later.

## 0.2 — Combat identity

- Weapon classes, active skills, cooldowns, mana, and configurable damage formulas.
- Java and Bedrock-safe combat animations with graceful vanilla fallbacks.
- Projectile-owner attribution and ranged quest credit when Pumpkin exposes the required event data.
- Status effects, critical hits, dodging, blocking, and floating combat text.

## 0.3 — Creatures and encounters

- Custom monster packs, elite affixes, level scaling, and configurable loot tables.
- Dungeon encounter definitions, boss phases, party contribution tracking, and instanced rewards.
- Target HUD threat, level, status-effect, and elemental-resistance indicators.

## 0.4 — Social MMO systems

- Parties, guilds, friend lists, shared quest progress, and guild banks.
- Trading and an auction house denominated in Dragon Seeds (`Ds`).
- NPC dialogue trees, shops, quest chains, and reputation factions.

## 0.5 — World progression

- Discoverable regions, map markers, fast travel, and per-region level ranges.
- Safe-zone visualization and in-game region editing tools.
- Daily, weekly, repeatable, branching, timed, escort, and exploration quests.

## 1.0 — Production hardening

- Database adapters, migration/versioning tools, backups, and multi-server synchronization.
- Performance profiling under large player counts and hostile-mob volumes.
- Admin web dashboard, audit logs, localization, accessibility review, and automated compatibility testing for supported Java/Bedrock releases.
