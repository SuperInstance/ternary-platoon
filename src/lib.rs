#![forbid(unsafe_code)]

//! Group formation and coordinated movement for ternary agents.
//!
//! A `Platoon` is a group of agents (one leader, N followers) that move together
//! in a defined formation. The `PlatoonController` maintains formation during
//! movement, handles transitions between formation patterns (line, column, wedge,
//! grid), and supports splitting and merging of platoons. A communication relay
//! passes messages through the platoon chain.

use std::collections::HashMap;

// ── Position ───────────────────────────────────────────────────────────────

/// A 2D position for an agent in the platoon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position.
    pub fn distance_to(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// ── AgentId ────────────────────────────────────────────────────────────────

/// Identifier for an agent in the platoon. Wraps a string name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── FormationPattern ───────────────────────────────────────────────────────

/// The geometric arrangement of followers relative to the leader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationPattern {
    /// Followers in a horizontal line behind the leader.
    Line,
    /// Followers in a single-file column behind the leader.
    Column,
    /// Followers in a V-shape (wedge) behind the leader.
    Wedge,
    /// Followers in a grid (rows x cols) behind the leader.
    Grid { rows: usize, cols: usize },
}

impl FormationPattern {
    /// Compute the ideal positions for `n` followers given the leader at `origin`.
    /// The leader is always at `origin`; followers are offset based on the pattern.
    pub fn compute_positions(&self, origin: Position, n: usize) -> Vec<Position> {
        match self {
            FormationPattern::Line => {
                (0..n)
                    .map(|i| Position::new(origin.x + (i as i32 + 1) * 2, origin.y))
                    .collect()
            }
            FormationPattern::Column => {
                (0..n)
                    .map(|i| Position::new(origin.x, origin.y + (i as i32 + 1) * 2))
                    .collect()
            }
            FormationPattern::Wedge => {
                (0..n)
                    .map(|i| {
                        let offset = (i as i32 + 1) * 2;
                        let side = if i % 2 == 0 { -1 } else { 1 };
                        Position::new(origin.x + side * offset, origin.y + offset)
                    })
                    .collect()
            }
            FormationPattern::Grid { rows, cols } => {
                let mut positions = Vec::new();
                for r in 0..*rows {
                    for c in 0..*cols {
                        if positions.len() >= n {
                            break;
                        }
                        positions.push(Position::new(
                            origin.x + (c as i32) * 2,
                            origin.y + (r as i32 + 1) * 2,
                        ));
                    }
                }
                positions
            }
        }
    }
}

// ── Platoon ────────────────────────────────────────────────────────────────

/// A group of agents with a designated leader and formation pattern.
#[derive(Debug, Clone)]
pub struct Platoon {
    /// Unique name for this platoon.
    pub name: String,
    /// The leader agent.
    pub leader: AgentId,
    /// Follower agents in order.
    pub followers: Vec<AgentId>,
    /// Current formation pattern.
    pub formation: FormationPattern,
    /// Current positions of all agents (leader + followers).
    pub positions: HashMap<AgentId, Position>,
}

impl Platoon {
    /// Create a new platoon with a leader at the given position.
    pub fn new(name: &str, leader: AgentId, leader_pos: Position) -> Self {
        let mut positions = HashMap::new();
        positions.insert(leader.clone(), leader_pos);
        Self {
            name: name.to_string(),
            leader,
            followers: Vec::new(),
            formation: FormationPattern::Line,
            positions,
        }
    }

    /// Add a follower to the platoon, placing it according to current formation.
    pub fn add_follower(&mut self, agent: AgentId) {
        self.followers.push(agent);
        self.recalculate_positions();
    }

    /// Remove a follower from the platoon.
    pub fn remove_follower(&mut self, agent: &AgentId) -> bool {
        let before = self.followers.len();
        self.followers.retain(|a| a != agent);
        self.positions.remove(agent);
        if self.followers.len() != before {
            self.recalculate_positions();
            true
        } else {
            false
        }
    }

    /// Total number of agents (leader + followers).
    pub fn size(&self) -> usize {
        self.followers.len() + 1
    }

    /// Set formation and recalculate all follower positions.
    pub fn set_formation(&mut self, pattern: FormationPattern) {
        self.formation = pattern;
        self.recalculate_positions();
    }

    /// Move the leader to a new position and recalculate follower positions.
    pub fn move_leader_to(&mut self, new_pos: Position) {
        self.positions.insert(self.leader.clone(), new_pos);
        self.recalculate_positions();
    }

