# FORGE.md — Forgemaster NPC & Crafting System

## Feature Overview

A friendly NPC (the Forgemaster) accepts scraps from the player's inventory. When the player contributes exactly 100 scraps, the Forgemaster produces a `ModPart` — a weapon modifier whose color is the weighted average of the scraps used and whose shape is a polygon influenced by the shapes of those scraps.

---

## Architecture Decisions

- **Friendly NPCs** are a new trait (`FriendlyNpc`), parallel to `Enemy`, owned by `World` as `Vec<Box<dyn FriendlyNpc>>`.
- **Interaction state** (dialog open/closed, contribution selections) lives in the Scene, not in World. World only signals *"player is near NPC of kind X"* — the scene decides what to show.
- **Dialog follows the `PauseState` pattern**: a `RefCell<InteractionState>` embedded in `SandboxScene`, rendered via `draw_ui`.
- **Shape generation** uses weighted polygon blending — each scrap shape is resampled to 12 vertices and averaged by contribution count, with small per-vertex noise for organic feel.
- **Forge action** is split across the frame boundary: `draw_ui` sets a `forge_requested: Cell<bool>` flag; `tick()` consumes it to mutate inventory (keeps draw_ui safely `&self`).

---

## Implementation Status

| Step | Description | Status |
|------|-------------|--------|
| 1 | `KeyCode::E` in input crate + window mapping | ✅ Done |
| 2 | `npc` module — `FriendlyNpc` trait + `Forgemaster` | ✅ Done |
| 3 | `mod_part` module — `ModPart` struct + `forge()` fn | ✅ Done |
| 4 | Wire NPCs into `World` | ✅ Done |
| 5 | `InteractionState` + forge dialog in `SandboxScene` | ✅ Done |
| 6 | Sandbox scrap spawning controls | ✅ Done |
| — | All 48 tests passing | ✅ Done |

---

## Files Changed / Created

| Action | File |
|--------|------|
| Modified | `crates/input/src/event.rs` — added `KeyCode::E` |
| Modified | `crates/window/src/app.rs` — mapped `KeyE` → `Button::Key(KeyCode::E)` |
| Modified | `crates/game/src/lib.rs` — exposed `mod_part` and `npc` modules |
| Modified | `crates/game/src/world.rs` — added `npcs` vec, `spawn_forgemaster`, `spawn_scrap`, `nearest_interactable_npc` |
| Modified | `crates/game/src/scrap.rs` — added `Inventory::remove` |
| Modified | `crates/game/src/scenes/sandbox/mod.rs` — full rewrite with forge dialog, interaction state, scrap spawn controls |
| Created | `crates/game/src/npc/mod.rs` — `FriendlyNpc` trait + `NpcKind` enum |
| Created | `crates/game/src/npc/forgemaster.rs` — Forgemaster NPC (amber hexagon, static body) |
| Created | `crates/game/src/mod_part.rs` — `ModPart`, `forge()`, `draw_mod_part()` |

---

## How It Works

### In the sandbox

- The Forgemaster spawns at `(2.5, 2.5)` when the sandbox scene loads — walk up to it.
- When within 1.8 world units, an `[E]  Forgemaster` prompt appears on screen.
- Press **E** to open the forge dialog.

### Forge dialog

- 8×4 grid shows each (color × shape) slot with current inventory count.
- Drag the value in any cell to select how many of that scrap type to contribute (clamped to what you have).
- The running total is shown below the grid. The **Forge** button enables only when the total reaches exactly 100.
- Press **Escape** or **Cancel** to close without forging.

### Forging

- On **Forge**, the selected scraps are deducted from inventory.
- A `ModPart` is produced with:
  - **`avg_color`** — weighted RGB mean of all contributing scrap colors.
  - **`shape`** — 12-vertex polygon blended from the contributing shape types (weighted by count), with ±10% per-vertex noise for organic variation. Dominant shape type dominates the silhouette.
- A result panel shows the part's color swatch and hex code.

### Spawning scraps (sandbox)

- Open the Inventory tab in the Sandbox panel.
- Select a color (row of colored buttons) and a shape (row of shape symbols).
- Click **Spawn Scrap** — a scrap of that type appears near the player.

---

## Implementation Plan (original, for reference)

### 1. Input — add `KeyCode::E`

- `crates/input/src/event.rs` — add `E` to the `KeyCode` enum.
- `crates/window/src/app.rs` — map `winit::keyboard::KeyCode::KeyE` → `Button::Key(input::event::KeyCode::E)`.

### 2. `npc` module (new)

**`crates/game/src/npc/mod.rs`**

```rust
pub trait FriendlyNpc {
    fn body(&self) -> BodyHandle;
    fn interaction_radius(&self) -> f32;
    fn kind(&self) -> NpcKind;
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn GraphicsDriver, camera: &Camera);
}

pub enum NpcKind { Forgemaster }
```

**`crates/game/src/npc/forgemaster.rs`**
Static body (infinite mass). Visual: a tall hexagon in warm amber with a darker stroke — readable as "friendly" at a glance.

### 3. `mod_part` module (new)

**`crates/game/src/mod_part.rs`**

```rust
pub struct ModPart {
    pub avg_color: [f32; 3],
    pub shape: Vec<Vec2>,   // polygon vertices, ready for draw_shape()
}

pub fn forge(contributions: &[(ScrapColor, ScrapShape, u16)]) -> Option<ModPart> { ... }
```

**Color:** weighted RGB mean of each `ScrapColor` value multiplied by its contribution count.

**Shape:** Each scrap shape type is resampled to 12 vertices by walking its perimeter at equal arc lengths. Per-vertex positions are weight-averaged across all contributing shape types. A small per-vertex random noise (±10% of radius) is added for organic feel. Result leans toward whichever shape type dominated.

### 4. `World` additions

- `pub npcs: Vec<Box<dyn FriendlyNpc>>`
- `pub fn spawn_forgemaster(&mut self, pos: Vec2)`
- `pub fn nearest_interactable_npc(&self) -> Option<NpcKind>`
- `pub fn spawn_scrap(&mut self, pos: Vec2, color: ScrapColor, shape: ScrapShape)`
- `World::draw` — draws NPCs

### 5. `InteractionState` + forge dialog

```rust
enum InteractionState { None, ForgeDialog(ForgeContribution) }
```

`SandboxScene` gains `interaction: RefCell<InteractionState>` + `forge_requested: Cell<bool>`.

`tick()` detects E-key + proximity → opens dialog; consumes `forge_requested` → mutates inventory.

`draw_ui()` renders the dialog and sets `forge_requested` on Forge button click.

### 6. Sandbox — scrap spawning

Color picker + shape picker + Spawn Scrap button in the Inventory tab. Spawns a scrap near P1.
