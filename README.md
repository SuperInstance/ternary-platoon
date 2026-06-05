# ternary-platoon: Group formation and coordinated movement for ternary agents

## Why This Exists

When ternary agents operate as a team — think Ensign specialist squads moving between rooms — they need to stay in formation. A loose gaggle of agents wandering independently isn't a platoon. This crate defines platoon structure (leader + followers), formation patterns (line, column, wedge, grid), movement with formation maintenance, splitting and merging, and communication relay through the chain of agents.

## Core Concepts

- **Platoon**: A named group with one leader and N followers. The leader's position drives all follower positioning.
- **Formation pattern**: The geometric arrangement — `Line` (horizontal row), `Column` (single-file), `Wedge` (V-shape), `Grid` (rows × columns). Each pattern computes ideal follower positions relative to the leader.
- **PlatoonController**: Wraps a platoon and maintains formation integrity during movement. Corrects followers that drift beyond a tolerance threshold.
- **Formation transition**: Changing from one pattern to another — all followers reposition.
- **Split/Merge**: Breaking one platoon into two (splitting at an index) or combining two platoons into one.
- **Communication relay**: Messages hop through the platoon chain from sender to receiver with a configurable max-hop limit.

## Quick Start

```toml
[dependencies]
ternary-platoon = "0.1"
```

```rust
use ternary_platoon::*;

let mut platoon = Platoon::new("alpha-team", AgentId::new("commander"), Position::new(0, 0));
platoon.add_follower(AgentId::new("scout-1"));
platoon.add_follower(AgentId::new("scout-2"));
platoon.set_formation(FormationPattern::Wedge);

let mut ctrl = PlatoonController::new(platoon, 5);
ctrl.move_platoon(1, 0); // move right, followers maintain wedge
ctrl.transition_formation(FormationPattern::Column);
let corrected = ctrl.correct_formation(); // snap any drifters back
```

## API Overview

| Type | Description |
|------|-------------|
| `Position` | 2D coordinate (x, y) with Manhattan distance |
| `AgentId` | Named identifier for an agent in the platoon |
| `FormationPattern` | Enum: `Line`, `Column`, `Wedge`, `Grid { rows, cols }` |
| `Platoon` | Leader + followers with positions and formation |
| `PlatoonController` | Drives movement and formation correction |
| `RelayMessage` | Message with hop counter and max-hop limit |
| `split_platoon()` | Split at index into two platoons |
| `merge_platoons()` | Absorb one platoon's followers into another |
| `relay_through_platoon()` | Route a message through the chain |

## How It Works

Formation positions are computed purely from the leader's position and the pattern. Each pattern is a deterministic offset function: `compute_positions(origin, n)` returns `n` positions. During movement, the leader moves first, then all follower ideal positions are recalculated. The `PlatoonController` tracks a tolerance; any follower whose actual position exceeds the tolerance from its ideal gets snapped back on `correct_formation()`.

Relay routing uses a simple linear chain: `[leader, follower_0, follower_1, ...]`. A message from agent at index i to agent at index j traverses the sub-chain `[i..j]`. The hop count must not exceed `max_hops`.

## Known Limitations

- All positions are integer-valued (`i32`). Sub-cell precision isn't supported.
- Formation computation assumes agents occupy distinct cells with spacing of 2 units. No collision avoidance.
- Relay routing is strictly linear through the chain — there's no mesh or multi-path routing.
- Grid formation truncates if `rows * cols > follower_count` (excess slots are unused).
- No asynchronous movement — all position updates are synchronous.

## Use Cases

1. **Room-to-room team movement**: A squad of Ensign specialists moves through a Codespace building in wedge formation, with the leader navigating and followers maintaining positions.
2. **Patrol routing**: A security platoon in column formation sweeps corridors, splitting at intersections to cover multiple paths, then merging back.
3. **Communication chain in degraded networks**: When direct radio isn't available, messages relay hop-by-hop through a platoon line to reach a distant agent.

## Ecosystem Context

Part of the SuperInstance ternary fleet. Works with `ternary-agent` (individual agents), `ternary-ensign` (specialist roles that fill platoon positions), and `ternary-swarm` (larger-scale multi-platoon coordination). Downstream of `ternary-protocol` for message formatting.

## License

MIT

## See Also
- **ternary-constellation** — related
- **ternary-captain** — related
- **ternary-consensus** — related
- **ternary-sync** — related
- **ternary-mesh** — related
- **ternary-room** — related

