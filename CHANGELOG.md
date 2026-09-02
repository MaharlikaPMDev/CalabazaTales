# Changelog

## 0.1.6 — 2026-09-02

- Rebuilt against the current Pumpkin plugin API ABI used by Pumpkin 0.1.0-dev+26.2-26.40, fixing `handle-event` export type checking during plugin initialization.

## 0.1.5 — 2026-09-01

- Removed all quest mutation, persistence, messaging, scheduler registration, and GUI opening from Java inventory and Bedrock Form callbacks; callbacks now only enqueue bounded intents for a pre-registered tick worker.
- Added the read-only `calabazatales.ipc` v1 `active_quest` action for CalabazaBoard and other plugins.

## 0.1.4 — 2026-08-28

- Deferred Java GUI refreshes until the next server tick so Pumpkin applies click cancellation to the original screen before it is replaced.
- Coalesced rapid menu clicks to prevent overlapping refreshes and hotbar-number item desynchronization.

## 0.1.3 — 2026-08-28

- Fixed a Pumpkin WASM re-entrancy deadlock that froze the server when Java players clicked items in the `/tales` GUI.
- Marked Java menu items with private custom data so stale menu state cannot intercept clicks in unrelated inventories.

## 0.1.2 — 2026-08-28

- Fixed the Java 26.2 resource-pack metadata to use the required `min_format` and `max_format` fields with resource-pack version `[88, 0]`.

## 0.1.1 — 2026-08-28

- Fixed Pumpkin permission nodes to use the required, case-sensitive `CalabazaTales:` plugin namespace.

## 0.1.0 — 2026-08-28

- Added configurable X/Y/Z cuboid safe zones.
- Added Java inventory menus and Bedrock native forms.
- Added 50 escalating quests stored in `quests.toml`.
- Added persistent levels, XP, Damage, Defense, Speed, and Vitality.
- Added Dragon Seed (`Ds`) currency.
- Added target health/armor boss bar.
- Added original Java 26.2 and Bedrock MMORPG HUD packs.
