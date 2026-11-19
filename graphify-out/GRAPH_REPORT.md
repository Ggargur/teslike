# Graph Report - teslike  (2026-11-19)

## Corpus Check
- 48 files · ~26,870 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 14 file(s) not represented in the graph (top: .ron 8, .glb 3, (none) 1)

## Summary
- 801 nodes · 1756 edges · 56 communities (21 shown, 35 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 80 edges (avg confidence: 0.65)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- feel.rs
- arena.rs
- examples/room.rs
- tests/combat.rs
- common.py
- viewmodel.rs
- src/combat.rs
- ConfigFile
- Action
- psx/src/lib.rs
- run
- CombatPlugin
- Marcos
- move_bodies
- convert_materials
- synth_sfx.py
- PlayerPlugin
- PsxSettings
- sync_standard_changes
- enemy.rs
- layout_output
- Intent
- Checklist de playtest: protótipo 1
- .build
- CLAUDE.md
- game
- 01-fundacao-do-workspace.md
- 02-sala-de-teste-com-visual-psx.md
- 03-andar-pela-sala.md
- 04-ataque-leve-no-boneco.md
- 05-camada-completa-de-game-feel.md
- 06-ataque-carregado.md
- 07-bloqueio-quebra-de-guarda-e-boneco-que-ataca.md
- 08-aparo.md
- 09-esquiva.md
- 10-chute-e-espetos.md
- 11-inimigo-humanoide-que-ataca.md
- 12-inimigo-que-defende-e-reage.md
- 13-loop-de-arena.md
- 14-playtest-e-ajuste-final.md
- build_all.sh

## God Nodes (most connected - your core abstractions)
1. `cfg()` - 38 edges
2. `exchange()` - 32 edges
3. `Combatant` - 30 edges
4. `Player` - 28 edges
5. `CombatConfig` - 28 edges
6. `Fighter` - 27 edges
7. `CombatRules` - 26 edges
8. `switch_opponent()` - 25 edges
9. `Vec3` - 24 edges
10. `countdown()` - 23 edges

## Surprising Connections (you probably didn't know these)
- `reset()` --calls--> `spawn_point()`  [INFERRED]
  crates/game/src/arena.rs → crates/game/src/player.rs
- `Blade` --references--> `Vec3`  [INFERRED]
  crates/game/src/combat.rs → crates/game/src/enemy.rs
- `CombatMsg` --references--> `Vec3`  [INFERRED]
  crates/game/src/combat.rs → crates/game/src/enemy.rs
- `HazardMsg` --references--> `Vec3`  [INFERRED]
  crates/game/src/combat.rs → crates/game/src/enemy.rs
- `resolve()` --references--> `Vec3`  [INFERRED]
  crates/game/src/combat.rs → crates/game/src/enemy.rs

## Import Cycles
- 2-file cycle: `crates/game/src/combat.rs -> crates/game/src/player.rs -> crates/game/src/combat.rs`

## Communities (56 total, 35 thin omitted)

### Community 0 - "feel.rs"
Cohesion: 0.09
Nodes (24): CombatSet, Beat, beat(), charge_sound(), drive_time(), FeelConfig, .FILE, FeelPlugin (+16 more)

### Community 1 - "arena.rs"
Cohesion: 0.09
Nodes (22): Arena, ArenaConfig, .FILE, ArenaPlugin, bar(), countdown(), EnemyBar, first_fight() (+14 more)

### Community 2 - "examples/room.rs"
Cohesion: 0.06
Nodes (16): FALL_Y, HALF, LEDGE_X, RoomPlugin, spawn_room(), texture(), tiled_box(), WALL_H (+8 more)

### Community 3 - "tests/combat.rs"
Cohesion: 0.06
Nodes (71): CombatConfig, Event, Blocked, Died, GuardBroken, Hit, Kicked, Parried (+63 more)

### Community 4 - "common.py"
Cohesion: 0.07
Nodes (11): _blade(), build_sword(), export_glb(), finish(), _finish_sword(), hex_rgb(), Palette, Part (+3 more)

### Community 5 - "viewmodel.rs"
Cohesion: 0.08
Nodes (17): ToRadians, Vec3, Spark, VIEWMODEL_LAYER, animate_boot(), animate_sword(), Boot, Pose (+9 more)

### Community 6 - "src/combat.rs"
Cohesion: 0.07
Nodes (40): Blade, Combatant, CombatMsg, CombatRules, .FILE, HazardMsg, hazards(), hit_filter() (+32 more)

### Community 7 - "ConfigFile"
Cohesion: 0.05
Nodes (17): assets_dir(), ConfigFile, .FILE, ConfigPlugin, ConfigPlugin<T>, mtime(), path(), read() (+9 more)

### Community 8 - "Action"
Cohesion: 0.10
Nodes (5): InputPlugin, InspectorOpen, main(), sky_matches_fog(), toggle_inspector()

### Community 9 - "psx/src/lib.rs"
Cohesion: 0.09
Nodes (9): is_nearest(), nearest_descriptor(), params_differ(), PSX_OUTPUT_LAYER, PSX_OUTPUT_ORDER, PsxExtension, PsxOutputCamera, PsxParams (+1 more)

### Community 10 - "run"
Cohesion: 0.09
Nodes (4): aim(), run(), Script, ScriptPlugin

### Community 12 - "Marcos"
Cohesion: 0.08
Nodes (22): Ação 3D em primeira pessoa, em tempo real, em Bevy, com o game feel em primeiro lugar, Consequências, Opções consideradas, Decisões do protótipo, Estrutura do código, M0: Fundação, M1: O visual PSX, M2: Corpo em primeira pessoa (+14 more)

### Community 13 - "move_bodies"
Cohesion: 0.11
Nodes (9): ground_hit(), Layer, Character, Trigger, World, move_bodies(), MoverPlugin, MoveSet (+1 more)

### Community 14 - "convert_materials"
Cohesion: 0.17
Nodes (12): apply_flash(), convert_materials(), ConvertItem, flash_material(), FlashActive, FlashOriginal, NoPsx, on_remove_flash() (+4 more)

### Community 15 - "synth_sfx.py"
Cohesion: 0.25
Nodes (14): bell(), bp(), env(), fade(), hp(), lp(), mix(), noise() (+6 more)

### Community 17 - "PsxSettings"
Cohesion: 0.20
Nodes (5): base_params(), on_add_camera(), PsxCamera, PsxSettings, sync_settings()

### Community 18 - "sync_standard_changes"
Cohesion: 0.23
Nodes (6): fix_samplers(), material_textures(), MaterialCache, PendingSamplers, PsxTarget, sync_standard_changes()

### Community 20 - "enemy.rs"
Cohesion: 0.08
Nodes (30): Ai, animate(), AttackMove, BodyPose, Brain, Approach, Blocking, Circle (+22 more)

### Community 22 - "Intent"
Cohesion: 0.29
Nodes (7): Intent, AttackPressed, AttackReleased, BlockPressed, BlockReleased, Dodge, Kick

### Community 23 - "Checklist de playtest: protótipo 1"
Cohesion: 0.33
Nodes (5): Checklist de playtest: protótipo 1, Controles, O que observar em cada verbo, O que observar na luta (humanoide), Registro de sessões

### Community 28 - "game"
Cohesion: 0.67
Nodes (3): game, psx, rules

## Knowledge Gaps
- **45 isolated node(s):** `Dummy`, `Humanoid`, `Approach`, `Circle`, `Close` (+40 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 328 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **35 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `CombatConfig` connect `tests/combat.rs` to `src/combat.rs`, `ConfigFile`?**
  _High betweenness centrality (0.068) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `exchange()` (e.g. with `active_light_attack_hits_once_per_swing()` and `being_hit_interrupts_the_defender()`) actually correct?**
  _`exchange()` has 23 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Dummy`, `Humanoid`, `Approach` to the rest of the system?**
  _45 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `feel.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09216255442670537 - nodes in this community are weakly interconnected._
- **Why does `Fighter` connect `tests/combat.rs` to `src/combat.rs`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `CombatConfig` (e.g. with `cfg()` and `charged_to_active()`) actually correct?**
  _`CombatConfig` has 8 INFERRED edges - model-reasoned connections that need verification._
- **Should `arena.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08897959183673469 - nodes in this community are weakly interconnected._