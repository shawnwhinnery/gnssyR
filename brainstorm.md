# Game Brainstorm

## Core Concept

Top-down vector graphics couch co-op (up to 4 players).  
Genre blend: bullet hell + roguelike + *Custom Robo*.

---

## Replayability Pillar

The goal is infinite RNG-driven collecting that keeps players coming back.  
Key insight from successful games: give players **something to hunt** that is both random and persistent.

### Case Studies

#### Elden Ring: Nightreign - Randomized Gems

- Players have 3 gem slots, each with a color type.
- Gems (matching the slot color) have 1-3 randomly populated sub-slots.
- A small number of randomly generated gems are awarded at the end of each run.
- The combination of variety + randomness keeps players grinding runs.

#### Pokemon GO - Individual Values (IVs)

- Each Pokemon has hidden random stats that cap its genetic potential.
- Players chase "perfect" (100% IV) specimens of each species.
- This drives catching hundreds of the same Pokemon repeatedly.

#### Sonic Adventure - Chao Garden

- Chao absorb attributes from small animals collected in levels.
- Players replay levels specifically to collect targeted animals.
- The garden acts as a persistent meta-game layer outside the main loop.

### Common Pattern

> Persistent collection layer + randomized drops with meaningful variance + a clear "best possible" state to hunt.

---

## Progression Systems
Mobs drop scraps of differnt colors (elements)
At the end of each rounds scraps are collected
When enough scraps have been collected, a modifier part is produced
A modifier's effects are random but influinced by the scraps used to create it
Players play multiple runs looking for elite weapons and elite mobs

### Weapon and Combat Modifiers

Brainstorming stats and modifiers:

| Modifier              | Notes                                  |
| --------------------- | -------------------------------------- |
| Bullet travel speed   |                                        |
| Projectiles per round |                                        |
| Bullet spread         |                                        |
| Bullet size           |                                        |
| Mag size              |                                        |
| Reload speed          |                                        |
| Full auto             | Boolean toggle                         |
| On-hit effects        | e.g. burn, slow, shock                 |
| Bullet piercing       |                                        |
| Homing bullets        |                                        |
| Hitscan / laser       | Boolean toggle                         |
| Bouncing              | Bounces in a random direction on hit   |
| Splitting             | Splits into smaller projectiles on hit |
| Exploding             | AoE damage                             |
| Burning               | DoT                                    |
| Flat stats            | HP, armor, damage, etc.                |
| Recoil                |                                        |
| Crit chance           |                                        |
| Crit multiplier       |                                        |

---

## Combat

### Elements

| Element   | Flavor / Notes                                     |
|-----------|----------------------------------------------------|
| Fire      | DoT burn, melts armor                              |
| Holy      | Bonus vs undead/dark enemies; heals allies on crit |
| Lightning | Chain-hits nearby enemies; interrupts shields      |
| Frost/Ice | Slows, then shatters for burst damage              |
| Poison    | Long DoT, stacks; synergizes with DoT modifiers    |
| Shadow    | Reduces enemy damage output; lifesteal component   |
| Arcane    | Wild-card; amplifies other element interactions    |
| Physical  | Neutral; benefits from armor-shred modifiers       |

### Elemental Damage Re-Allocation (Weapon Affinity)

- Each weapon has a **base damage pool** (e.g. 80% Physical, 20% Fire).
- Applying an **Affinity** re-slices that pool (e.g. Holy Affinity → 40% Physical, 60% Holy).
- Trade-off: shifting into an element lowers the dominant physical number, so affinity is situational, not strictly better.
- Affinity is a per-weapon setting, swappable at a hub or between runs.

### Elemental Defenses

- Enemies have a **resistance profile** — each element has a value: Weak / Neutral / Resistant / Immune.
- Players also have a defense profile, upgraded via items or weapon synergies.
- Resistances create a reason to own multiple weapon types for team coverage.

---

## Off-Hand Items

Off-hand items enable class-like identity and state-based playstyles. Synergies between roles encourage team-oriented builds.

| Item    | Role      | Description                                                                 |
|---------|-----------|-----------------------------------------------------------------------------|
| Sword   | Offense   | Fast melee sweep — close-range burst (Mega Man X)                           |
| Wand    | Offense   | Ranged magic projectile — high damage, low mobility (WoW Mage)              |
| Sickle  | Offense   | Necromancer-flavored; life drain or summon synergy                          |
| Shield  | Tank      | Directional block; protects nearby allies (Braum, League of Legends)        |
| Bunker  | Tank      | Deployable one-way barrier — blocks incoming fire, allows shooting out; has HP |
| Totems  | Support   | Planted zone buffs / debuffs for the team (WoW Shaman)                     |
| Drones  | Engineer  | Autonomous unit — attack, scout, or repair                                  |
| Repairs | Engineer  | Restores bunker HP or ally armor; area tool

---

### Weapon Collection & Development (Chao Garden × Pokemon GO)

**Collection loop**
- Primary weapons are collectible loot drops, not purchased.
- Players maintain an inventory of elite weapons; junk weapons are recycled into **parts**.

**Leveling**
- Parts and end-of-run rewards are spent to level up a chosen weapon.
- The *type* of part spent influences how the weapon develops:
  - Red part → increases Fire stat
  - Yellow part → increases Holy stat
  - (one color per element)
- This makes farming specific enemy types or runs intentional.

**Weapon DNA (randomized at drop)**

| Property               | How determined                                       |
|------------------------|------------------------------------------------------|
| Elemental scaling      | Random at drop — sets the weapon's *ceiling*         |
| Damage type allocation | Random at drop (% split across elements + physical)  |
| IVs (per-element)      | Random hidden value per element at drop              |

**IV quality & modifier slots**
- Each element stat has its own hidden IV assigned at drop.
- **IV quality** = average of all per-element IVs — this determines the weapon's **max modifier slot count**.
- Players hunting a "perfect" weapon are chasing high IV quality and a favorable element distribution.

**Progression unlocks (earned by leveling)**
- Base damage grows with level (not random).
- Modifier slots unlock as the weapon levels up — current available slots increase toward the IV-quality-gated maximum.
- Scaling multipliers activate at level milestones and are multiplicative on top of base damage.

**Base damage vs. scaling**
- **Base damage** = flat value from level + stat allocation.
- **Scaling** = multiplicative bonus that only activates and grows as the weapon levels, determined by the random elemental scaling stat at drop.
- A low-scaling weapon can be strong early (good base) but caps out; a high-scaling weapon rewards long-term investment.

---

## Home Base

A persistent hub players return to between runs — the Chao Garden equivalent for this game. All meta-progression happens here.

### Stations

| Station          | Purpose                                                                 |
|------------------|-------------------------------------------------------------------------|
| Armory           | Swap primary weapon and off-hand item; view loadout stats               |
| Forge            | Spend parts to level up a weapon; apply affinity                        |
| Weapon Stash     | Browse and manage the full weapon inventory; recycle junk for parts     |
| Modifier Bench   | Install, swap, or remove modifiers in unlocked weapon slots             |

### Design Goals

- The base should feel lived-in and legible — players can see at a glance what needs attention (weapons ready to level, slots waiting for modifiers).
- Each station is a discrete interaction, not a menu stack, keeping the hub feel spatial rather than UI-driven.
- Re-entering the base between runs should feel like a reward loop, not a chore — players always have something to do here after a run.

---

## Open TODOs

### Progression
- Players progress through a level where they fight a unique or high ranking enemy at the end of the level.
- Must feel meaningful at all times.
- Must be possible to become proficient.
