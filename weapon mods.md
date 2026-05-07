Lets plan weapon mods. Weapon mods will have a very wide variety of effects. 
They will occupy a number of slots and can be combined to create a unique weapon.
Mods can haeve multiple effects


- split into a fan shot pattern (depending on rarity) projectiles
- reduces kickback buildup / reduced sway
- faster kickback decay while not firing (tune `stability` τ)
- reduces recoil
- damage increase 
- every {N} shots player shoots bonus projectiles
- split on hit {N} projectiles
- split on hit recursively {N} times
- bouncing projectiles
- projectiles create explosion on hit / ricochet
- knockback increased
- projectiles grow as they age
- instant death on hit randomly (chance increases with rarity)


Mods will also have procedural names based on their rarity and the effects they have. 
We'll need to come up with an algorithm to generate names using a list of
words and a list of adjectives.


## 1. Elemental & Natural Forces
These provide sensory feedback and are staple choices for RPG weapons, spells, or world regions.

### **Electric & Kinetic**
* **Voltaic** (High-tech/Scientific)
* **Galvanic** (Industrial/Steampunk)
* **Static** (Low-level/Passive)
* **Fulgurant** (Rare/Poetic for lightning)
* **Kinetic** (Movement-based)
* **Conductive** (Energy-transferring)

### **Corrosive & Toxic**
* **Blighted** (Disease/Nature-rot)
* **Caustic** (Chemical/Acidic)
* **Virulent** (Spreading/Infectious)
* **Miasmic** (Gas/Fog-based)
* **Vitreous** (Glassy/Corrosive)

### **Aetheric & Arcane**
* **Ethereal** (Ghostly/Lightweight)
* **Eldritch** (Lovecraftian/Unknowable)
* **Esoteric** (Secret/Ancient)
* **Resonant** (Vibrational/Sound)
* **Voidborn** (Dark/Empty)

---

## 2. Weight & Materiality
These define the durability or physical presence of an object or character.

| Light & Swift | Heavy & Relentless | Fragile & Sharpened |
| :--- | :--- | :--- |
| **Aerated** | **Adamantine** | **Serrated** |
| **Ephemeral** | **Monolithic** | **Fractured** |
| **Mercurial** | **Obsidian** | **Honed** |
| **Fleet** | **Bastion** | **Brittle** |

---

## 3. Atmospheric & Moral Tone
These set the mood, ranging from "Holy" to "Cursed."

### **Celestial & Divine**
* **Hallowed** (Blessed/Sanctified)
* **Seraphic** (Angelic/High-tier)
* **Gilded** (Wealthy/Surface-level gold)
* **Vigilant** (Protective/Watchful)
* **Luminous** / **Radiant** / **Brilliant**

### **Abyssal & Dread**
* **Malefic** (Actively evil)
* **Squalid** (Filthy/Ruined)
* **Moribund** (Dying/Fading)
* **Desolate** (Lonely/Empty)
* **Infernal** / **Burning** / **Molten**

---

## 4. Temporal & Meta-Physical
Ideal for legendary items or time-manipulation mechanics.

* **Ancient:** Primeval, Relic, Antediluvian.
* **Infinite:** Eternal, Boundless, Perpetual.
* **Unstable:** Volatile, Erratic, Fissured.
* **Hidden:** Veiled, Latent, Cryptic.

---

## 5. Thermal (Cryo)
* **Glacial** (Slow/Massive)
* **Frostbitten** (Damaged/Cold-seared)
* **Frozen** (Static/Solid)
* **Gelid** (Bitterly cold)