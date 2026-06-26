# DevOps-First Design — Airplane Mode
### Meet the platform-engineer adopter where they are — without breaking the trust boundary
*New workstream. The adopter does many POCs and productionizes templates; they want local, docker, or k8s. Give them parity — for the right surfaces only.*

---

## The call

**Yes — design for run-surface parity (local / docker / k8s), but parity applies only to the server-side surfaces: the eval/reproduction harness and the control plane. The on-device scrub never enters a cluster.**

This is not a contradiction of "we killed k8s on the data path" (ADR-002, ADR-006). It's the same split applied to packaging: the data plane is a phone; the control plane and the eval harness are server-side and belong wherever the adopter already runs things.

---

## The persona (who pulls this down)

A platform / DevOps engineer who runs many proofs-of-concept, demos stakeholders, then productionizes the winners as reusable templates. They already have config, a CI pipeline, and a k8s cluster. Their adoption move is: drop the template into their existing pipeline, wire the gate into CI, and ship.

This persona *is* the ecosystem extender from the clinic-pack pattern. The coach-pack is the "extended template" they productionize. Meeting them is the highest-leverage adoption investment we can make — because they adopt by templating, and they bring their org with them.

### Their lifecycle → our surfaces
| Their stage | Surface we give them | What runs there |
|---|---|---|
| **POC** | Local (`./run.sh`) | scrub + eval on golden notes |
| **Demo** | Docker / compose | the full desktop loop, portable |
| **Productionize** | Helm / kustomize on k8s | the **control plane** + eval-at-scale — *not the scrub* |

---

## The boundary (the line that must hold)

This is the negative-space decision (Constitution IV). What gets the k8s treatment, and what never does:

**Gets parity (server-side, PHI-free):**
- The eval / reproduction harness (scrub + gate + scorer over golden notes).
- The control plane: signed pack registry, eval CI gate, OSCAL conformance, model/manifest distribution to the device fleet.

**Never enters a cluster (data plane):**
- The on-device scrubber, verifier gate runtime, enclave, and the iOS app.
- Raw input, the redaction map, anything PHI.

> k8s touches packs, models, eval, signing, and distribution. It never touches raw PHI or the scrub runtime. If a design has PHI flowing into a pod, it's wrong.

---

## The run-surface parity contract

The same core behaves identically across surfaces. Twelve-factor-style:
- **Config is external** — env vars + mounted pack files; no surface-specific code paths.
- **Model fetched by hash** everywhere; checksum-verified before run.
- **The eval is stateless and deterministic** (temp 0, pinned hash/commit) — identical output local, in docker, in a k8s Job.
- **The control plane is the only stateful component** (the registry, the trajectory/eval store).
- **One entrypoint, same verbs** across surfaces: `eval | demo | scrub`.

If a result differs across surfaces, that's a bug, not a configuration difference.

---

## The three packaging targets

**1. Local (`./run.sh`) — the POC front door.**
Already the Tier-1 reproduction path. The platform engineer's first 60 seconds.

**2. Docker / compose — the demo surface.**
`docker compose up` brings the CLI + a Slack webhook (or mock). OS-independent; hermetic; the surface they show a stakeholder. Already half-built (the Dockerfile in the implementation plan).

**3. Helm / kustomize on k8s — the productionize surface.**
Runs the **control plane**, not the scrub:
- signed pack registry (OCI artifacts; cosign/SLSA/in-toto),
- the **eval gate as policy-as-code** in their CI (OPA/Kyverno-JSON) — a pack promotes only if recall holds,
- OSCAL conformance mapping (MHMDA + Safe Harbor) as machine-checkable controls,
- model + manifest distribution to the device fleet (MDM / pull).
And, usefully, **eval-at-scale**: regression-test many packs / recognizers as k8s Jobs in CI.

---

## The unlock for this persona: eval-as-CI-gate

The single feature that turns "a demo repo" into "a template I productionize": the eval harness wired into *their* pipeline as a gate. A contributed pack (theirs or a third party's) only ships if it passes the recall gate in CI. That is the policy-as-code pattern from the CNCF work, now serving their pipeline. It's also the same gate that powers our reproducibility story — one mechanism, three jobs (our build gate, the stranger's reproduction proof, their CI promotion gate).

---

## Build sequence (design now, build by demand)

Do **not** draw the owl — no Helm chart before the CLI reproduces.
1. **Now (at M1):** design the parity contract (external config, fetch-by-hash, stateless deterministic eval). Costs almost nothing if done as the CLI is built; expensive to retrofit.
2. **M1:** ship local + Dockerfile (already planned).
3. **M2:** add compose when there's a full loop to demo.
4. **Later (gated, on real adopter demand):** Helm chart + control plane. Build it when someone productionizes, not speculatively.

The deliverable *now* is the contract, not the manifests.

---

## One-line test
> The same core runs identically local, in docker, and as a k8s Job; the platform engineer wires our eval into their CI as a promotion gate; and at no point does raw PHI or the scrub runtime enter a cluster — k8s serves the packs, the phone does the scrubbing.
