use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// MissionId newtype
// ---------------------------------------------------------------------------

/// A unique mission identifier backed by a `String`.
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MissionId(pub String);

impl fmt::Display for MissionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for MissionId {
    fn from(s: &str) -> Self {
        MissionId(s.to_string())
    }
}

impl From<String> for MissionId {
    fn from(s: String) -> Self {
        MissionId(s)
    }
}

// ---------------------------------------------------------------------------
// MissionType
// ---------------------------------------------------------------------------

/// The category of work a mission represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MissionType {
    Construction,
    Farming,
    Scouting,
    Trading,
    ConservationAudit,
    Rescue,
    Exploration,
    Defense,
    Research,
}

impl fmt::Display for MissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MissionType::Construction => write!(f, "Construction"),
            MissionType::Farming => write!(f, "Farming"),
            MissionType::Scouting => write!(f, "Scouting"),
            MissionType::Trading => write!(f, "Trading"),
            MissionType::ConservationAudit => write!(f, "Conservation Audit"),
            MissionType::Rescue => write!(f, "Rescue"),
            MissionType::Exploration => write!(f, "Exploration"),
            MissionType::Defense => write!(f, "Defense"),
            MissionType::Research => write!(f, "Research"),
        }
    }
}

// ---------------------------------------------------------------------------
// MissionStatus
// ---------------------------------------------------------------------------

/// The lifecycle state of a mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MissionStatus {
    Planning,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Aborted,
}

impl fmt::Display for MissionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MissionStatus::Planning => write!(f, "Planning"),
            MissionStatus::Assigned => write!(f, "Assigned"),
            MissionStatus::InProgress => write!(f, "In Progress"),
            MissionStatus::Completed => write!(f, "Completed"),
            MissionStatus::Failed => write!(f, "Failed"),
            MissionStatus::Aborted => write!(f, "Aborted"),
        }
    }
}

// ---------------------------------------------------------------------------
// MissionObjective
// ---------------------------------------------------------------------------

/// An individual objective within a mission.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissionObjective {
    pub description: String,
    pub success_criteria: String,
    /// Importance weight between 0.0 and 1.0.
    pub weight: f64,
    pub completed: bool,
}

impl MissionObjective {
    /// Create a new objective with the given weight, not yet completed.
    pub fn new(description: &str, success_criteria: &str, weight: f64) -> Self {
        MissionObjective {
            description: description.to_string(),
            success_criteria: success_criteria.to_string(),
            weight: weight.clamp(0.0, 1.0),
            completed: false,
        }
    }
}

// ---------------------------------------------------------------------------
// MissionResult
// ---------------------------------------------------------------------------

/// The outcome of a completed mission.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissionResult {
    /// Overall success score between 0.0 and 1.0.
    pub success_score: f64,
    /// Magnitude of conservation error (how much the mission deviated from
    /// conservation goals).
    pub conservation_error: f64,
    pub objectives_completed: usize,
    pub objectives_total: usize,
    /// Per-agent performance scores (agent_id -> score).
    pub agent_performance: HashMap<String, f64>,
    pub lessons_learned: Vec<String>,
}

