use std::cell::{Cell, RefCell};

use gfx::Color;
use glam::Vec2;
use input::{
    event::{Button, InputEvent, KeyCode},
    player::PlayerId,
};
use physics::{Body, Collider};

use crate::{
    loot,
    mod_part::{self, ModPart},
    namegen,
    npc::NpcKind,
    pause::PauseState,
    physics_layers,
    scrap::{ScrapColor, ScrapShape},
    weapon::{ProjectileBehavior, WeaponFiringState, WeaponStats},
    world::World,
};

use super::{Scene, SceneTransition};

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum SandboxTab {
    PrimaryWeapon,
    Enemies,
    Inventory,
}

fn primary_weapon_grid_section(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).strong());
    ui.label("");
    ui.end_row();
}

// ---------------------------------------------------------------------------
// Interaction / forge dialog state
// ---------------------------------------------------------------------------

const NUM_COLORS: usize = 8;
const NUM_SHAPES: usize = 4;
const NUM_KINDS: usize = NUM_COLORS * NUM_SHAPES;

/// How many of each inventory slot the player has selected to contribute.
#[derive(Clone)]
struct ForgeContribution {
    selected: [u16; NUM_KINDS],
}

impl ForgeContribution {
    fn zeroed() -> Self {
        Self {
            selected: [0; NUM_KINDS],
        }
    }

    fn total(&self) -> u32 {
        self.selected.iter().map(|&n| n as u32).sum()
    }

    fn get(&self, color: ScrapColor, shape: ScrapShape) -> u16 {
        self.selected[color as usize * NUM_SHAPES + shape as usize]
    }

    fn get_mut(&mut self, color: ScrapColor, shape: ScrapShape) -> &mut u16 {
        &mut self.selected[color as usize * NUM_SHAPES + shape as usize]
    }
}

enum InteractionState {
    None,
    ForgeDialog(ForgeContribution),
}

// ---------------------------------------------------------------------------
// SandboxScene
// ---------------------------------------------------------------------------

pub struct SandboxScene {
    world: World,
    pause: PauseState,
    weapon_editor: RefCell<(WeaponStats, ProjectileBehavior)>,
    weapon_name: RefCell<Option<String>>,
    slow_motion: Cell<bool>,
    enemy_spawn_requests: Cell<u32>,
    player_respawn_requested: Cell<bool>,
    random_weapon_requested: Cell<bool>,
    selected_tab: Cell<SandboxTab>,
    // Scrap spawning
    spawn_color: Cell<ScrapColor>,
    spawn_shape: Cell<ScrapShape>,
    scrap_spawn_request: Cell<bool>,
    // NPC interaction
    interaction: RefCell<InteractionState>,
    forge_requested: Cell<bool>,
    last_forged: RefCell<Option<ModPart>>,
    // Rarity distribution test
    rarity_test_requested: Cell<bool>,
    rarity_test_results: RefCell<Option<[u32; 5]>>,
}

impl SandboxScene {
    pub fn new() -> Self {
        let mut world = World::new();
        add_sandbox_walls(&mut world);
        world.spawn_forgemaster(Vec2::new(2.5, 2.5));
        Self {
            world,
            pause: PauseState::new(),
            weapon_editor: RefCell::new((WeaponStats::default(), ProjectileBehavior::default())),
            weapon_name: RefCell::new(None),
            slow_motion: Cell::new(false),
            enemy_spawn_requests: Cell::new(0),
            player_respawn_requested: Cell::new(false),
            random_weapon_requested: Cell::new(false),
            selected_tab: Cell::new(SandboxTab::PrimaryWeapon),
            spawn_color: Cell::new(ScrapColor::Red),
            spawn_shape: Cell::new(ScrapShape::Diamond),
            scrap_spawn_request: Cell::new(false),
            interaction: RefCell::new(InteractionState::None),
            forge_requested: Cell::new(false),
            last_forged: RefCell::new(None),
            rarity_test_requested: Cell::new(false),
            rarity_test_results: RefCell::new(None),
        }
    }
}

