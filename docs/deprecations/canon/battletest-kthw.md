# Battletest — Airplane Mode vs. Kubernetes the Hard Way
### Map every KTHW lab to our design, mark where we diverge and why, and find the holes
*The deepest platform engineers learn k8s through KTHW. If our architecture survives their mental model — and we speak it in their vocabulary — we earn the biggest adopter. The honest output is the gaps it exposes, not a comfortable mapping.*

---

## How to read

For each KTHW lab: what the engineer **learns to care about**, our **analog**, which **plane** it lives on (data = on-device, control = PHI-free backend), the **divergence and why**, and whether the plan **holds** or a **gap** is exposed.

The headline: most KTHW concerns map to our **control plane**; the deliberate divergences cluster at the **worker node** — because their worker is scheduled compute and ours is a phone you can't schedule onto. That divergence *is* the thesis.

---

## The mapping

| KTHW lab | What they care about | Our analog | Plane | Divergence & why | Holds? |
|---|---|---|---|---|---|
| **01 Prerequisites** (4 Debian machines) | A known, reproducible substrate | Hardware matrix: 1 laptop reproduces correctness, 1 iPhone 11+ proves on-device | both | They need 4 homogeneous servers; we need a laptop + a phone already in a pocket | ✅ (parity contract + Dockerfile) |
| **02 Jumpbox / client tools** | Admin entrypoint + pinned tooling | `run.sh` + pinned toolchain (model-by-hash, fork commit, lockfile) | control | Their jumpbox administers a remote cluster; ours runs locally and has **no remote admin into the data plane** (by design) | ✅ |
| **03 Compute resources / SSH / hostnames** | Addressable node inventory + secure access | Devices **pull** signed artifacts by version manifest; we never push or SSH in | control | Push-to-addressable-nodes → pull-by-devices; stronger (no inbound attack surface on PHI) | ⚠️ **Gap 2** (manifest + revocation) |
| **04 CA + TLS certificates** | A trust root + identity for every component | Artifact **signing**: cosign/Sigstore + Rekor + SLSA + in-toto; the pack loader verifies signature before load | control + device verify | PKI secures *connections* (mTLS); ours secures *artifacts* — our components don't chatter over a network, so there's nothing to mTLS | ⚠️ **Gap 1** (key custody) |
| **05 Kubeconfigs (auth)** | Who may talk to the control plane | Who may **publish a pack** (signing identity + eval-gate admission) + on-device operator/consent auth | control + data | No central API to authenticate *to* for the scrub; auth is publish-side and device-side | ⚠️ tied to Gap 1 |
| **06 Data encryption config + key** | Secrets encrypted at rest in etcd | The **Enclave**: raw input + redaction map held only in Secure Enclave, encrypted, never leaving hardware | data | We encrypt the secret on the *user's own device* with no admin reach — stronger than shared-etcd-at-rest | ⚠️ **Gap 3** (trajectory store at-rest) |
| **07 etcd (state store)** | A consistent source of truth | Pack/model **registry + version manifest** (control truth); enclave + local pack (device state) | control + data | Quorum-coordinated mutable store → signed artifact registry devices reconcile to; no quorum because devices share no mutable state | ⚠️ tied to Gap 2 |
| **08 Control plane (apiserver/scheduler/controller + RBAC)** | The brain reconciling desired→actual + authz | Registry + **eval CI gate** + OSCAL conformance + reconcile/push of signed manifests; RBAC ≈ PHI-blind pack contract + signature + eval admission | control | **No scheduler** — nothing is scheduled; the workload's location is fixed at install (the device with the data). apiserver/controller analog exists; scheduler does not | ✅ (the centerpiece divergence) |
| **09 Worker nodes (containerd/kubelet/kube-proxy/CNI)** | Nodes that run workloads + runtime + net | The **phone** runs the scrub — but it is **not a kubelet**: no scheduled pods, no node API, no exec. App client + mlx-swift runtime; **no pod network** | data | The kubelet's whole job (accept scheduled work, report status, allow exec) is exactly what we forbid on a PHI-holding device | ⚠️ **Gap 4** (fleet observability) |
| **10 kubectl remote access** | Remote inspect/control of workloads | **Intentionally none** into the data plane; control plane has its own admin surface | data | Remote exec into a workload is the backdoor we're preventing — absence is a feature | ✅ |
| **11 Pod network routes (CNI)** | Pods reach each other; policy controls flows | Scrub runs with **no network** (airplane-mode); only signed-pull in + clean-record-push out; gate = default-deny egress | data | They build a network so workloads talk; we build the *absence* of network dependence for the sensitive op | ✅ (airplane mode maps here directly) |
| **12 DNS add-on (CoreDNS)** | Services discover each other by name | Pack/model **version discovery** via signed manifest; sink endpoint in the pack | control | Artifact-version resolution, not service-to-service DNS | ✅ (tied to manifest) |
| **13 Smoke test** (encryption verified, deploy, exec, logs, service) | End-to-end proof + the security property holds | The **eval harness + reproduction ladder**: "verify secrets encrypted in etcd" ≈ "verify no identifier escapes the gate"; `run.sh demo` ≈ deploy/exec | both | Their smoke test proves workloads run; ours proves the **no-leak property** *and* reproduces the numbers for a stranger | ✅ (strongest mapping) |
| **14 Cleanup** | Clean teardown / start fresh | **Deletion**: enclave wipe + local pack removal; control plane holds only signed artifacts + de-identified data | both | Their cleanup tears down owned infra; ours satisfies the consumer's MHMDA deletion right — trivial because regulated data lives on the device | ✅ (a legal selling point) |