impl MissionResult {
    /// Build a result from the actual mission state, auto-calculating the
    /// success score based on weighted objectives.
    pub fn from_mission(mission: &Mission, conservation_error: f64) -> Self {
        let objectives_total = mission.objectives.len();
        let objectives_completed = mission.objectives.iter().filter(|o| o.completed).count();

        // Weighted success score based on completed objectives
        let total_weight: f64 = mission.objectives.iter().map(|o| o.weight).sum();
        let earned_weight: f64 = mission
            .objectives
            .iter()
            .filter(|o| o.completed)
            .map(|o| o.weight)
            .sum();
        let success_score = if total_weight > 0.0 {
            earned_weight / total_weight
        } else {
            1.0
        };

        let agent_performance = mission
            .assigned_agents
            .iter()
            .map(|id| (id.clone(), success_score))
            .collect();

        MissionResult {
            success_score,
            conservation_error,
            objectives_completed,
            objectives_total,
            agent_performance,
            lessons_learned: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Mission
// ---------------------------------------------------------------------------

/// A deployable mission that agents can be assigned to.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mission {
    pub id: MissionId,
    pub name: String,
    pub mission_type: MissionType,
    pub objectives: Vec<MissionObjective>,
    pub assigned_agents: Vec<String>,
    pub team_leader: Option<String>,
    pub status: MissionStatus,
    /// Difficulty between 0.0 and 1.0.
    pub difficulty: f64,
    pub tick_created: u64,
    pub tick_started: Option<u64>,
    pub tick_completed: Option<u64>,
    pub conservation_budget: f64,
    pub result: Option<MissionResult>,
}

impl Mission {
    /// Create a new mission in `Planning` status with a generated ID.
    pub fn new(name: &str, mission_type: MissionType, difficulty: f64) -> Self {
        let id = MissionId(format!(
            "{}-{}",
            name.to_lowercase().replace(' ', "-"),
            uuid_v4_short()
        ));
        Mission {
            id,
            name: name.to_string(),
            mission_type,
            objectives: Vec::new(),
            assigned_agents: Vec::new(),
            team_leader: None,
            status: MissionStatus::Planning,
            difficulty: difficulty.clamp(0.0, 1.0),
            tick_created: 0,
            tick_started: None,
            tick_completed: None,
            conservation_budget: 0.0,
            result: None,
        }
    }

    /// Add an objective to this mission.
    pub fn add_objective(&mut self, obj: MissionObjective) {
        self.objectives.push(obj);
    }

    /// Assign an agent to the mission (no-ops for duplicates).
    pub fn assign_agent(&mut self, agent_id: String) {
        if !self.assigned_agents.contains(&agent_id) {
            self.assigned_agents.push(agent_id);
        }
    }

    /// Set the team leader (must already be an assigned agent).
    pub fn set_leader(&mut self, agent_id: String) {
        if self.assigned_agents.contains(&agent_id) {
            self.team_leader = Some(agent_id);
        }
    }

    /// Transition the mission from Planning/Assigned to InProgress.
    /// Errors if no agents are assigned.
    pub fn start(&mut self, tick: u64) -> Result<(), String> {
        if self.assigned_agents.is_empty() {
            return Err("Cannot start a mission with no assigned agents".to_string());
        }
        if self.status == MissionStatus::Planning || self.status == MissionStatus::Assigned {
            self.status = MissionStatus::InProgress;
            self.tick_started = Some(tick);
            Ok(())
        } else {
            Err(format!("Cannot start mission in status {:?}", self.status))
        }
    }

    /// Mark objective `index` as completed. No-op if index is out of bounds.
    pub fn complete_objective(&mut self, index: usize) {
        if let Some(obj) = self.objectives.get_mut(index) {
            obj.completed = true;
        }
    }

    /// Complete the mission with a result.
    pub fn complete(&mut self, tick: u64, result: MissionResult) {
        self.status = MissionStatus::Completed;
        self.tick_completed = Some(tick);
        self.result = Some(result);
    }

    /// Fail the mission with a reason (stored as a fake result).
    pub fn fail(&mut self, tick: u64, reason: String) {
        self.status = MissionStatus::Failed;
        self.tick_completed = Some(tick);
        self.result = Some(MissionResult {
            success_score: 0.0,
            conservation_error: 0.0,
            objectives_completed: self.objectives.iter().filter(|o| o.completed).count(),
            objectives_total: self.objectives.len(),
            agent_performance: HashMap::new(),
            lessons_learned: vec![reason],
        });
    }

    /// Abort the mission.
    pub fn abort(&mut self, tick: u64) {
        self.status = MissionStatus::Aborted;
        self.tick_completed = Some(tick);
    }

    /// Returns `true` if the mission is currently active (InProgress).
    pub fn is_active(&self) -> bool {
        self.status == MissionStatus::InProgress
    }

    /// Fraction of objectives completed (0.0 - 1.0).
    pub fn progress(&self) -> f64 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        let completed = self.objectives.iter().filter(|o| o.completed).count() as f64;
        completed / self.objectives.len() as f64
    }

    /// Duration in ticks from start to completion/failure/abort.
    pub fn duration(&self) -> Option<u64> {
        match (self.tick_started, self.tick_completed) {
            (Some(start), Some(end)) => Some(end.saturating_sub(start)),
            _ => None,
        }
    }

    /// Number of assigned agents.
    pub fn team_size(&self) -> usize {
        self.assigned_agents.len()
    }
}

// ---------------------------------------------------------------------------
// MissionBriefing
// ---------------------------------------------------------------------------

/// A briefing generated from a mission for planning purposes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissionBriefing {
    pub mission: Mission,
    pub required_skills: Vec<String>,
    pub recommended_team_size: usize,
    /// Risk assessment between 0.0 and 1.0.
    pub risk_assessment: f64,
    /// Estimated duration in ticks.
    pub estimated_duration: u64,
}

impl MissionBriefing {
    /// Auto-generate a briefing from a mission.
    pub fn from_mission(mission: &Mission) -> Self {
        let required_skills = briefing_skills_for(mission.mission_type);
        let recommended_team_size = (mission.difficulty * 5.0).ceil() as usize + 1;
        let risk_assessment =
            mission.difficulty * 0.8 + if mission.assigned_agents.is_empty() { 0.2 } else { 0.0 };
        let raw = mission.objectives.len() as f64 * 10.0 * (1.0 + mission.difficulty);
        let estimated_duration = (raw.ceil() as u64).max(1);

        MissionBriefing {
            mission: mission.clone(),
            required_skills,
            recommended_team_size,
            risk_assessment: risk_assessment.clamp(0.0, 1.0),
            estimated_duration,
        }
    }
}

fn briefing_skills_for(mt: MissionType) -> Vec<String> {
    match mt {
        MissionType::Construction => vec!["building".into(), "engineering".into()],
        MissionType::Farming => vec!["agriculture".into(), "botany".into()],
        MissionType::Scouting => vec!["reconnaissance".into(), "stealth".into()],
        MissionType::Trading => vec!["negotiation".into(), "logistics".into()],
        MissionType::ConservationAudit => vec!["ecology".into(), "data-analysis".into()],
        MissionType::Rescue => vec!["medicine".into(), "search-and-rescue".into()],
        MissionType::Exploration => vec!["navigation".into(), "surveying".into()],
        MissionType::Defense => vec!["combat".into(), "tactics".into()],
        MissionType::Research => vec!["science".into(), "analysis".into()],
    }
}