/// The four classic sandbox obstacle shapes (circle, rect, triangle, octagon).
fn add_sandbox_walls(world: &mut World) {
    let (cl, cm) = physics_layers::wall_collision();
    // Circle — right side.
    world.add_wall(
        Body {
            position: Vec2::new(3.0, 0.0),
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.3,
            collision_layers: cl,
            collision_mask: cm,
            collider: Collider::Circle { radius: 0.65 },
        },
        'C',
        Color::hex(0x7C4DFF99),
    );

    // Rectangle — left side.
    world.add_wall(
        Body {
            position: Vec2::new(-3.0, 0.0),
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.3,
            collision_layers: cl,
            collision_mask: cm,
            collider: Collider::Convex {
                vertices: vec![
                    Vec2::new(-0.8, -0.5),
                    Vec2::new(0.8, -0.5),
                    Vec2::new(0.8, 0.5),
                    Vec2::new(-0.8, 0.5),
                ],
            },
        },
        'R',
        Color::hex(0xFF6D0099),
    );

    // Triangle — top side (equilateral, CCW, circumradius 0.75).
    let r = 0.75_f32;
    world.add_wall(
        Body {
            position: Vec2::new(0.0, 3.0),
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.3,
            collision_layers: cl,
            collision_mask: cm,
            collider: Collider::Convex {
                vertices: vec![
                    Vec2::new(-r * 0.866, -r * 0.5),
                    Vec2::new(r * 0.866, -r * 0.5),
                    Vec2::new(0.0, r),
                ],
            },
        },
        'T',
        Color::hex(0x00BFA599),
    );

    // Octagon — bottom side (circumradius 0.75, CCW via increasing angle).
    let r = 0.75_f32;
    let oct_verts: Vec<Vec2> = (0..8)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            Vec2::new(r * angle.cos(), r * angle.sin())
        })
        .collect();
    world.add_wall(
        Body {
            position: Vec2::new(0.0, -3.0),
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.3,
            collision_layers: cl,
            collision_mask: cm,
            collider: Collider::Convex {
                vertices: oct_verts,
            },
        },
        'O',
        Color::hex(0xFFD60099),
    );
}

