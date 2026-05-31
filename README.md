# lau-mission

Mission deployment system — the captain assigns crew to jobs. A mission has objectives, constraints, and crew assignments. When the mission launches, the system tracks progress against objectives and enforces constraints.

## The concept in 60 seconds

Between training and operations lies mission planning. This crate implements:

- **Missions** with objectives, priority, and deadline
- **Crew assignment** matching capabilities to requirements
- **Constraint enforcement** — energy budgets, time limits, conservation laws
- **Progress tracking** — objective completion percentages
- **Mission status:** proposed → active → complete/failed/aborted

## Quick start

```rust
use lau_mission::{Mission, Objective, Crew, MissionStatus};

let mut mission = Mission::new("explore_sector_7")
    .with_priority(0.8)
    .with_constraint("energy_budget", 100.0);

mission.add_objective(Objective::new("map_sector").with_weight(0.5));
mission.add_objective(Objective::new("collect_samples").with_weight(0.3));
mission.add_objective(Objective::new("report_findings").with_weight(0.2));

// Assign crew
mission.assign_crew(Crew::new("ensign_data").with_capability("science"));
mission.assign_crew(Crew::new("ensign_worf").with_capability("security"));

// Launch
mission.launch();
assert_eq!(mission.status(), MissionStatus::Active);

// Report progress
mission.report_progress("map_sector", 0.7);
let completion = mission.overall_progress();
```

## Contributing

[Open an issue](https://github.com/SuperInstance/lau-mission/issues) or PR.
