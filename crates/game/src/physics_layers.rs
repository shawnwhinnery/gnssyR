//! Bitmasks for [`physics::Body::collision_layers`] / [`physics::Body::collision_mask`].
//!
//! Player-owned and enemy-owned projectiles use separate layers so volleys do not
//! interact with each other, and shots do not physically collide with their owner type.

/// Static level / obstacle geometry.
pub const LAYER_WALL: u32 = 1 << 0;
/// Couch co-op player capsule.
pub const LAYER_PLAYER: u32 = 1 << 1;
/// Hostile actors.
pub const LAYER_ENEMY: u32 = 1 << 2;
/// Bullets fired by a player weapon.
pub const LAYER_PROJ_PLAYER: u32 = 1 << 3;
/// Bullets fired by an enemy weapon.
pub const LAYER_PROJ_ENEMY: u32 = 1 << 4;
/// Friendly NPCs (static obstacles).
pub const LAYER_NPC: u32 = 1 << 5;

#[inline]
pub fn wall_collision() -> (u32, u32) {
    (
        LAYER_WALL,
        LAYER_PLAYER | LAYER_ENEMY | LAYER_NPC | LAYER_PROJ_PLAYER | LAYER_PROJ_ENEMY,
    )
}

#[inline]
pub fn player_collision() -> (u32, u32) {
    (
        LAYER_PLAYER,
        LAYER_WALL | LAYER_PLAYER | LAYER_ENEMY | LAYER_NPC | LAYER_PROJ_ENEMY,
    )
}

#[inline]
pub fn enemy_collision() -> (u32, u32) {
    (
        LAYER_ENEMY,
        LAYER_WALL | LAYER_PLAYER | LAYER_ENEMY | LAYER_NPC | LAYER_PROJ_PLAYER,
    )
}

#[inline]
pub fn npc_collision() -> (u32, u32) {
    (LAYER_NPC, LAYER_WALL | LAYER_PLAYER | LAYER_ENEMY)
}

#[inline]
pub fn projectile_player_owned() -> (u32, u32) {
    (LAYER_PROJ_PLAYER, LAYER_WALL | LAYER_ENEMY)
}

#[inline]
pub fn projectile_enemy_owned() -> (u32, u32) {
    (LAYER_PROJ_ENEMY, LAYER_WALL | LAYER_PLAYER)
}