// ---------------------------------------------------------------------------
// MissionLogStats
// ---------------------------------------------------------------------------

/// Aggregate statistics over the mission log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissionLogStats {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub active: usize,
    pub avg_success_score: f64,
    pub avg_conservation_error: f64,
    pub most_common_type: Option<MissionType>,
}

// ---------------------------------------------------------------------------
// MissionLog
// ---------------------------------------------------------------------------

/// The central registry of all missions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissionLog {
    pub missions: HashMap<MissionId, Mission>,
    pub active: Vec<MissionId>,
}

impl MissionLog {
    /// Create an empty mission log.
    pub fn new() -> Self {
        MissionLog {
            missions: HashMap::new(),
            active: Vec::new(),
        }
    }

    /// Register a new mission and return its ID.
    pub fn create(&mut self, mission: Mission) -> MissionId {
        let id = mission.id.clone();
        self.missions.insert(id.clone(), mission);
        id
    }

    /// Start the mission identified by `id`.
    pub fn start_mission(&mut self, id: &MissionId, tick: u64) -> Result<(), String> {
        let mission = self
            .missions
            .get_mut(id)
            .ok_or_else(|| format!("Mission {} not found", id))?;
        mission.start(tick)?;
        self.active.push(id.clone());
        Ok(())
    }

    /// Complete an objective for the given mission.
    pub fn update_objective(&mut self, id: &MissionId, obj_idx: usize) {
        if let Some(mission) = self.missions.get_mut(id) {
            mission.complete_objective(obj_idx);
        }
    }

    /// Complete a mission with a result.
    pub fn complete_mission(&mut self, id: &MissionId, tick: u64, result: MissionResult) {
        if let Some(mission) = self.missions.get_mut(id) {
            mission.complete(tick, result);
        }
        self.active.retain(|a| a != id);
    }

    /// Fail a mission.
    pub fn fail_mission(&mut self, id: &MissionId, tick: u64, reason: String) {
        if let Some(mission) = self.missions.get_mut(id) {
            mission.fail(tick, reason);
        }
        self.active.retain(|a| a != id);
    }

    /// Look up a mission by ID.
    pub fn get(&self, id: &MissionId) -> Option<&Mission> {
        self.missions.get(id)
    }

    /// All currently active (InProgress) missions.
    pub fn active_missions(&self) -> Vec<&Mission> {
        self.missions.values().filter(|m| m.is_active()).collect()
    }

    /// All completed missions.
    pub fn completed_missions(&self) -> Vec<&Mission> {
        self.missions
            .values()
            .filter(|m| m.status == MissionStatus::Completed)
            .collect()
    }

    /// All failed missions.
    pub fn failed_missions(&self) -> Vec<&Mission> {
        self.missions
            .values()
            .filter(|m| m.status == MissionStatus::Failed)
            .collect()
    }

    /// All missions a particular agent was assigned to.
    pub fn agent_mission_history(&self, agent_id: &str) -> Vec<&Mission> {
        self.missions
            .values()
            .filter(|m| m.assigned_agents.contains(&agent_id.to_string()))
            .collect()
    }

    /// Overall success rate (completed / total).
    pub fn success_rate(&self) -> f64 {
        let total = self.missions.len();
        if total == 0 {
            return 0.0;
        }
        let completed = self
            .missions
            .values()
            .filter(|m| matches!(m.status, MissionStatus::Completed))
            .count();
        completed as f64 / total as f64
    }

    /// Aggregate statistics over the log.
    pub fn stats(&self) -> MissionLogStats {
        let total = self.missions.len();
        let completed_count = self.completed_missions().len();
        let failed_count = self.failed_missions().len();
        let active_count = self.active_missions().len();

        let completed_results: Vec<&Mission> = self
            .missions
            .values()
            .filter(|m| matches!(m.status, MissionStatus::Completed))
            .collect();

        let (avg_score, avg_cons) = if completed_results.is_empty() {
            (0.0, 0.0)
        } else {
            let (s, c): (f64, f64) = completed_results.iter().fold((0.0, 0.0), |acc, m| {
                (
                    acc.0 + m.result.as_ref().map_or(0.0, |r| r.success_score),
                    acc.1 + m.result.as_ref().map_or(0.0, |r| r.conservation_error),
                )
            });
            let n = completed_results.len() as f64;
            (s / n, c / n)
        };

        let mut counts: HashMap<MissionType, usize> = HashMap::new();
        for m in self.missions.values() {
            *counts.entry(m.mission_type).or_insert(0) += 1;
        }
        let most_common = counts.into_iter().max_by_key(|&(_, c)| c).map(|(t, _)| t);

        MissionLogStats {
            total,
            completed: completed_count,
            failed: failed_count,
            active: active_count,
            avg_success_score: avg_score,
            avg_conservation_error: avg_cons,
            most_common_type: most_common,
        }
    }
}

