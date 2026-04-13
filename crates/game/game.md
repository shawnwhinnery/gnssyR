# Spec: Game Loop

## Status: Placeholder

This spec will be written once the supporting infrastructure (graphics, input, window)
is implemented and the game mechanics are decided.

---

## Open Questions

- What is the core game concept? (IO-game style — needs more definition)
- Fixed timestep or variable? What target tick rate?
- How is game state structured? (entities, components, or flat structs?)
- Multiplayer: local only, or networked later?
- What constitutes a "round" / win condition?

---

## Known Requirements (from planning.md)

- Up to 4 local players (couch co-op)
- 2D vector graphics, web IO game aesthetic
- Game loop must be testable headlessly (SoftwareDriver + SimulatedBackend)
- Mechanics sandbox must be in place before the game loop is locked in