    /// Recalculate follower positions based on current formation and leader position.
    pub fn recalculate_positions(&mut self) {
        let leader_pos = *self.positions.get(&self.leader).unwrap_or(&Position::new(0, 0));
        let ideal = self.formation.compute_positions(leader_pos, self.followers.len());
        for (i, follower) in self.followers.iter().enumerate() {
            if i < ideal.len() {
                self.positions.insert(follower.clone(), ideal[i]);
            }
        }
    }

    /// Get the formation integrity: how many followers are within tolerance of ideal positions.
    pub fn formation_integrity(&self, tolerance: i32) -> usize {
        let leader_pos = *self.positions.get(&self.leader).unwrap_or(&Position::new(0, 0));
        let ideal = self.formation.compute_positions(leader_pos, self.followers.len());
        let mut count = 0;
        for (i, follower) in self.followers.iter().enumerate() {
            if i < ideal.len() {
                if let Some(actual) = self.positions.get(follower) {
                    if actual.distance_to(&ideal[i]) <= tolerance {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

// ── PlatoonController ──────────────────────────────────────────────────────

/// Controls platoon movement, maintaining formation during operations.
#[derive(Debug)]
pub struct PlatoonController {
    /// The platoon being controlled.
    pub platoon: Platoon,
    /// Maximum distance a follower can be from its ideal position before correction.
    pub tolerance: i32,
    /// Speed multiplier for movement (1 = normal, 2 = double speed, etc.).
    pub speed: u32,
}

impl PlatoonController {
    pub fn new(platoon: Platoon, tolerance: i32) -> Self {
        Self {
            platoon,
            tolerance,
            speed: 1,
        }
    }

    /// Move the platoon in a direction (dx, dy) by the speed amount.
    pub fn move_platoon(&mut self, dx: i32, dy: i32) {
        let step_x = dx * (self.speed as i32);
        let step_y = dy * (self.speed as i32);
        let leader_pos = self.platoon.positions.get(&self.platoon.leader)
            .copied()
            .unwrap_or(Position::new(0, 0));
        let new_pos = Position::new(leader_pos.x + step_x, leader_pos.y + step_y);
        self.platoon.move_leader_to(new_pos);
    }

    /// Transition to a new formation pattern.
    pub fn transition_formation(&mut self, pattern: FormationPattern) {
        self.platoon.set_formation(pattern);
    }

    /// Check and correct followers that have drifted beyond tolerance.
    /// Returns the number of followers corrected.
    pub fn correct_formation(&mut self) -> usize {
        let leader_pos = *self.platoon.positions.get(&self.platoon.leader)
            .unwrap_or(&Position::new(0, 0));
        let ideal = self.platoon.formation.compute_positions(leader_pos, self.platoon.followers.len());
        let mut corrected = 0;
        for (i, follower) in self.platoon.followers.iter().enumerate() {
            if i < ideal.len() {
                if let Some(actual) = self.platoon.positions.get(follower) {
                    if actual.distance_to(&ideal[i]) > self.tolerance {
                        self.platoon.positions.insert(follower.clone(), ideal[i]);
                        corrected += 1;
                    }
                }
            }
        }
        corrected
    }

    /// Set a follower's actual position (simulating real-world drift).
    pub fn set_follower_position(&mut self, agent: &AgentId, pos: Position) {
        self.platoon.positions.insert(agent.clone(), pos);
    }
}

// ── PlatoonSplit/Merge ─────────────────────────────────────────────────────

/// Result of splitting a platoon into two.
#[derive(Debug)]
pub struct SplitResult {
    pub primary: Platoon,
    pub secondary: Platoon,
}

/// Split a platoon at the given index. Followers [0..at) stay with the leader,
/// followers [at..] form a new platoon with a new leader.
pub fn split_platoon(
    platoon: &Platoon,
    at: usize,
    secondary_name: &str,
    secondary_leader: AgentId,
    secondary_leader_pos: Position,
) -> SplitResult {
    let mut primary = platoon.clone();
    let mut secondary = Platoon::new(secondary_name, secondary_leader, secondary_leader_pos);
    secondary.formation = platoon.formation;

    let removed: Vec<AgentId> = primary.followers.drain(at..).collect();
    for follower in &removed {
        primary.positions.remove(follower);
    }
    secondary.followers = removed;
    secondary.recalculate_positions();
    primary.recalculate_positions();

    SplitResult { primary, secondary }
}

/// Merge two platoons. The secondary's followers join the primary.
pub fn merge_platoons(primary: &mut Platoon, secondary: &Platoon) {
    for follower in &secondary.followers {
        primary.add_follower(follower.clone());
    }
}

// ── Communication Relay ────────────────────────────────────────────────────

/// A message relayed through the platoon chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayMessage {
    pub origin: AgentId,
    pub payload: String,
    pub hops: u32,
    pub max_hops: u32,
}

impl RelayMessage {
    pub fn new(origin: AgentId, payload: &str, max_hops: u32) -> Self {
        Self {
            origin,
            payload: payload.to_string(),
            hops: 0,
            max_hops,
        }
    }

    /// Relay the message one hop. Returns None if max hops exceeded.
    pub fn relay(&self) -> Option<RelayMessage> {
        if self.hops + 1 > self.max_hops {
            None
        } else {
            Some(RelayMessage {
                origin: self.origin.clone(),
                payload: self.payload.clone(),
                hops: self.hops + 1,
                max_hops: self.max_hops,
            })
        }
    }
}

/// Relay a message from one agent through the platoon chain to another agent.
/// Returns the list of agents the message passes through (including sender and receiver).
pub fn relay_through_platoon(
    platoon: &Platoon,
    from: &AgentId,
    to: &AgentId,
    payload: &str,
    max_hops: u32,
) -> Result<Vec<AgentId>, String> {
    // Build ordered chain: leader, followers[0], followers[1], ...
    let mut chain = vec![platoon.leader.clone()];
    chain.extend(platoon.followers.clone());

    let from_idx = chain.iter().position(|a| a == from)
        .ok_or_else(|| format!("Agent {} not in platoon", from))?;
    let to_idx = chain.iter().position(|a| a == to)
        .ok_or_else(|| format!("Agent {} not in platoon", to))?;

    let hops = (to_idx as i32 - from_idx as i32).abs() as u32;
    if hops > max_hops {
        return Err(format!("Requires {} hops, max is {}", hops, max_hops));
    }

    let (start, end) = if from_idx < to_idx {
        (from_idx, to_idx)
    } else {
        (to_idx, from_idx)
    };

    Ok(chain[start..=end].to_vec())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 4);
        assert_eq!(a.distance_to(&b), 7);
    }

    #[test]
    fn test_position_zero_distance() {
        let a = Position::new(5, 5);
        assert_eq!(a.distance_to(&a), 0);
    }

    #[test]
    fn test_formation_line_positions() {
        let leader = Position::new(0, 0);
        let positions = FormationPattern::Line.compute_positions(leader, 3);
        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], Position::new(2, 0));
        assert_eq!(positions[1], Position::new(4, 0));
        assert_eq!(positions[2], Position::new(6, 0));
    }

    #[test]
    fn test_formation_column_positions() {
        let leader = Position::new(0, 0);
        let positions = FormationPattern::Column.compute_positions(leader, 2);
        assert_eq!(positions[0], Position::new(0, 2));
        assert_eq!(positions[1], Position::new(0, 4));
    }

    #[test]
    fn test_formation_wedge_positions() {
        let leader = Position::new(0, 0);
        let positions = FormationPattern::Wedge.compute_positions(leader, 4);
        assert_eq!(positions.len(), 4);
        // Even indices go left, odd go right
        assert!(positions[0].x < 0); // left side
        assert!(positions[1].x > 0); // right side
    }

    #[test]
    fn test_formation_grid_positions() {
        let leader = Position::new(0, 0);
        let positions = FormationPattern::Grid { rows: 2, cols: 3 }.compute_positions(leader, 5);
        assert_eq!(positions.len(), 5);
        // Row 0: (0,2), (2,2), (4,2)  Row 1: (0,4), (2,4)
        assert_eq!(positions[0], Position::new(0, 2));
        assert_eq!(positions[4], Position::new(2, 4));
    }

    #[test]
    fn test_platoon_create_and_add_followers() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        assert_eq!(p.size(), 3);
        assert_eq!(p.followers.len(), 2);
    }