impl Default for MissionLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pre-built mission templates
// ---------------------------------------------------------------------------

/// Create a "Bridge Builder" mission: Construction, 3 objectives, difficulty
/// 0.4.
pub fn bridge_builder_mission() -> Mission {
    let mut m = Mission::new("Bridge Builder", MissionType::Construction, 0.4);
    m.add_objective(MissionObjective::new(
        "Survey riverbank",
        "Both banks surveyed and marked",
        0.2,
    ));
    m.add_objective(MissionObjective::new(
        "Build foundation",
        "Foundation laid and inspected",
        0.5,
    ));
    m.add_objective(MissionObjective::new(
        "Complete decking",
        "Deck completed and load-tested",
        0.3,
    ));
    m
}

/// Create a "Farm Setup" mission: Farming, 4 objectives, difficulty 0.3.
pub fn farm_setup_mission() -> Mission {
    let mut m = Mission::new("Farm Setup", MissionType::Farming, 0.3);
    m.add_objective(MissionObjective::new(
        "Clear land",
        "5 hectares cleared of debris",
        0.2,
    ));
    m.add_objective(MissionObjective::new(
        "Install irrigation",
        "Irrigation system operational",
        0.3,
    ));
    m.add_objective(MissionObjective::new(
        "Plant crops",
        "Initial crop batch planted",
        0.3,
    ));
    m.add_objective(MissionObjective::new(
        "Set up storage",
        "Storage silos constructed",
        0.2,
    ));
    m
}

/// Create a "Scout Report" mission: Scouting, 2 objectives, difficulty 0.5.
pub fn scout_report_mission() -> Mission {
    let mut m = Mission::new("Scout Report", MissionType::Scouting, 0.5);
    m.add_objective(MissionObjective::new(
        "Recon area",
        "Perimeter surveyed and mapped",
        0.6,
    ));
    m.add_objective(MissionObjective::new(
        "Report findings",
        "Report delivered with intel",
        0.4,
    ));
    m
}

/// Create a "Conservation Audit" mission: ConservationAudit, 3 objectives,
/// difficulty 0.6.
pub fn conservation_audit_mission() -> Mission {
    let mut m = Mission::new("Conservation Audit", MissionType::ConservationAudit, 0.6);
    m.add_objective(MissionObjective::new(
        "Survey wildlife",
        "Species count completed",
        0.4,
    ));
    m.add_objective(MissionObjective::new(
        "Assess habitat",
        "Habitat health index above 0.7",
        0.4,
    ));
    m.add_objective(MissionObjective::new(
        "Generate report",
        "Audit report filed with recommendations",
        0.2,
    ));
    m
}

/// Create a "Rescue Operation" mission: Rescue, 4 objectives, difficulty 0.8.
pub fn rescue_operation_mission() -> Mission {
    let mut m = Mission::new("Rescue Operation", MissionType::Rescue, 0.8);
    m.add_objective(MissionObjective::new(
        "Locate survivors",
        "All survivors located via thermal scan",
        0.3,
    ));
    m.add_objective(MissionObjective::new(
        "Extract survivors",
        "Survivors extracted to safe zone",
        0.4,
    ));
    m.add_objective(MissionObjective::new(
        "Provide medical aid",
        "All survivors triaged and treated",
        0.2,
    ));
    m.add_objective(MissionObjective::new(
        "Secure area",
        "Area secured for follow-up teams",
        0.1,
    ));
    m
}

