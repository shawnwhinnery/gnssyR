# CLAUDE.md — game/src/scenes/level_select/

## Purpose

Placeholder for a future level selection screen. Not yet reachable from `MainMenuScene`.

## File

`mod.rs` — stub implementation.

## Current State

- Displays "coming soon" text via egui.
- Esc returns to `MainMenuScene`.
- No `World`, no physics, no simulation.

## When to expand

Wire into `MainMenuScene` once there are two or more distinct playable levels. Until then, leave untouched — `Level1Scene` is launched directly from the menu.