    #[test]
    fn test_platoon_remove_follower() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        assert!(p.remove_follower(&AgentId::new("a1")));
        assert_eq!(p.size(), 2);
        assert!(!p.remove_follower(&AgentId::new("a1"))); // already removed
    }

    #[test]
    fn test_platoon_formation_change() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        p.set_formation(FormationPattern::Column);
        // In column, followers should be below leader
        let a1_pos = p.positions.get(&AgentId::new("a1")).unwrap();
        assert_eq!(a1_pos.x, 0); // same x as leader
        assert!(a1_pos.y > 0); // below leader
    }

    #[test]
    fn test_platoon_move_leader() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.move_leader_to(Position::new(10, 10));
        let leader_pos = p.positions.get(&p.leader).unwrap();
        assert_eq!(*leader_pos, Position::new(10, 10));
        // Follower should have moved relative to new leader position
        let a1_pos = p.positions.get(&AgentId::new("a1")).unwrap();
        assert_eq!(a1_pos.x, 12); // line: leader.x + 2
    }

    #[test]
    fn test_formation_integrity() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        // All in perfect formation
        assert_eq!(p.formation_integrity(0), 2);
    }

    #[test]
    fn test_formation_integrity_with_drift() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        // Drift a1 away from ideal
        p.positions.insert(AgentId::new("a1"), Position::new(100, 100));
        assert_eq!(p.formation_integrity(5), 1); // only a2 is in formation
    }

    #[test]
    fn test_controller_move_platoon() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        let mut ctrl = PlatoonController::new(p, 5);
        ctrl.move_platoon(1, 0); // move right
        assert_eq!(*ctrl.platoon.positions.get(&ctrl.platoon.leader).unwrap(), Position::new(1, 0));
    }

    #[test]
    fn test_controller_correct_formation() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        let mut ctrl = PlatoonController::new(p, 5);
        // Drift the follower
        ctrl.set_follower_position(&AgentId::new("a1"), Position::new(50, 50));
        let corrected = ctrl.correct_formation();
        assert_eq!(corrected, 1);
        let a1_pos = ctrl.platoon.positions.get(&AgentId::new("a1")).unwrap();
        assert_eq!(*a1_pos, Position::new(2, 0)); // corrected to ideal
    }

    #[test]
    fn test_controller_speed() {
        let mut p = Platoon::new("alpha", AgentId::new("leader"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        let mut ctrl = PlatoonController::new(p, 5);
        ctrl.speed = 3;
        ctrl.move_platoon(1, 0);
        assert_eq!(*ctrl.platoon.positions.get(&ctrl.platoon.leader).unwrap(), Position::new(3, 0));
    }

    #[test]
    fn test_split_platoon() {
        let mut p = Platoon::new("alpha", AgentId::new("L"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        p.add_follower(AgentId::new("a3"));
        let result = split_platoon(&p, 2, "bravo", AgentId::new("L2"), Position::new(0, 0));
        assert_eq!(result.primary.size(), 3); // leader + a1 + a2
        assert_eq!(result.secondary.size(), 2); // L2 + a3
        assert_eq!(result.primary.followers.len(), 2);
        assert_eq!(result.secondary.followers.len(), 1);
    }

    #[test]
    fn test_merge_platoons() {
        let mut p1 = Platoon::new("alpha", AgentId::new("L1"), Position::new(0, 0));
        p1.add_follower(AgentId::new("a1"));
        let mut p2 = Platoon::new("bravo", AgentId::new("L2"), Position::new(10, 10));
        p2.add_follower(AgentId::new("a2"));
        merge_platoons(&mut p1, &p2);
        assert_eq!(p1.size(), 3); // L1, a1, a2
    }

    #[test]
    fn test_relay_message_hop() {
        let msg = RelayMessage::new(AgentId::new("a1"), "hello", 3);
        let relayed = msg.relay().unwrap();
        assert_eq!(relayed.hops, 1);
    }

    #[test]
    fn test_relay_message_max_hops() {
        let msg = RelayMessage::new(AgentId::new("a1"), "hello", 2);
        let r1 = msg.relay().unwrap();
        let r2 = r1.relay().unwrap();
        assert!(r2.relay().is_none()); // would be hop 3, max is 2
    }

    #[test]
    fn test_relay_through_platoon() {
        let mut p = Platoon::new("alpha", AgentId::new("L"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        p.add_follower(AgentId::new("a3"));
        let path = relay_through_platoon(&p, &AgentId::new("L"), &AgentId::new("a2"), "go", 5).unwrap();
        assert_eq!(path.len(), 3); // L -> a1 -> a2
        assert_eq!(path[0], AgentId::new("L"));
        assert_eq!(path[2], AgentId::new("a2"));
    }

    #[test]
    fn test_relay_through_platoon_too_far() {
        let mut p = Platoon::new("alpha", AgentId::new("L"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        p.add_follower(AgentId::new("a2"));
        let result = relay_through_platoon(&p, &AgentId::new("L"), &AgentId::new("a2"), "go", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_relay_reverse_direction() {
        let mut p = Platoon::new("alpha", AgentId::new("L"), Position::new(0, 0));
        p.add_follower(AgentId::new("a1"));
        let path = relay_through_platoon(&p, &AgentId::new("a1"), &AgentId::new("L"), "back", 5).unwrap();
        assert_eq!(path[0], AgentId::new("L"));
        assert_eq!(path[1], AgentId::new("a1"));
    }
}
