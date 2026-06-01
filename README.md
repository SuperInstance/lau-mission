# lau-mission

> Missions are the atomic unit of purposeful work in the PLATO ecosystem. This crate defines what a mission is, how it progresses, and how agents organize around one.

**lau-mission** provides a complete mission lifecycle library — from planning and briefing through execution, completion, failure, and post-mortem analysis. It is the task-planning backbone for multi-agent coordination.

---

## What This Does

This crate models the full lifecycle of a mission:

```
Planning → Assigned → InProgress → Completed
                                  → Failed
                                  → Aborted
```

It provides:

| Component | Purpose |
|---|---|
| **`Mission`** | A deployable unit of work with objectives, assigned agents, a team leader, tick-based timing, and conservation budget. |
| **`MissionObjective`** | A weighted, completable goal within a mission. |
| **`MissionResult`** | The post-mortem: weighted success score, conservation error, per-agent performance, lessons learned. |
| **`MissionBriefing`** | Auto-generated planning document: required skills, recommended team size, risk assessment, estimated duration. |
| **`MissionLog`** | The central registry — create, start, update, complete, fail missions; query by status, agent, or aggregate statistics. |
| **Templates** | Pre-built missions (Bridge Builder, Farm Setup, Scout Report, Conservation Audit, Rescue Operation) for rapid prototyping. |

---

## Key Idea

Missions are **weighted-objective state machines**. Every objective has a weight (0.0–1.0), and the success score is computed as the sum of weights of completed objectives divided by the total weight. This means a mission with one critical objective (weight 0.8) and one minor one (weight 0.2) can score 0.8 even if only the critical objective is done.

The `MissionBriefing` auto-generates planning intelligence from a mission's type and difficulty:

- **Required skills** are derived from mission type (Construction → building + engineering, Rescue → medicine + search-and-rescue, etc.)
- **Recommended team size** scales with difficulty: `ceil(difficulty × 5) + 1`
- **Risk assessment** = `difficulty × 0.8` + `0.2` if no agents are assigned yet
- **Estimated duration** = `objectives × 10 × (1 + difficulty)` ticks

---

## Install

```toml
[dependencies]
lau-mission = "0.1"
```

Or:

```bash
cargo add lau-mission
```

### Dependencies

- `serde` 1.x (with `derive`) — serialisation
- `serde_json` 1.x (dev-only, for round-trip tests)

No async runtime, no database, no filesystem.

---

## Quick Start

```rust
use lau_mission::*;

// 1. Create a mission from a template
let mut mission = bridge_builder_mission();

// 2. Assign agents and set a leader
mission.assign_agent("alice".into());
mission.assign_agent("bob".into());
mission.set_leader("alice".into());

// 3. Generate a briefing for planning
let briefing = MissionBriefing::from_mission(&mission);
println!("Skills needed: {:?}", briefing.required_skills);
println!("Risk: {:.0}%", briefing.risk_assessment * 100.0);

// 4. Register in the log and start
let mut log = MissionLog::new();
let id = log.create(mission);
log.start_mission(&id, 100).unwrap();

// 5. Complete objectives
log.update_objective(&id, 0);  // Survey riverbank
log.update_objective(&id, 1);  // Build foundation
log.update_objective(&id, 2);  // Complete decking

// 6. Generate result and complete
let result = MissionResult::from_mission(log.get(&id).unwrap(), 0.03);
log.complete_mission(&id, 200, result);

// 7. Check statistics
let stats = log.stats();
println!("Success rate: {:.0}%", log.success_rate() * 100.0);
println!("Avg score: {:.2}", stats.avg_success_score);
```

---

## API Reference

### `MissionId`

A newtype wrapper around `String`. Implements `Display`, `From<&str>`, `From<String>`, `Hash`, `Eq`, `Serialize`, `Deserialize`.

### `MissionType`

| Variant | Display | Briefing Skills |
|---|---|---|
| `Construction` | "Construction" | building, engineering |
| `Farming` | "Farming" | agriculture, botany |
| `Scouting` | "Scouting" | reconnaissance, stealth |
| `Trading` | "Trading" | negotiation, logistics |
| `ConservationAudit` | "Conservation Audit" | ecology, data-analysis |
| `Rescue` | "Rescue" | medicine, search-and-rescue |
| `Exploration` | "Exploration" | navigation, surveying |
| `Defense` | "Defense" | combat, tactics |
| `Research` | "Research" | science, analysis |

### `MissionStatus`

`Planning` → `Assigned` → `InProgress` → `Completed` | `Failed` | `Aborted`

### `MissionObjective`

```rust
pub struct MissionObjective {
    pub description: String,
    pub success_criteria: String,
    pub weight: f64,       // clamped to 0.0–1.0
    pub completed: bool,
}
```

- `new(description, success_criteria, weight)` — creates an incomplete objective with clamped weight.

### `MissionResult`

```rust
pub struct MissionResult {
    pub success_score: f64,
    pub conservation_error: f64,
    pub objectives_completed: usize,
    pub objectives_total: usize,
    pub agent_performance: HashMap<String, f64>,
    pub lessons_learned: Vec<String>,
}
```

- `from_mission(mission, conservation_error)` — auto-computes `success_score` as weighted sum of completed objectives, and assigns each agent the same score.

### `Mission`

Key methods:

