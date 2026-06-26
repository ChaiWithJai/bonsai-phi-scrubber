# The Clinic Pack Pattern
### Do it once. Let any clinic or system run it without forking.
*Capstone of the PHI-scrubber design. Consumes the constitution and the technical survey.*

---

## The principle

**One signed core. Many declarative clinic packs.**

You build, harness, and certify the core exactly once. Everything that differs between Cityblock and the next clinic is a *pack* — pure configuration and declarative assets, no executable code. A new clinic authors a pack against a fixed contract and runs the identical signed core. Nobody forks anything.

This is the Terraform-module / provider model applied to a regulated edge workload: the module is certified once; adopters instantiate it with their own variables.

```
        ┌─────────────────────────────────────────────┐
        │            CORE RUNTIME (signed)             │   built + certified ONCE
        │  pipeline · rules executor · Bonsai ·        │   never forked by adopters
        │  verifier gate · enclave · pack loader       │
        └───────────────────────▲─────────────────────┘
                                 │ loads (declarative, PHI-blind)
        ┌───────────────┬────────┴────────┬───────────────┐
        │ Cityblock pack│  Clinic-B pack  │  System-C pack │   authored by anyone
        └───────────────┴─────────────────┴───────────────┘   against a fixed contract
```

---

## Layer 1 — The Core Runtime (build once, sign, freeze)

The parts that must be correct and ideally certified. An extender never edits these.

- **Pipeline runner** — Source → Scrubber → Structurer → Sink, with the offline queue.
- **Rules executor** (native Swift) — runs the pack's recognizer definitions.
- **Bonsai integration** (mlx-swift) — the contextual/ambiguous de-id layer.
- **Verifier gate + egress control** — re-scans scrubbed output; blocks the network on any residual hit. The trust boundary lives here.
- **Enclave handler** — writes the redaction map only to the secure enclave; never to disk or wire.
- **Pack loader** — validates a pack's signature and contract conformance before loading.
- **Sink-adapter interface** — the seam a sink plugs into (the Slack reference sink ships in-box).

Discipline: this is what you reproduce-by-hand (Constitution III), harness every scar of (VI), review every line of (VII), and certify once. Its security properties hold for every clinic because no clinic can change it.

---

## Layer 2 — The Clinic Pack (the extension surface)

A pack is a signed bundle of **declaration only**. If it could contain code, it could touch PHI; it cannot, so it can't. Five components:

```
clinic-pack/
├── pack.hcl                 # name, version, target core version, signature ref
├── recognizers/
│   ├── members.json         # org identifier rule pack: regex + context + checksum
│   ├── partners.json        #   + deny/allow lists (authored in Presidio, exported)
│   └── ...
├── schema/
│   └── home-visit.json      # note archetype(s): fields, types, required/optional
├── policy.hcl               # Safe Harbor | Expert Determination; per-entity operators;
│                            #   model size (1.7B / 4B); recall threshold
├── sink.hcl                 # sink type(s); channel/endpoint mapping;
│                            #   credential SOURCE (e.g. keychain ref) — never the secret
└── eval/
    ├── golden/*.txt         # synthetic-PHI notes (never real PHI)
    └── expected/*.json      # required redactions per note
```

What a pack can change: which strings are identifiers, which policy applies, what the record schema is, and where clean records go.
What a pack can never do: see raw PHI, read the redaction map, alter the verifier, or ship executable code.

That boundary is what makes the extension surface safe to open to strangers.

---

## Layer 3 — Adoption (the "anyone can try it" path)

```
1. INSTALL        the signed core (App Store build / MDM).
2. COPY           the reference pack (cityblock-chp) as your starting point.
3. CONFIGURE      edit the five components for your org:
                    recognizers · schema · policy · sink · eval notes.
4. EVAL           run the harness:  pack must pass the recall gate or it does not ship.
                    (this is your mini-certification — generalized from Philter re-cert)
5. SIGN           sign the pack.
6. DEPLOY         load into the core; push to your device fleet.
                    Mac-mini sites: minis serve signed pack + model artifacts;
                    phones pull by version manifest. PHI never on the LAN.
```

Step 4 is the gate that lets you trust packs you didn't write: a pack is only as good as its proven recall on its own golden notes, and the harness enforces that automatically.

---

## Distribution — registry, not scheduler

Packs and the Bonsai model are **signed OCI artifacts** (cosign signatures, SLSA-style provenance). A clinic publishes its pack to a registry or ships the bundle directly; devices reconcile against a version manifest (Argo/Flux-style on the control plane). "Extendible to anyone" is literal: anyone authors a pack against the public contract and publishes it; the core everyone runs is the same signed binary.

The control plane is PHI-free by construction — it moves version numbers, signatures, eval scores, and de-identified audit counts. It never moves PHI, so it carries no compliance weight and its tooling choice is free.

---

## The reference pack (worked example)

Ship `cityblock-chp` as the canonical example every adopter copies:

- **recognizers/** — Cityblock member-ID format, partner-org names, internal codes, on top of the Safe Harbor 18 baseline.
- **schema/home-visit.json** — `{member_pseudonym, visit_type, sdoh_flags[], follow_ups[], risk_signals[], next_touch}`.
- **policy.hcl** — Safe Harbor; redaction (non-reversible); model `bonsai-1.7b`; recall threshold `0.99`.
- **sink.hcl** — Slack; care-team → channel map; bot token sourced from device keychain.
- **eval/** — ~20 synthetic home-visit notes with hand-labeled expected redactions (the by-hand work from Constitution III, turned into the permanent test set).

A second clinic forks *this pack*, not the core. That distinction is the entire design.

---

## The invariant (print this on the wall)

> The core is signed and certified once; the pack is declarative and PHI-blind. A pack can redefine what an identifier is and where clean records go — it can never see raw PHI, read the redaction map, or change the verifier. So anyone may write a pack, and the trust boundary still holds.

---

## Try-it-at-your-clinic, in one breath

Install the core, copy `cityblock-chp`, change five files to match your identifiers / notes / policy / sink, pass the eval gate, sign, deploy. Same scrubber, same guarantees, your clinic — and you never touched a line of the code that matters.