impl Default for SandboxScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for SandboxScene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        // Check for E-key press to open/close forge dialog before pause logic,
        // and Escape to close dialog before PauseState sees it.
        let e_pressed = events.iter().any(|e| {
            matches!(
                e,
                InputEvent::Button {
                    player: PlayerId::P1,
                    button: Button::Key(KeyCode::E),
                    pressed: true,
                }
            )
        });
        let esc_pressed = events.iter().any(|e| {
            matches!(
                e,
                InputEvent::Button {
                    button: Button::Key(KeyCode::Escape),
                    pressed: true,
                    ..
                }
            )
        });

        // If dialog is open, Escape closes it (consuming the event so pause doesn't trigger).
        let dialog_open = matches!(*self.interaction.borrow(), InteractionState::ForgeDialog(_));
        if dialog_open && esc_pressed {
            *self.interaction.borrow_mut() = InteractionState::None;
            // Don't forward escape to pause.
            let filtered: Vec<InputEvent> =
                events.iter().filter(|e| !is_escape(e)).cloned().collect();
            self.world.time_scale = if self.slow_motion.get() { 0.2 } else { 1.0 };
            if let Some(player) = self.world.players.first_mut() {
                let (stats, beh) = &*self.weapon_editor.borrow();
                player.weapon.stats = stats.clone();
                player.weapon.projectile_behavior = *beh;
            }
            self.world.tick(&filtered);
            return None;
        }

        self.pause.tick(events);
        self.world.time_scale = if self.slow_motion.get() { 0.2 } else { 1.0 };

        if self.random_weapon_requested.get() {
            self.random_weapon_requested.set(false);
            let mut rng = rand::thread_rng();
            let stats = loot::random_weapon_stats(&mut rng);
            let name = namegen::gun_name(&stats, &mut rng);
            self.weapon_editor.borrow_mut().0 = stats;
            *self.weapon_name.borrow_mut() = Some(name);
        }

        if self.rarity_test_requested.get() {
            self.rarity_test_requested.set(false);
            let mut rng = rand::thread_rng();
            // counts: [common, uncommon, rare, epic, mythic]
            let mut counts = [0u32; 5];
            for _ in 0..1000 {
                let stats = loot::random_weapon_stats(&mut rng);
                let score = loot::WeaponStatRarities::from_stats(&stats).overall_score();
                let tier = if score >= 0.80 { 4 } else if score >= 0.60 { 3 } else if score >= 0.40 { 2 } else if score >= 0.20 { 1 } else { 0 };
                counts[tier] += 1;
            }
            println!("[rarity test] common={} uncommon={} rare={} epic={} mythic={}", counts[0], counts[1], counts[2], counts[3], counts[4]);
            *self.rarity_test_results.borrow_mut() = Some(counts);
        }

        if let Some(player) = self.world.players.first_mut() {
            let (stats, beh) = &*self.weapon_editor.borrow();
            player.weapon.stats = stats.clone();
            player.weapon.projectile_behavior = *beh;
        }

        if self.player_respawn_requested.get() {
            self.player_respawn_requested.set(false);
            self.world.respawn_player(0);
        }

        let n = self.enemy_spawn_requests.get();
        if n > 0 {
            self.enemy_spawn_requests.set(0);
            let angle_step = std::f32::consts::TAU / n as f32;
            for i in 0..n {
                let angle = angle_step * i as f32;
                let pos = Vec2::new(angle.cos(), angle.sin()) * 4.0;
                self.world.spawn_enemy(pos);
            }
        }

        // Consume a forge request produced by draw_ui.
        if self.forge_requested.get() {
            self.forge_requested.set(false);
            let mut state = self.interaction.borrow_mut();
            if let InteractionState::ForgeDialog(ref contribution) = *state {
                let mut list = Vec::new();
                for (color, _, _) in COLORS {
                    for (_, shape) in SHAPES {
                        let n = contribution.get(color, shape);
                        if n > 0 {
                            self.world.inventory.remove(color, shape, n);
                            list.push((color, shape, n));
                        }
                    }
                }
                if let Some(part) = mod_part::forge(&list) {
                    *self.last_forged.borrow_mut() = Some(part);
                }
            }
            *state = InteractionState::None;
        }

        if self.scrap_spawn_request.get() {
            self.scrap_spawn_request.set(false);
            let color = self.spawn_color.get();
            let shape = self.spawn_shape.get();
            if let Some(player) = self.world.players.first() {
                let pos = self.world.physics.body(player.actor.body).position;
                let offset = Vec2::new(0.5, 0.5);
                self.world.spawn_scrap(pos + offset, color, shape);
            }
        }

        // Open forge dialog on E if near the Forgemaster and dialog not already open.
        if !self.pause.is_paused() && !dialog_open && e_pressed {
            if self.world.nearest_interactable_npc() == Some(NpcKind::Forgemaster) {
                *self.interaction.borrow_mut() =
                    InteractionState::ForgeDialog(ForgeContribution::zeroed());
            }
        }

        if !self.pause.is_paused() && !dialog_open {
            self.world.tick(events);
        }
        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        self.world.draw(driver);
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        {
            let editor = self.weapon_editor.borrow();
            let (stats, behavior) = &*editor;
            let name_ref = self.weapon_name.borrow();
            let weapon_display = Some((
                name_ref.as_deref().unwrap_or("Default Loadout"),
                stats,
                *behavior,
            ));
            self.pause.draw_ui(ctx, weapon_display);
        }

        if self.pause.is_paused() {
            return;
        }

        // ── Interaction prompt ────────────────────────────────────────────────
        if let InteractionState::None = *self.interaction.borrow() {
            if self.world.nearest_interactable_npc() == Some(NpcKind::Forgemaster) {
                egui::Area::new(egui::Id::new("interact_prompt"))
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -60.0))
                    .show(ctx, |ui| {
                        ui.label(
                            egui::RichText::new("[E]  Forgemaster")
                                .color(egui::Color32::from_rgb(230, 160, 32))
                                .strong(),
                        );
                    });
            }
        }

        // ── Forge dialog ──────────────────────────────────────────────────────
        let dialog_open = matches!(*self.interaction.borrow(), InteractionState::ForgeDialog(_));
        if dialog_open {
            self.draw_forge_dialog(ctx);
        }

        // ── Forge result panel ────────────────────────────────────────────────
        if self.last_forged.borrow().is_some() {
            self.draw_forge_result(ctx);
        }

        // ── Rarity test results popup ─────────────────────────────────────────
        if self.rarity_test_results.borrow().is_some() {
            self.draw_rarity_test_results(ctx);
        }

        // ── Sandbox panel (tabbed) ────────────────────────────────────────────
        if !dialog_open {
            self.draw_sandbox_panel(ctx);
        }
    }
}