/// Return all template missions as a vector.
pub fn template_missions() -> Vec<Mission> {
    vec![
        bridge_builder_mission(),
        farm_setup_mission(),
        scout_report_mission(),
        conservation_audit_mission(),
        rescue_operation_mission(),
    ]
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Generate a short unique ID (8 hex chars).
fn uuid_v4_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:08x}", (nanos & 0xFFFF_FFFF) as u32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- MissionId ----

    #[test]
    fn mission_id_newtype() {
        let id = MissionId("abc-123".into());
        assert_eq!(id.0, "abc-123");
        assert_eq!(id.clone(), id);
        let h: std::collections::HashSet<MissionId> =
            [id.clone(), MissionId("abc-123".into())].into_iter().collect();
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn mission_id_display() {
        let id = MissionId("test-id".into());
        assert_eq!(format!("{id}"), "test-id");
    }

    #[test]
    fn mission_id_from_str() {
        let id: MissionId = "hello".into();
        assert_eq!(id.0, "hello");
    }

    #[test]
    fn mission_id_from_string() {
        let id: MissionId = "world".to_string().into();
        assert_eq!(id.0, "world");
    }

    #[test]
    fn mission_id_equality() {
        let a = MissionId("x".into());
        let b = MissionId("x".into());
        let c = MissionId("y".into());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ---- MissionType ----

    #[test]
    fn mission_type_display() {
        assert_eq!(MissionType::Construction.to_string(), "Construction");
        assert_eq!(MissionType::ConservationAudit.to_string(), "Conservation Audit");
        assert_eq!(MissionType::Defense.to_string(), "Defense");
    }

    #[test]
    fn mission_type_clone_copy() {
        let a = MissionType::Rescue;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn mission_type_hash() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(MissionType::Trading);
        s.insert(MissionType::Trading);
        assert_eq!(s.len(), 1);
    }

    // ---- MissionStatus ----

    #[test]
    fn mission_status_display() {
        assert_eq!(MissionStatus::InProgress.to_string(), "In Progress");
        assert_eq!(MissionStatus::Planning.to_string(), "Planning");
        assert_eq!(MissionStatus::Assigned.to_string(), "Assigned");
        assert_eq!(MissionStatus::Completed.to_string(), "Completed");
        assert_eq!(MissionStatus::Failed.to_string(), "Failed");
        assert_eq!(MissionStatus::Aborted.to_string(), "Aborted");
    }

    // ---- MissionObjective ----

    #[test]
    fn objective_new_defaults() {
        let obj = MissionObjective::new("Do thing", "Thing done", 0.5);
        assert_eq!(obj.description, "Do thing");
        assert_eq!(obj.success_criteria, "Thing done");
        assert!((obj.weight - 0.5).abs() < 1e-9);
        assert!(!obj.completed);
    }

    #[test]
    fn objective_weight_clamped_above() {
        let obj = MissionObjective::new("A", "B", 2.0);
        assert!((obj.weight - 1.0).abs() < 1e-9);
    }

    #[test]
    fn objective_weight_clamped_below() {
        let obj = MissionObjective::new("A", "B", -0.5);
        assert!((obj.weight - 0.0).abs() < 1e-9);
    }

    // ---- Mission ----

    #[test]
    fn mission_new_planning() {
        let m = Mission::new("Test", MissionType::Research, 0.5);
        assert_eq!(m.status, MissionStatus::Planning);
        assert_eq!(m.name, "Test");
        assert!((m.difficulty - 0.5).abs() < 1e-9);
    }

    #[test]
    fn mission_new_id_generated() {
        let m = Mission::new("Alpha Mission", MissionType::Scouting, 0.3);
        assert!(m.id.0.starts_with("alpha-mission-"));
    }

    #[test]
    fn mission_difficulty_clamped() {
        let m = Mission::new("Test", MissionType::Research, 2.0);
        assert!((m.difficulty - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mission_add_objective() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 1.0));
        assert_eq!(m.objectives.len(), 1);
    }

    #[test]
    fn mission_add_multiple_objectives() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 0.3));
        m.add_objective(MissionObjective::new("O2", "S2", 0.7));
        assert_eq!(m.objectives.len(), 2);
    }

    #[test]
    fn mission_assign_agent() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        assert_eq!(m.assigned_agents, vec!["alice"]);
    }

    #[test]
    fn mission_assign_agent_dedup() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.assign_agent("alice".into());
        assert_eq!(m.assigned_agents.len(), 1);
    }

    #[test]
    fn mission_assign_multiple_agents() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.assign_agent("bob".into());
        m.assign_agent("charlie".into());
        assert_eq!(m.assigned_agents.len(), 3);
    }

    #[test]
    fn mission_set_leader_assigned() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.set_leader("alice".into());
        assert_eq!(m.team_leader, Some("alice".into()));
    }

    #[test]
    fn mission_set_leader_not_assigned_noop() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.set_leader("bob".into());
        assert_eq!(m.team_leader, None);
    }

    #[test]
    fn mission_start_ok() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        assert!(m.start(42).is_ok());
        assert_eq!(m.status, MissionStatus::InProgress);
        assert_eq!(m.tick_started, Some(42));
    }

    #[test]
    fn mission_start_no_agents_error() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        let err = m.start(1).unwrap_err();
        assert!(err.contains("no assigned agents"));
    }

    #[test]
    fn mission_start_already_started_error() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        let err = m.start(2).unwrap_err();
        assert!(err.contains("InProgress"));
    }

    #[test]
    fn mission_complete_objective() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 1.0));
        m.complete_objective(0);
        assert!(m.objectives[0].completed);
    }

    #[test]
    fn mission_complete_objective_out_of_range_noop() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.complete_objective(99);
        assert!(m.objectives.is_empty());
    }

    #[test]
    fn mission_complete_with_result() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        let r = MissionResult {
            success_score: 0.9,
            conservation_error: 0.1,
            objectives_completed: 2,
            objectives_total: 3,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        };
        m.complete(100, r);
        assert_eq!(m.status, MissionStatus::Completed);
        assert_eq!(m.tick_completed, Some(100));
        assert!(m.result.is_some());
        assert!((m.result.as_ref().unwrap().success_score - 0.9).abs() < 1e-9);
    }

    #[test]
    fn mission_fail() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        m.fail(50, "Ran out of supplies".into());
        assert_eq!(m.status, MissionStatus::Failed);
        assert!(m.result.unwrap().lessons_learned[0].contains("supplies"));
    }

    #[test]
    fn mission_abort() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        m.abort(10);
        assert_eq!(m.status, MissionStatus::Aborted);
        assert_eq!(m.tick_completed, Some(10));
    }

    #[test]
    fn mission_is_active() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        assert!(!m.is_active());
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        assert!(m.is_active());
        m.complete(2, MissionResult {
            success_score: 1.0,
            conservation_error: 0.0,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        assert!(!m.is_active());
    }

    #[test]
    fn mission_is_active_after_fail() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        m.fail(5, "bad luck".into());
        assert!(!m.is_active());
    }

    #[test]
    fn mission_is_active_after_abort() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(1).unwrap();
        m.abort(5);
        assert!(!m.is_active());
    }

    #[test]
    fn mission_progress_empty() {
        let m = Mission::new("Test", MissionType::Research, 0.5);
        assert!((m.progress() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn mission_progress_partial() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 1.0));
        m.add_objective(MissionObjective::new("O2", "S2", 1.0));
        assert!((m.progress() - 0.0).abs() < 1e-9);
        m.complete_objective(0);
        assert!((m.progress() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn mission_progress_full() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 1.0));
        m.add_objective(MissionObjective::new("O2", "S2", 1.0));
        m.complete_objective(0);
        m.complete_objective(1);
        assert!((m.progress() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mission_duration_none() {
        let m = Mission::new("Test", MissionType::Research, 0.5);
        assert!(m.duration().is_none());
    }

    #[test]
    fn mission_duration_some() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(10).unwrap();
        assert!(m.duration().is_none());
        m.complete(25, MissionResult {
            success_score: 1.0,
            conservation_error: 0.0,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        assert_eq!(m.duration(), Some(15));
    }

    #[test]
    fn mission_duration_fail() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.start(10).unwrap();
        m.fail(20, "oops".into());
        assert_eq!(m.duration(), Some(10));
    }

    #[test]
    fn mission_team_size_zero() {
        let m = Mission::new("Test", MissionType::Research, 0.5);
        assert_eq!(m.team_size(), 0);
    }

    #[test]
    fn mission_team_size_two() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("a".into());
        m.assign_agent("b".into());
        assert_eq!(m.team_size(), 2);
    }

    #[test]
    fn mission_serde_roundtrip() {
        let mut m = Mission::new("Serde Test", MissionType::Defense, 0.7);
        m.assign_agent("agent1".into());
        m.add_objective(MissionObjective::new("Obj", "Done", 1.0));
        let json = serde_json::to_string(&m).unwrap();
        let back: Mission = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Serde Test");
        assert_eq!(back.mission_type, MissionType::Defense);
        assert_eq!(back.assigned_agents, vec!["agent1"]);
    }

    // ---- MissionResult ----

    #[test]
    fn mission_result_from_mission_all_completed() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.add_objective(MissionObjective::new("O1", "S1", 0.5));
        m.add_objective(MissionObjective::new("O2", "S2", 0.5));
        m.complete_objective(0);
        m.complete_objective(1);
        let r = MissionResult::from_mission(&m, 0.02);
        assert!((r.success_score - 1.0).abs() < 1e-9);
        assert_eq!(r.objectives_completed, 2);
        assert_eq!(r.objectives_total, 2);
        assert_eq!(r.agent_performance.len(), 1);
    }

    #[test]
    fn mission_result_from_mission_partial() {
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.assign_agent("alice".into());
        m.add_objective(MissionObjective::new("O1", "S1", 0.7));
        m.add_objective(MissionObjective::new("O2", "S2", 0.3));
        m.complete_objective(0);
        let r = MissionResult::from_mission(&m, 0.0);
        assert!((r.success_score - 0.7).abs() < 1e-9);
    }

    #[test]
    fn mission_result_from_mission_no_objectives() {
        let m = Mission::new("Test", MissionType::Research, 0.5);
        let r = MissionResult::from_mission(&m, 0.0);
        assert!((r.success_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mission_result_serde() {
        let r = MissionResult {
            success_score: 0.85,
            conservation_error: 0.12,
            objectives_completed: 3,
            objectives_total: 4,
            agent_performance: [("alice".into(), 0.85)].into(),
            lessons_learned: vec!["check supplies".into()],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: MissionResult = serde_json::from_str(&json).unwrap();
        assert!((back.success_score - 0.85).abs() < 1e-9);
        assert_eq!(back.lessons_learned[0], "check supplies");
    }

    // ---- MissionBriefing ----

    #[test]
    fn briefing_from_mission() {
        let mut m = Mission::new("Test", MissionType::Construction, 0.4);
        m.add_objective(MissionObjective::new("Build", "Done", 1.0));
        let b = MissionBriefing::from_mission(&m);
        assert!(b.required_skills.contains(&"building".to_string()));
        assert!(b.recommended_team_size >= 1);
        assert!(b.estimated_duration > 0);
    }

    #[test]
    fn briefing_rescue_skills() {
        let m = Mission::new("Res", MissionType::Rescue, 0.5);
        let b = MissionBriefing::from_mission(&m);
        assert!(b.required_skills.contains(&"search-and-rescue".into()));
    }

    #[test]
    fn briefing_risk_increases_with_difficulty() {
        let low = MissionBriefing::from_mission(&Mission::new("L", MissionType::Scouting, 0.1));
        let high = MissionBriefing::from_mission(&Mission::new("H", MissionType::Scouting, 0.9));
        assert!(high.risk_assessment > low.risk_assessment);
    }

    // ---- MissionLog ----

    #[test]
    fn log_create_and_get() {
        let mut log = MissionLog::new();
        let m = Mission::new("Alpha", MissionType::Scouting, 0.3);
        let id = log.create(m);
        assert!(log.get(&id).is_some());
    }

    #[test]
    fn log_get_nonexistent() {
        let log = MissionLog::new();
        assert!(log.get(&MissionId("nope".into())).is_none());
    }

    #[test]
    fn log_start_mission() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("Alpha", MissionType::Scouting, 0.3);
        m.assign_agent("alice".into());
        let id = log.create(m);
        assert!(log.start_mission(&id, 1).is_ok());
        assert!(log.get(&id).unwrap().is_active());
    }

    #[test]
    fn log_start_mission_fails_no_agents() {
        let mut log = MissionLog::new();
        let m = Mission::new("Alpha", MissionType::Scouting, 0.3);
        let id = log.create(m);
        assert!(log.start_mission(&id, 1).is_err());
    }

    #[test]
    fn log_update_objective() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("Test", MissionType::Research, 0.5);
        m.add_objective(MissionObjective::new("O1", "S1", 1.0));
        let id = log.create(m);
        log.update_objective(&id, 0);
        assert!(log.get(&id).unwrap().objectives[0].completed);
    }

    #[test]
    fn log_complete_mission() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("T", MissionType::Farming, 0.3);
        m.assign_agent("a".into());
        let id = log.create(m);
        log.start_mission(&id, 1).unwrap();
        log.complete_mission(&id, 10, MissionResult {
            success_score: 0.8,
            conservation_error: 0.05,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        assert_eq!(log.get(&id).unwrap().status, MissionStatus::Completed);
        assert!(!log.active.contains(&id));
    }

    #[test]
    fn log_fail_mission() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("T", MissionType::Farming, 0.3);
        m.assign_agent("a".into());
        let id = log.create(m);
        log.start_mission(&id, 1).unwrap();
        log.fail_mission(&id, 5, "disaster".into());
        assert_eq!(log.get(&id).unwrap().status, MissionStatus::Failed);
    }

    #[test]
    fn log_active_missions() {
        let mut log = MissionLog::new();
        let mut m1 = Mission::new("A", MissionType::Scouting, 0.3);
        m1.assign_agent("x".into());
        let mut m2 = Mission::new("B", MissionType::Construction, 0.5);
        m2.assign_agent("y".into());
        let id1 = log.create(m1);
        let id2 = log.create(m2);
        assert!(log.active_missions().is_empty());
        log.start_mission(&id1, 1).unwrap();
        assert_eq!(log.active_missions().len(), 1);
        log.start_mission(&id2, 1).unwrap();
        assert_eq!(log.active_missions().len(), 2);
        let r = MissionResult {
            success_score: 1.0,
            conservation_error: 0.0,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        };
        log.complete_mission(&id1, 10, r);
        assert_eq!(log.active_missions().len(), 1);
    }

    #[test]
    fn log_completed_missions() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("T", MissionType::Construction, 0.3);
        m.assign_agent("a".into());
        let id = log.create(m);
        assert!(log.completed_missions().is_empty());
        log.start_mission(&id, 1).unwrap();
        log.complete_mission(&id, 10, MissionResult {
            success_score: 1.0,
            conservation_error: 0.0,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        assert_eq!(log.completed_missions().len(), 1);
    }

    #[test]
    fn log_failed_missions() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("T", MissionType::Construction, 0.3);
        m.assign_agent("a".into());
        let id = log.create(m);
        log.start_mission(&id, 1).unwrap();
        log.fail_mission(&id, 5, "boom".into());
        assert_eq!(log.failed_missions().len(), 1);
    }

    #[test]
    fn log_agent_mission_history() {
        let mut log = MissionLog::new();
        let mut m1 = Mission::new("M1", MissionType::Scouting, 0.3);
        m1.assign_agent("alice".into());
        m1.assign_agent("bob".into());
        let mut m2 = Mission::new("M2", MissionType::Farming, 0.3);
        m2.assign_agent("alice".into());
        let mut m3 = Mission::new("M3", MissionType::Defense, 0.5);
        m3.assign_agent("charlie".into());
        log.create(m1);
        log.create(m2);
        log.create(m3);
        let alice_missions = log.agent_mission_history("alice");
        assert_eq!(alice_missions.len(), 2);
        let bob_missions = log.agent_mission_history("bob");
        assert_eq!(bob_missions.len(), 1);
        let charlie_missions = log.agent_mission_history("charlie");
        assert_eq!(charlie_missions.len(), 1);
        let no_one = log.agent_mission_history("nobody");
        assert!(no_one.is_empty());
    }

    #[test]
    fn log_success_rate_empty() {
        let log = MissionLog::new();
        assert!((log.success_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn log_success_rate_half() {
        let mut log = MissionLog::new();
        let mut m1 = Mission::new("M1", MissionType::Scouting, 0.3);
        m1.assign_agent("a".into());
        let id1 = log.create(m1);
        log.start_mission(&id1, 1).unwrap();
        log.fail_mission(&id1, 2, "fail".into());
        let mut m2 = Mission::new("M2", MissionType::Farming, 0.3);
        m2.assign_agent("b".into());
        let id2 = log.create(m2);
        log.start_mission(&id2, 1).unwrap();
        log.complete_mission(&id2, 2, MissionResult {
            success_score: 1.0,
            conservation_error: 0.0,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        assert!((log.success_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn log_stats_basic() {
        let log = MissionLog::new();
        let stats = log.stats();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.active, 0);
        assert!(stats.most_common_type.is_none());
    }

    #[test]
    fn log_stats_with_data() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("T", MissionType::Farming, 0.3);
        m.assign_agent("a".into());
        let id = log.create(m);
        log.start_mission(&id, 1).unwrap();
        log.complete_mission(&id, 10, MissionResult {
            success_score: 0.9,
            conservation_error: 0.1,
            objectives_completed: 0,
            objectives_total: 0,
            agent_performance: HashMap::new(),
            lessons_learned: vec![],
        });
        let stats = log.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.active, 0);
        assert!((stats.avg_success_score - 0.9).abs() < 1e-9);
        assert!((stats.avg_conservation_error - 0.1).abs() < 1e-9);
    }

    #[test]
    fn log_stats_most_common_type() {
        let mut log = MissionLog::new();
        log.create(Mission::new("A", MissionType::Farming, 0.3));
        log.create(Mission::new("B", MissionType::Farming, 0.3));
        log.create(Mission::new("C", MissionType::Scouting, 0.3));
        let stats = log.stats();
        assert_eq!(stats.most_common_type, Some(MissionType::Farming));
    }

    #[test]
    fn log_default() {
        let log = MissionLog::default();
        assert!(log.missions.is_empty());
    }

    #[test]
    fn log_serde_roundtrip() {
        let mut log = MissionLog::new();
        let mut m = Mission::new("M", MissionType::Research, 0.5);
        m.assign_agent("a".into());
        let id = log.create(m);
        log.start_mission(&id, 1).unwrap();
        let json = serde_json::to_string(&log).unwrap();
        let back: MissionLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back.missions.len(), 1);
        assert_eq!(back.active.len(), 1);
    }

    // ---- Templates ----

    #[test]
    fn bridge_builder_template() {
        let m = bridge_builder_mission();
        assert_eq!(m.name, "Bridge Builder");
        assert_eq!(m.mission_type, MissionType::Construction);
        assert!((m.difficulty - 0.4).abs() < 1e-9);
        assert_eq!(m.objectives.len(), 3);
    }

    #[test]
    fn farm_setup_template() {
        let m = farm_setup_mission();
        assert_eq!(m.name, "Farm Setup");
        assert_eq!(m.mission_type, MissionType::Farming);
        assert_eq!(m.objectives.len(), 4);
    }

    #[test]
    fn scout_report_template() {
        let m = scout_report_mission();
        assert_eq!(m.name, "Scout Report");
        assert_eq!(m.mission_type, MissionType::Scouting);
        assert_eq!(m.objectives.len(), 2);
    }

    #[test]
    fn conservation_audit_template() {
        let m = conservation_audit_mission();
        assert_eq!(m.name, "Conservation Audit");
        assert_eq!(m.mission_type, MissionType::ConservationAudit);
        assert_eq!(m.objectives.len(), 3);
    }

    #[test]
    fn rescue_operation_template() {
        let m = rescue_operation_mission();
        assert_eq!(m.name, "Rescue Operation");
        assert_eq!(m.mission_type, MissionType::Rescue);
        assert_eq!(m.objectives.len(), 4);
    }

    #[test]
    fn template_missions_count() {
        let templates = template_missions();
        assert_eq!(templates.len(), 5);
    }

    // ---- Integration ----

    #[test]
    fn full_mission_lifecycle() {
        let mut log = MissionLog::new();
        let mut m = bridge_builder_mission();
        m.assign_agent("alice".into());
        m.assign_agent("bob".into());
        m.set_leader("alice".into());

        let id = log.create(m);
        let stored = log.get(&id).unwrap();
        assert_eq!(stored.status, MissionStatus::Planning);
        assert_eq!(stored.assigned_agents.len(), 2);
        assert_eq!(stored.team_leader.as_deref(), Some("alice"));

        log.start_mission(&id, 100).unwrap();
        assert!(log.get(&id).unwrap().is_active());

        log.update_objective(&id, 0);
        assert!((log.get(&id).unwrap().progress() - (1.0 / 3.0)).abs() < 1e-9);
        log.update_objective(&id, 1);
        log.update_objective(&id, 2);
        assert!((log.get(&id).unwrap().progress() - 1.0).abs() < 1e-9);

        let result = MissionResult::from_mission(log.get(&id).unwrap(), 0.03);
        log.complete_mission(&id, 200, result);
        let finished = log.get(&id).unwrap();
        assert_eq!(finished.status, MissionStatus::Completed);
        assert_eq!(finished.tick_created, 0);
        assert_eq!(finished.tick_started, Some(100));
        assert_eq!(finished.tick_completed, Some(200));
        assert_eq!(finished.duration(), Some(100));
        assert!(finished.result.as_ref().unwrap().success_score > 0.99);

        let stats = log.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.active, 0);
    }
}