---

## The gaps the battletest exposed

The rigor earned its keep. Four real holes, each traceable to a lab:

**Gap 1 — Signing key custody & rotation** *(labs 04, 05).*
KTHW makes you generate and fiercely protect the CA key. We say "signed packs" but never say **who holds the signing root or how it rotates.** Direction: keyless signing (Sigstore/Fulcio, identity-based, no long-lived key) with Rekor transparency, or an HSM-held root with a documented rotation + revocation policy. This is the #1 gap — the signing root is the entire trust anchor.

**Gap 2 — Version manifest + revocation** *(labs 03, 07, 12).*
Devices pull by manifest, but we never specified **how a device knows the right version, or how a bad pack/model is revoked** so devices stop pulling it. Direction: a signed, monotonically-versioned manifest + a revocation list the device checks before load.

**Gap 3 — Trajectory store: at-rest encryption + location** *(lab 06).*
We specified trajectories are *de-identified*; we never specified they're **encrypted at rest or where they live** (on-device vs. aggregated). De-identified is not the same as protected. Direction: enclave-backed encryption on-device; if ever aggregated, the minimum-cohort/DP decision from RFC open-question 2 gates it.

**Gap 4 — PHI-free fleet observability** *(labs 09, 10).*
Without a kubelet reporting status, **how do you know the fleet is healthy and on the right version** — without a backdoor into a PHI-holding device? This is the real tension: "no remote access into the data plane" vs. "know your fleet." Direction: opt-in, de-identified health/version telemetry only (version, gate-pass-rate, crash counts — never content), pushed by the device, never pulled.

---

## Verdict

The plan holds against KTHW's rigor. The architecture maps **concept-for-concept** — trust root, encryption at rest, control plane, smoke test, cleanup all have clean analogs — and every divergence (no scheduler, no kubelet, no pod network, no remote exec) is **intentional and is the thesis**: the worker is a phone you don't schedule onto. The battletest exposed four real gaps, all in the control plane / supply chain, none fatal, each with a clear fix direction.

**The one-liner for the biggest adopter:**
> Everything you learned to care about in Kubernetes the Hard Way has an analog here — the CA becomes artifact signing, etcd becomes a signed registry, encryption-at-rest becomes the enclave, the smoke test becomes the eval harness. The difference is the worker node: it's a phone, you don't schedule onto it, and there's no kubelet to exec into — because the whole point is that the data never leaves it.