// ---------------------------------------------------------------------------
// draw_ui helpers
// ---------------------------------------------------------------------------

const SHAPES: [(&str, ScrapShape); 4] = [
    ("◆", ScrapShape::Diamond),
    ("●", ScrapShape::Circle),
    ("☽", ScrapShape::Crescent),
    ("▲", ScrapShape::Triangle),
];

const COLORS: [(ScrapColor, &str, egui::Color32); 8] = [
    (ScrapColor::Red, "Red", egui::Color32::from_rgb(230, 38, 38)),
    (
        ScrapColor::Orange,
        "Orange",
        egui::Color32::from_rgb(242, 127, 25),
    ),
    (
        ScrapColor::Yellow,
        "Yellow",
        egui::Color32::from_rgb(220, 210, 25),
    ),
    (
        ScrapColor::Green,
        "Green",
        egui::Color32::from_rgb(25, 200, 51),
    ),
    (
        ScrapColor::Cyan,
        "Cyan",
        egui::Color32::from_rgb(25, 200, 220),
    ),
    (
        ScrapColor::Blue,
        "Blue",
        egui::Color32::from_rgb(38, 76, 240),
    ),
    (
        ScrapColor::Purple,
        "Purple",
        egui::Color32::from_rgb(165, 25, 230),
    ),
    (
        ScrapColor::Pink,
        "Pink",
        egui::Color32::from_rgb(240, 89, 190),
    ),
];

impl SandboxScene {
    fn draw_forge_dialog(&self, ctx: &egui::Context) {
        const GOAL: u32 = 100;

        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(10, 10, 10, 240))
            .inner_margin(egui::Margin::symmetric(24.0, 16.0));

