# State Machine System — Workload · Data Model · Control Planes
### One coordinated system of machines, not one swamp. Invariants as unreachable transitions.
*The formal backbone of the canon. Each stateful entity and plane gets its own machine; the verifier gate is the guard they share; the security/legal/ethical invariants become edges that do not exist.*

---

## The discipline

- **A system of small machines**, one per stateful entity and per plane — never one mega-machine (that's the over-modeling smell).
- **Invariants are unreachable transitions.** "Raw input never reaches the sink" isn't a rule we enforce — it's an edge the graph doesn't contain. Stronger than a check; it's a graph property (Constitution IX).
- **Shared guards** connect the machines: the verifier gate guards three of them; the eval+gates guard the Pack machine.
- **Goal states encode ethics.** The Commitment machine's terminal is `Receded`, not max-engagement — the anti-dependence ethic (ADR-012) as where the machine is *trying to go*.

---

## Data model (entities + their governing machine)

```mermaid
classDiagram
  Client "1" --> "*" Session
  Session "1" --> "1" Record
  Record "1" --> "*" Commitment
  Commitment "1" --> "*" FollowUp
  Session ..> Trajectory : de-identified
  Pack ..> Record : configures
  class Record { FSM: Raw to Delivered (data plane) }
  class Commitment { FSM: Open to Receded (autonomy exit) }
  class FollowUp { FSM: Pending to ActedOn/Dismissed }
  class Trajectory { FSM: Captured to Stored/Rejected }
  class Pack { FSM: Authored to Active/Revoked (control plane) }
  class Device { FSM: Pulling to Active/Quarantined }
```

---

## Data-plane machines (on-device)

### Record — the workload (one session's journey)
```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Capturing: consent given
  Capturing --> Scrubbing: note captured
  Scrubbing --> Gated: scrub complete
  Gated --> Structured: clean
  Gated --> Blocked: residual identifier
  Structured --> Delivered: online
  Structured --> Queued: offline
  Queued --> Delivered: back online
  Delivered --> [*]
  Blocked --> [*]
```
There is **no edge** from `Capturing` or `Scrubbing` to `Delivered`/`Queued`. Every path out passes through `Gated`. The trust boundary is the graph's shape.

### Commitment — the autonomy exit (the ethical machine)
```mermaid
stateDiagram-v2
  [*] --> Open: committed in session
  Open --> Scheduled
  Scheduled --> Nudged: client-paced cadence
  Nudged --> Completed: acted
  Nudged --> Drifting: no action
  Drifting --> Nudged: bounded re-engage
  Drifting --> Dropped: declines / mutes
  Completed --> Internalized: self-sustains
  Internalized --> Receded: system recedes
  Completed --> Reviewed: next session
  Dropped --> Reviewed
  Receded --> [*]
  Reviewed --> [*]
```
`Receded` is the goal terminal. There is **no** `MaximallyEngaged` state — the machine cannot pursue dependence because the graph offers no path to it.

### FollowUp
```mermaid
stateDiagram-v2
  [*] --> Pending
  Pending --> Sent: gate clean AND client cadence allows
  Pending --> Blocked: gate residual
  Sent --> ActedOn
  Sent --> Dismissed
  ActedOn --> [*]
  Dismissed --> [*]
  Blocked --> [*]
```

### Trajectory — RL-environment ingress
```mermaid
stateDiagram-v2
  [*] --> Captured: loop completes
  Captured --> Stored: gate clean, de-identified, enclave-encrypted
  Captured --> Rejected: gate residual
  Stored --> [*]
  Rejected --> [*]
```
No edge from `Captured` to `Stored` that bypasses the gate. The RL environment can only ingest gate-clean trajectories.

---

## Control-plane machines (PHI-free)

### Pack — the harnessed delivery loop
```mermaid
stateDiagram-v2
  [*] --> Authored
  Authored --> Eval: submit
  Eval --> GatedFail: recall or gates fail
  Eval --> GatedPass: recall ok AND all gates pass
  GatedFail --> Authored: fix (scar to harness)
  GatedPass --> Signed: keyless sign
  Signed --> Published: manifest bump
  Published --> Active: device verifies
  Active --> Superseded: new version
  Active --> Revoked: revocation
  Superseded --> [*]
  Revoked --> [*]
```
No edge from `Authored` or `GatedFail` to `Signed`. **Nothing ships unverified.**

### Device — reconciliation
```mermaid
stateDiagram-v2
  [*] --> Pulling
  Pulling --> Verifying: artifact fetched
  Verifying --> Active: signature AND manifest ok, not revoked
  Verifying --> Rejected: bad signature or revoked
  Active --> Pulling: manifest changed
  Active --> Quarantined: version revoked
  Quarantined --> Pulling
  Rejected --> [*]
```
No edge from `Verifying` to `Active` without signature + manifest + revocation check. **Nothing runs unsigned.**

---

## Composition — the system and its boundary

```mermaid
flowchart TB
  subgraph DATA["Data plane · on-device"]
    REC[Record FSM]
    COM[Commitment FSM]
    FUP[FollowUp FSM]
    TRAJ[Trajectory FSM]
  end
  subgraph CTRL["Control plane · PHI-free"]
    PACK[Pack FSM]
    DEV[Device FSM]
  end
  GATE{{Verifier Gate · shared guard}}
  REC -. guarded by .-> GATE
  FUP -. guarded by .-> GATE
  TRAJ -. guarded by .-> GATE
  PACK -- signed artifact --> DEV
  DEV -- configures --> REC
  REC --> COM
  COM --> FUP
  REC --> TRAJ
  DATA -. de-identified telemetry .-> CTRL
  CTRL -. signed packs / models .-> DATA
```
Across the plane boundary: only **signed artifacts down** and **de-identified telemetry up**. PHI has no edge that crosses it.

---

## The invariants, as unreachable transitions (the formal payoff)

| Invariant | Encoded as | Machine |
|---|---|---|
| Raw input never reaches the sink | no `Scrubbing → Delivered` edge | Record |
| The RL environment never ingests PHI | no `Captured → Stored` bypassing gate | Trajectory |
| Nothing runs unsigned | no `Verifying → Active` without checks | Device |
| Nothing ships unverified | no `Eval → Signed` without GatedPass | Pack |
| The system cannot pursue dependence | no `MaximallyEngaged` state exists | Commitment |
| PHI never leaves the device | no PHI edge crosses the plane boundary | Composition |

Each is a property of the graph, not a runtime hope.

---

## Transition → test mapping (feeds the eval harness)

- **Every transition** becomes a test case (does the guard fire correctly?).
- **Every guard** becomes a harness (the gates from the harnessed loop).
- **Every unreachable edge** becomes an assertion (the test suite proves the edge cannot be taken).
- **Every terminal** becomes an acceptance check (`Delivered`, `Receded`, `Stored`, `Active`).

The FSM is not documentation beside the build — it generates the test plan.

---

## One-line test
> The whole system is six small machines sharing one gate; the trust boundary, the no-leak rule, the signing requirement, and the anti-dependence ethic are all edges the graph does not contain — and every edge it *does* contain is a test.
