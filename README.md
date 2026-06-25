# Airplane Mode

**A coaching scribe that de-identifies on your phone — even a 2019 one, even with the radios off — and only ever lets a clean record leave.**

This repo is a **buildable demo and a reusable pattern**. It exists to make one bet watchable enough to argue about, and to hand application developers the *shape* of applying that bet to their own vertical.

> "AI's future will not be defined by who can build the largest datacenters. It will be defined by who can deliver the most intelligence per unit of energy and cost." — Vinod Khosla

If that's true, the most sensitive workloads should run **where the data already is** — on the device in someone's hand — instead of shipping the data to the compute. We prove it with a 1-bit [Bonsai](docs/seed/bonsai-ecosystem-brief.md) model running offline on an iPhone, de-identifying a (synthetic) mental-health coaching session on-device, with a verifier gate that **refuses to send anything until it's provably clean**.

The demo *is* the trust boundary, made watchable. And its architecture *is* the pitch: the sensitive workload becomes **one portable, owned core**; everything platform-specific is a swappable adapter — the literal shape of pulling a workload off the datacenter and repatriating it to the edge.

---

## Status: core + CLI + web Beat 1 built and working

- **The trust core** — `airplane-core` (Rust): rules executor + verifier gate + pipeline + pack loader, behind four ports (ADR-014). clippy-clean, unit-tested.
- **Reproduction (CLI)** — `./run.sh eval` runs the scrub over a 20-note synthetic golden set: **rules ∪ Bonsai-1.7B → 100% recall / 0 leakage** through owned code, deterministic (`eval/golden-run.txt`).
- **Beat 1 live (web)** — `./run.sh web` serves the on-phone "airplane-mode loop" (capture → on-device scrub → verifier gate → clean record → held-while-offline → flush → Slack card). Runbook: **[docs/demo/onboarding.md](docs/demo/onboarding.md)**.
- **The design + build loop** — `CANON.md` indexes the design corpus; `AGENTS.md` + `backlog/` run the build as a harnessed loop.

Still ahead: M2 (structurer/follow-up polish + the two ethical gates), the MCP shell, optional native iOS.

## Run the build loop (drop it into Codex or Claude Code)

This repo is built by a **harnessed loop**: an agent reads `AGENTS.md`, pulls the next unblocked task from `backlog/`, builds exactly that, runs `./run.sh gates`, commits, and stops — one task per iteration, with human review at milestone boundaries.

```bash
git clone <repo> && cd airplane-mode
# open in Codex or Claude Code, then:
#   "Read AGENTS.md and start the loop."
```

The agent will start at **M0** (author the `coach-session` pack + ~20 golden notes — the truth set everything else is measured against). The hard rules in `AGENTS.md` keep the trust boundary intact no matter who (or what) is building.

---

## The architecture (one core, many runtimes)

```
        airplane-core  (Rust · portable · signed · the "repatriated workload")
        rules executor · verifier gate · pipeline · pack loader
        depends only on PORTS:  InferenceProvider · SecureStore · Capture · Sink
                 ▲                      ▲                       ▲
         web shell (live)          CLI shell             MCP · iOS (planned)
        browser · laptop-edge  llama-server · file      same core, more reach
        "Beat 1, on any phone"  "numbers reproduce"     "an agent / a device too"
```

The **identical** recall-critical logic runs across every shell — which is what makes a reproduced number meaningful and the repatriation real, not asserted. The model is a *port*, not baked in.

## Make it yours in five files

Everything specific to a practice (or vertical) is a **pack** — five declarative files, no code: `recognizers/ · schema/ · policy/ · sink/ · eval/`. A pack can redefine your identifiers, record shape, policy, and destination — it can **never** see raw data, the redaction map, or the gate. So it's safe to write, and safe to share. Same signed core, your identifiers, no fork. (`packs/README.md`)

## What this is — honestly

A demo and a pattern: synthetic data only, one phone, one channel. We show the *shape* of what's possible, not a finished product. And we don't overclaim the model: PrismML's "intelligence density" is a self-coined metric that loses on raw benchmarks; "1-bit" is sign-only weights with grouped scale factors; frontier cloud still wins peak quality. The bet isn't that this beats GPT — it's that **the most sensitive work should run where the data lives**, and on a six-year-old phone it already can.

## Run Beat 1 on your phone

```bash
~/projects/bonsai/scripts/serve.sh    # the model server (separate terminal)
./run.sh web                          # the UI — prints the URL
```

On a phone on the same Wi-Fi, open the printed `http://<laptop-ip>:8088`.
On managed/corporate/guest Wi-Fi that blocks device-to-device traffic, use your iPhone's Personal Hotspot instead — full steps and troubleshooting in **[docs/demo/onboarding.md](docs/demo/onboarding.md)**.

## Repo map

| Path | What |
|---|---|
| `CANON.md` | index of the design canon |
| `files/` | the design corpus (RFCs, specs, ADRs) — `rfc-002-final-ship.md` is the build spec |
| `docs/superpowers/specs/` | the architecture design doc |
| `docs/seed/` | scoped Bonsai ecosystem brief |
| `AGENTS.md` · `CLAUDE.md` | the build-loop operating manual |
| `backlog/` | the M0–M5 work queue the loop pulls from |
| `crates/airplane-core/` · `shells/` · `packs/` · `gates/` | where the build lands |
| `run.sh` | the one entrypoint: `eval · scrub · gates · web` |

*Built to make intelligence come to the data — not the other way around.*