        egui::Window::new("##forge_dialog")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Forgemaster")
                            .heading()
                            .color(egui::Color32::from_rgb(230, 160, 32)),
                    );
                    ui.label(
                        egui::RichText::new("Select scraps to contribute (goal: 100)")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });

                ui.add_space(8.0);

                // We need mut access to contribution while also reading inventory.
                // Borrow separately and carefully.
                let inv = &self.world.inventory;
                let mut contribution = {
                    let state = self.interaction.borrow();
                    if let InteractionState::ForgeDialog(ref c) = *state {
                        c.clone()
                    } else {
                        return;
                    }
                };

                // Grid: header row + one row per color.
                egui::Grid::new("forge_grid")
                    .num_columns(5)
                    .spacing([6.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.label("");
                        for (sym, _) in SHAPES {
                            ui.label(egui::RichText::new(sym).strong());
                        }
                        ui.end_row();

                        for (color, name, egui_color) in COLORS {
                            ui.colored_label(egui_color, name);
                            for (_, shape) in SHAPES {
                                let have = inv.count(color, shape);
                                let selected = contribution.get(color, shape);
                                if have == 0 {
                                    ui.label(
                                        egui::RichText::new("—").color(egui::Color32::DARK_GRAY),
                                    );
                                } else {
                                    let val = contribution.get_mut(color, shape);
                                    let mut v = *val;
                                    ui.add(egui::DragValue::new(&mut v).range(0..=have).speed(0.1));
                                    // Clamp in case inventory shrank (shouldn't happen here).
                                    *contribution.get_mut(color, shape) = v.min(have);
                                    let _ = selected; // silence unused warning
                                }
                            }
                            ui.end_row();
                        }
                    });

                ui.separator();

                let total = contribution.total();
                let color = if total == GOAL {
                    egui::Color32::from_rgb(80, 220, 80)
                } else {
                    egui::Color32::GRAY
                };
                ui.horizontal(|ui| {
                    ui.label("Total selected:");
                    ui.colored_label(color, format!("{} / {}", total, GOAL));
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let forge_btn =
                        egui::Button::new(egui::RichText::new("Forge").color(egui::Color32::BLACK))
                            .fill(if total == GOAL {
                                egui::Color32::from_rgb(230, 160, 32)
                            } else {
                                egui::Color32::from_gray(60)
                            });

                    if ui.add_enabled(total == GOAL, forge_btn).clicked() {
                        // Write back contribution first, then signal tick() to execute.
                        if let InteractionState::ForgeDialog(ref mut c) =
                            *self.interaction.borrow_mut()
                        {
                            *c = contribution.clone();
                        }
                        self.forge_requested.set(true);
                        return;
                    }

                    if ui.button("Cancel").clicked() {
                        *self.interaction.borrow_mut() = InteractionState::None;
                        return;
                    }
                });

                // Write back contribution edits from the grid.
                if let InteractionState::ForgeDialog(ref mut c) = *self.interaction.borrow_mut() {
                    *c = contribution;
                }
            });
    }

    fn draw_forge_result(&self, ctx: &egui::Context) {
        let (r, g, b, shape_pts) = {
            let part_ref = self.last_forged.borrow();
            let Some(part) = part_ref.as_ref() else {
                return;
            };
            let [r, g, b] = part.avg_color;
            (r, g, b, part.shape.clone())
        };
        let fill = egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);

        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(10, 10, 10, 220))
            .inner_margin(egui::Margin::symmetric(20.0, 14.0));

        egui::Window::new("##forge_result")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Mod Part Forged!")
                            .heading()
                            .color(egui::Color32::from_rgb(230, 160, 32)),
                    );
                    ui.add_space(8.0);
                    // Polygon preview — render the actual mod part shape.
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(100.0, 100.0), egui::Sense::hover());
                    let center = rect.center();
                    // BASE_RADIUS in mod_part is 0.5; scale to fill ~80% of the preview box.
                    let scale = rect.width() * 0.80;
                    let points: Vec<egui::Pos2> = shape_pts
                        .iter()
                        .map(|v| egui::pos2(center.x + v.x * scale, center.y - v.y * scale))
                        .collect();
                    ui.painter().add(egui::Shape::Path(egui::epaint::PathShape {
                        points,
                        closed: true,
                        fill,
                        stroke: egui::Stroke::new(2.0, egui::Color32::BLACK).into(),
                    }));
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "#{:02X}{:02X}{:02X}",
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8
                        ))
                        .color(fill),
                    );
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        *self.last_forged.borrow_mut() = None;
                    }
                });
            });
    }

    fn draw_rarity_test_results(&self, ctx: &egui::Context) {
        let counts = {
            let r = self.rarity_test_results.borrow();
            let Some(c) = *r else { return };
            c
        };

        const TOTAL: u32 = 1000;
        const TIERS: [(&str, egui::Color32); 5] = [
            ("Common",   egui::Color32::from_rgb(130, 130, 130)),
            ("Uncommon", egui::Color32::from_rgb(60, 115, 255)),
            ("Rare",     egui::Color32::from_rgb(160, 50, 235)),
            ("Epic",     egui::Color32::from_rgb(255, 175, 0)),
            ("Mythic",   egui::Color32::from_rgb(220, 50, 50)),
        ];

        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(10, 10, 10, 240))
            .inner_margin(egui::Margin::symmetric(24.0, 16.0));

        egui::Window::new("##rarity_test_results")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Rarity Distribution (n=1000)")
                            .heading()
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add_space(10.0);

                egui::Grid::new("rarity_results_grid")
                    .num_columns(3)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        for (i, (name, color)) in TIERS.iter().enumerate() {
                            let n = counts[i];
                            let pct = n as f32 / TOTAL as f32 * 100.0;
                            ui.colored_label(*color, *name);
                            ui.colored_label(*color, n.to_string());
                            ui.colored_label(*color, format!("{:.1}%", pct));
                            ui.end_row();
                        }
                    });

                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    if ui.button("Close").clicked() {
                        *self.rarity_test_results.borrow_mut() = None;
                    }
                });
            });
    }

    fn draw_sandbox_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Sandbox")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_min_width(248.0);

                let mut tab = self.selected_tab.get();
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut tab, SandboxTab::PrimaryWeapon, "Primary weapon");
                    ui.selectable_value(&mut tab, SandboxTab::Enemies, "Enemies");
                    ui.selectable_value(&mut tab, SandboxTab::Inventory, "Inventory");
                });
                self.selected_tab.set(tab);
                ui.separator();

                match tab {
                    SandboxTab::PrimaryWeapon => self.draw_primary_weapon_tab(ui),
                    SandboxTab::Enemies => self.draw_enemies_tab(ui),
                    SandboxTab::Inventory => self.draw_inventory_tab(ui),
                }
            });
    }

    fn draw_primary_weapon_tab(&self, ui: &mut egui::Ui) {
        let state_display = self.world.players.first().map(|p| {
            let color = match &p.weapon.state {
                WeaponFiringState::Idle => egui::Color32::from_rgb(100, 220, 100),
                WeaponFiringState::Cooldown(_) => egui::Color32::from_rgb(220, 180, 60),
                WeaponFiringState::Burst { .. } => egui::Color32::from_rgb(220, 120, 60),
                WeaponFiringState::Reloading(_) => egui::Color32::from_rgb(160, 160, 220),
            };
            (
                p.weapon.state.label(),
                color,
                p.weapon.state.remaining_secs(),
            )
        });

        if let Some((label, color, rem)) = state_display {
            ui.horizontal(|ui| {
                ui.label("State:");
                ui.colored_label(color, label);
                if rem > 0.0 {
                    ui.label(format!("({:.2}s)", rem));
                }
            });
        }

        ui.separator();

        let slow = self.slow_motion.get();
        let btn_text = if slow {
            "⏸  Normal speed"
        } else {
            "🐢  Slow motion  (0.2×)"
        };
        let btn = egui::Button::new(btn_text);
        let btn = if slow {
            btn.fill(egui::Color32::from_rgb(40, 120, 60))
        } else {
            btn
        };
        if ui.add_sized([ui.available_width(), 24.0], btn).clicked() {
            self.slow_motion.set(!slow);
        }

        ui.separator();

        let name_color = {
            let editor = self.weapon_editor.borrow();
            let rarities = crate::loot::WeaponStatRarities::from_stats(&editor.0);
            crate::pause::rarity_color(rarities.overall_score())
        };

        ui.horizontal(|ui| {
            let name = self.weapon_name.borrow();
            let label_text = name.as_deref().unwrap_or("Default Loadout");
            ui.label(
                egui::RichText::new(label_text)
                    .italics()
                    .color(name_color),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Roll Random").clicked() {
                    self.random_weapon_requested.set(true);
                }
                if ui.button("Rarity Test").clicked() {
                    self.rarity_test_requested.set(true);
                }
            });
        });

        ui.separator();

        let live_kickback_deg = self
            .world
            .players
            .first()
            .map(|p| p.weapon.kickback.to_degrees());

        let mut editor = self.weapon_editor.borrow_mut();
        let (stats, behavior) = &mut *editor;

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                egui::Grid::new("weapon_stats_editor")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        primary_weapon_grid_section(ui, "Firing & burst");
                        ui.label("fire_rate");
                        ui.add(
                            egui::DragValue::new(&mut stats.fire_rate)
                                .range(0.5..=60.0)
                                .suffix(" rps")
                                .speed(0.1),
                        );
                        ui.end_row();

                        ui.label("burst_count");
                        ui.add(egui::Slider::new(&mut stats.burst_count, 1u32..=8));
                        ui.end_row();

                        ui.label("burst_delay");
                        let mut delay_ms = stats.burst_delay * 1000.0;
                        if ui
                            .add(
                                egui::DragValue::new(&mut delay_ms)
                                    .range(1.0_f32..=2000.0)
                                    .suffix(" ms")
                                    .speed(1.0),
                            )
                            .changed()
                        {
                            stats.burst_delay = delay_ms / 1000.0;
                        }
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Shot pattern");
                        ui.label("projectiles_per_shot");
                        ui.add(egui::Slider::new(
                            &mut stats.projectiles_per_shot,
                            1u32..=16,
                        ));
                        ui.end_row();

                        ui.label("shot_arc");
                        let mut arc_deg = stats.shot_arc.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut arc_deg, 0.0_f32..=360.0).suffix("°"))
                            .changed()
                        {
                            stats.shot_arc = arc_deg.to_radians();
                        }
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Spread & kickback");
                        ui.label("jitter");
                        let mut jitter_deg = stats.jitter.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut jitter_deg, 0.0_f32..=45.0).suffix("°"))
                            .changed()
                        {
                            stats.jitter = jitter_deg.to_radians();
                        }
                        ui.end_row();

                        ui.label("kickback");
                        let mut iv_deg = stats.kickback.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut iv_deg, 0.0_f32..=15.0).suffix("°"))
                            .changed()
                        {
                            stats.kickback = iv_deg.to_radians();
                        }
                        ui.end_row();

                        ui.label("sway (τ)");
                        ui.add(
                            egui::DragValue::new(&mut stats.sway)
                                .range(0.02..=5.0)
                                .suffix(" s")
                                .speed(0.02),
                        );
                        ui.end_row();

                        ui.label("kickback (live)");
                        if let Some(deg) = live_kickback_deg {
                            ui.label(format!("{deg:.2}°"));
                        } else {
                            ui.label("—");
                        }
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Projectile");
                        ui.label("projectile_behavior");
                        egui::ComboBox::from_id_salt("sandbox_proj_behavior")
                            .selected_text(behavior.label())
                            .show_ui(ui, |ui| {
                                for b in ProjectileBehavior::ALL {
                                    ui.selectable_value(behavior, b, b.label());
                                }
                            });
                        ui.end_row();

                        ui.label("projectile_speed");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_speed)
                                .range(1.0..=100.0)
                                .suffix(" u/s")
                                .speed(0.5),
                        );
                        ui.end_row();

                        ui.label("projectile_size");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_size)
                                .range(0.02..=0.5)
                                .speed(0.005),
                        );
                        ui.end_row();

                        ui.label("projectile_lifetime");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_lifetime)
                                .range(0.1..=10.0)
                                .suffix(" s")
                                .speed(0.05),
                        );
                        ui.end_row();

                        ui.label("piercing");
                        ui.add(egui::Slider::new(&mut stats.piercing, 0u32..=10));
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Oscillating (path)");
                        ui.label("oscillation_frequency");
                        ui.add(
                            egui::DragValue::new(&mut stats.oscillation_frequency)
                                .range(0.1..=30.0)
                                .suffix(" Hz")
                                .speed(0.05),
                        );
                        ui.end_row();

                        ui.label("oscillation_magnitude");
                        ui.add(
                            egui::DragValue::new(&mut stats.oscillation_magnitude)
                                .range(0.0..=2.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Physics projectile");
                        ui.label("physics_max_bounces (0 = ∞)");
                        ui.add(egui::Slider::new(&mut stats.physics_max_bounces, 0u32..=32));
                        ui.end_row();

                        ui.label("physics_friction");
                        ui.add(
                            egui::DragValue::new(&mut stats.physics_friction)
                                .range(0.0..=20.0)
                                .speed(0.05),
                        );
                        ui.end_row();

                        ui.label("physics_min_speed");
                        ui.add(
                            egui::DragValue::new(&mut stats.physics_min_speed)
                                .range(0.0..=30.0)
                                .suffix(" u/s")
                                .speed(0.05),
                        );
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Rocket");
                        ui.label("rocket_acceleration");
                        ui.add(
                            egui::DragValue::new(&mut stats.rocket_acceleration)
                                .range(0.0..=200.0)
                                .suffix(" u/s²")
                                .speed(0.5),
                        );
                        ui.end_row();

                        ui.label("kinetic_damage_scale");
                        ui.add(
                            egui::DragValue::new(&mut stats.kinetic_damage_scale)
                                .range(0.0..=10.0)
                                .speed(0.02),
                        );
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Seeking");
                        ui.label("seeking_turn_radius");
                        ui.add(
                            egui::DragValue::new(&mut stats.seeking_turn_radius)
                                .range(0.1..=20.0)
                                .suffix(" u")
                                .speed(0.05),
                        );
                        ui.end_row();

                        ui.label("seeking_acquire_half_angle");
                        let mut half_deg = stats.seeking_acquire_half_angle.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut half_deg, 5.0_f32..=175.0).suffix("°"))
                            .changed()
                        {
                            stats.seeking_acquire_half_angle = half_deg.to_radians();
                        }
                        ui.end_row();

                        primary_weapon_grid_section(ui, "Impact");
                        ui.label("damage_total");
                        ui.add(
                            egui::DragValue::new(&mut stats.damage_total)
                                .range(0.0..=1000.0)
                                .speed(1.0),
                        );
                        ui.end_row();

                        ui.label("recoil_force");
                        ui.add(
                            egui::DragValue::new(&mut stats.recoil_force)
                                .range(0.0..=20.0)
                                .speed(0.1),
                        );
                        ui.end_row();
                    });
            });

        ui.separator();
        ui.label(
            egui::RichText::new("Smaller sway τ → runtime kickback decays faster (always, including while firing).")
                .small()
                .color(egui::Color32::GRAY),
        );
        ui.label(
            egui::RichText::new("LMB / Space to fire")
                .small()
                .color(egui::Color32::GRAY),
        );
    }

    fn draw_enemies_tab(&self, ui: &mut egui::Ui) {
        let alive = self.world.alive_enemy_count();
        let player_dead = self.world.player_health(0).map_or(false, |h| h <= 0.0);

        ui.label(format!("{alive} enemy alive"));
        ui.separator();

        if ui
            .add_sized(
                [ui.available_width(), 24.0],
                egui::Button::new("Spawn Dummy"),
            )
            .clicked()
        {
            self.enemy_spawn_requests
                .set(self.enemy_spawn_requests.get() + 1);
        }

        let respawn_btn = egui::Button::new("Respawn P1");
        let respawn_btn = if player_dead {
            respawn_btn.fill(egui::Color32::from_rgb(180, 60, 60))
        } else {
            respawn_btn
        };
        if ui
            .add_sized([ui.available_width(), 24.0], respawn_btn)
            .clicked()
        {
            self.player_respawn_requested.set(true);
        }
    }

    fn draw_inventory_tab(&self, ui: &mut egui::Ui) {
        let inv = &self.world.inventory;

        // ── Inventory grid ────────────────────────────────────────────────────
        egui::Grid::new("inv_grid")
            .num_columns(5)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("");
                for (sym, _) in SHAPES {
                    ui.label(egui::RichText::new(sym).strong());
                }
                ui.end_row();

                for (color, name, egui_color) in COLORS {
                    ui.colored_label(egui_color, name);
                    for (_, shape) in SHAPES {
                        let n = inv.count(color, shape);
                        let text = egui::RichText::new(n.to_string()).color(if n > 0 {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::DARK_GRAY
                        });
                        ui.label(text);
                    }
                    ui.end_row();
                }
            });

        ui.separator();
        ui.label(format!("Total: {}", inv.total()));

        ui.separator();

        // ── Spawn scrap controls ──────────────────────────────────────────────
        ui.label(egui::RichText::new("Spawn Scrap").strong());

        // Color picker row.
        ui.horizontal_wrapped(|ui| {
            for (color, name, egui_color) in COLORS {
                let selected = self.spawn_color.get() == color;
                let btn = egui::Button::new(egui::RichText::new(name).color(egui::Color32::BLACK))
                    .fill(egui_color)
                    .stroke(if selected {
                        egui::Stroke::new(2.0, egui::Color32::WHITE)
                    } else {
                        egui::Stroke::NONE
                    });
                if ui.add(btn).clicked() {
                    self.spawn_color.set(color);
                }
            }
        });

        // Shape picker row.
        ui.horizontal(|ui| {
            for (sym, shape) in SHAPES {
                let selected = self.spawn_shape.get() == shape;
                let btn = egui::Button::new(egui::RichText::new(sym).heading());
                let btn = if selected {
                    btn.stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                } else {
                    btn
                };
                if ui.add(btn).clicked() {
                    self.spawn_shape.set(shape);
                }
            }
        });

        if ui
            .add_sized(
                [ui.available_width(), 24.0],
                egui::Button::new("Spawn Scrap"),
            )
            .clicked()
        {
            self.scrap_spawn_request.set(true);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_escape(e: &InputEvent) -> bool {
    matches!(
        e,
        InputEvent::Button {
            button: Button::Key(KeyCode::Escape),
            pressed: true,
            ..
        }
    )
}