| Method | Description |
|---|---|
| `new(name, type, difficulty)` | Create in `Planning` status with auto-generated `{slug}-{hex}` ID. |
| `add_objective(obj)` | Append an objective. |
| `assign_agent(id)` | Add agent (deduped). |
| `set_leader(id)` | Set leader (must be assigned already). |
| `start(tick)` | Transition to `InProgress`. Requires ≥1 agent. |
| `complete_objective(idx)` | Mark objective as done. No-op if out of bounds. |
| `complete(tick, result)` | Transition to `Completed` with result. |
| `fail(tick, reason)` | Transition to `Failed`. Reason stored as lesson learned. |
| `abort(tick)` | Transition to `Aborted`. |
| `is_active()` | `true` if `InProgress`. |
| `progress()` | Fraction of objectives completed (0.0–1.0). |
| `duration()` | Ticks from start to completion, or `None`. |
| `team_size()` | Number of assigned agents. |

### `MissionBriefing`

```rust
pub struct MissionBriefing {
    pub mission: Mission,
    pub required_skills: Vec<String>,
    pub recommended_team_size: usize,
    pub risk_assessment: f64,    // 0.0–1.0
    pub estimated_duration: u64, // in ticks
}
```

- `from_mission(mission)` — auto-generates all fields from the mission's type, difficulty, and objectives.

### `MissionLog`

The central registry.

| Method | Description |
|---|---|
| `new()` / `default()` | Empty log. |
| `create(mission)` | Register, return `MissionId`. |
| `start_mission(id, tick)` | Start a mission (validates agents). |
| `update_objective(id, idx)` | Complete an objective. |
| `complete_mission(id, tick, result)` | Complete with result. |
| `fail_mission(id, tick, reason)` | Fail with reason. |
| `get(id)` | Look up by ID. |
| `active_missions()` | All `InProgress`. |
| `completed_missions()` | All `Completed`. |
| `failed_missions()` | All `Failed`. |
| `agent_mission_history(agent_id)` | All missions an agent was on. |
| `success_rate()` | Completed / total. |
| `stats()` | `MissionLogStats` aggregate. |

### `MissionLogStats`

```rust
pub struct MissionLogStats {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub active: usize,
    pub avg_success_score: f64,
    pub avg_conservation_error: f64,
    pub most_common_type: Option<MissionType>,
}
```

### Template Functions

| Function | Type | Objectives | Difficulty |
|---|---|---|---|
| `bridge_builder_mission()` | Construction | 3 | 0.4 |
| `farm_setup_mission()` | Farming | 4 | 0.3 |
| `scout_report_mission()` | Scouting | 2 | 0.5 |
| `conservation_audit_mission()` | ConservationAudit | 3 | 0.6 |
| `rescue_operation_mission()` | Rescue | 4 | 0.8 |
| `template_missions()` | — | — | Returns all 5 |

---

## How It Works

### State Machine

```
                    ┌──────────┐
                    │ Planning │
                    └────┬─────┘
                         │ assign agents
                    ┌────▼─────┐
                    │ Assigned │
                    └────┬─────┘
                         │ start(tick)
                    ┌────▼──────┐
              ┌─────│ InProgress │─────┐
              │     └────────────┘     │
         complete()              fail() / abort()
         ┌───────▼───────┐    ┌───────▼────────┐
         │   Completed   │    │ Failed / Aborted│
         └───────────────┘    └────────────────┘
```

Guards:
- `start()` fails if no agents assigned or already started.
- `set_leader()` no-ops if the agent isn't already assigned.
- `assign_agent()` deduplicates.
- `complete_objective(idx)` no-ops on out-of-bounds index.

### ID Generation

Mission IDs are generated as `{slug}-{8-hex-chars}` where the slug is the mission name lowercased with spaces replaced by hyphens, and the hex chars are derived from the low 32 bits of the current nanosecond timestamp.

---

## The Math

### Weighted Success Score

For a mission with objectives $o_1, o_2, \ldots, o_n$, each with weight $w_i$:

$$\text{success\_score} = \frac{\sum_{i \in \text{completed}} w_i}{\sum_{i=1}^{n} w_i}$$

If total weight is 0 (no objectives), the score defaults to 1.0 (vacuously complete).

### Risk Assessment

$$\text{risk} = \min\!\Big(\text{difficulty} \times 0.8 + \begin{cases} 0.2 & \text{if no agents} \\ 0.0 & \text{otherwise} \end{cases},\ 1.0\Big)$$

### Estimated Duration

$$\text{duration} = \max\!\Big(1,\ \lceil |\text{objectives}| \times 10 \times (1 + \text{difficulty}) \rceil\Big)$$

### Recommended Team Size

$$\text{team\_size} = \lceil \text{difficulty} \times 5 \rceil + 1$$

### Conservation Error

The crate doesn't compute `conservation_error` — it accepts it as an external input in `MissionResult::from_mission()`. This is a domain-specific metric that measures how much the mission deviated from conservation goals, intended to be provided by the PLATO ecosystem's conservation layer.

---

## Testing

**74 tests** covering:

- `MissionId` construction, equality, hashing, display, conversions
- `MissionType` and `MissionStatus` display, clone/copy, hashing
- `MissionObjective` construction and weight clamping
- `Mission` full lifecycle: creation, assignment, leadership, starting, objective completion, completion, failure, abort
- Progress, duration, team size, active status
- `MissionResult::from_mission()` with all/partial/no objectives
- `MissionBriefing` auto-generation: skills, risk scaling, duration
- `MissionLog` CRUD, queries by status/agent, success rate, statistics
- Serde round-trips for all major types
- All 5 template missions
- Full integration test (lifecycle from planning to completion with stats)

```bash
cargo test
```

---

## License

MIT
