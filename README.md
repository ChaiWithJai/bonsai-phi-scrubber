# Airplane Mode

**De-identify a sensitive note on the edge — and let only a clean record leave.** A 1-bit PrismML **Bonsai** model scrubs a (synthetic) mental-health coaching session locally; a **verifier gate** refuses to send anything until it's provably clean; only the de-identified record reaches Slack.

> "AI's future will not be defined by who can build the largest datacenters. It will be defined by who can deliver the most intelligence per unit of energy and cost." — Vinod Khosla

The bet: the most sensitive workloads should run **where the data already is**, not shipped to a datacenter. This repo makes that bet *watchable* — and hands an application developer the **pattern** to apply it to their own vertical.

---

## What's shipped vs. the vision (read this first)

**Shipped — Beat 1, runnable today.** A phone-driven web demo: you dictate a note on your phone, it's scrubbed on a **local edge node** (your laptop, running Bonsai offline), the verifier gate blocks egress until the output is provably clean, and the de-identified record posts to Slack for real.
*Honest scope:* in this web build the "device" is the **laptop-as-edge**; the phone is the touchscreen, and "airplane mode" is a **simulated** egress toggle — not the literal radios-off proof. Why, and what we traded, is recorded in **[ADR-015](files/adr-015-airplane-mode-simulated-in-web-demo.md)**.

**The vision — next.** The same core on a **native iPhone** (mlx-swift), where airplane mode is the *literal, tamper-evident* proof: radios physically off, nothing can transmit. The architecture is built for exactly this — the scrub logic is one portable Rust core and the phone is just another shell ([issue #9](../../issues/9)).

The demo *is* the trust boundary, made watchable. Its architecture *is* the pitch: the sensitive workload is **one portable, owned core**; everything platform-specific is a swappable adapter — the literal shape of pulling a workload off the datacenter and repatriating it to the edge.

---

## Reproduce the numbers (the trust mechanism)

Belief shouldn't come from a pitch — it comes from `./run.sh eval` printing the same numbers on **your** laptop that we claim on ours, against the golden notes committed in this repo.

```bash
git clone https://github.com/ChaiWithJai/airplane-mode && cd airplane-mode

# 1. get + serve the model (one-time; ~5 min). See docs/model-setup.md for details.
./scripts/serve-model.sh            # downloads Ternary-Bonsai-1.7B (Apache-2.0) + serves on :8080

# 2. reproduce the de-identification numbers through the owned Rust core
./run.sh eval
```

You should see **rules ∪ Bonsai-1.7B → 100% recall / 0 leakage** on the 20-note synthetic golden set — the same `airplane-core` the demo runs. (Recall and leakage reproduce; exact over-redaction counts may vary slightly across machines — tracked in [issue #12](../../issues/12).) Full setup, including the model footguns, is in **[docs/model-setup.md](docs/model-setup.md)**.

> Needs: a Mac (or Linux), the Rust toolchain, and `llama.cpp` (`brew install llama.cpp`). The model is **free under Apache 2.0** from `prism-ml/` on Hugging Face.

---

## Run the phone demo (Beat 1)

```bash
./scripts/serve-model.sh    # the model (terminal 1)
./run.sh web                # the UI (terminal 2) — prints a LAN URL
```

Open the printed `http://<laptop-ip>:8088` on a phone on the same Wi-Fi (or your iPhone hotspot — see the runbook). Dictate → **Scrub on device** → the name/ID/date/relationship get caught → the gate clears → the de-identified card **posts to Slack**. The full runbook, the Wi-Fi-isolation fix, and the 2-minute Slack-webhook setup are in **[docs/demo/onboarding.md](docs/demo/onboarding.md)**.

---

## Make it yours — five files, no fork (extendability)

The hard, correctness-critical core is built and gated **once**. Everything specific to *your* practice or vertical is a **pack** — five declarative files, **no code**:

```
packs/coach-session/
├── recognizers/   your identifiers (member IDs, partner orgs)
├── schema.yaml    your record shape
├── policy.yaml    what to redact · recall threshold · the reward rules
├── sink.yaml      where clean records go (credential sourced, never stored)
└── eval/          your golden notes — proves it doesn't leak
```

Copy the reference pack, edit five files for your identifiers, run the eval gate (it **must pass** before it ships), and you're running the identical signed core with your data. A pack can't see raw input, the redaction map, or the gate — so it's safe to write and safe to share. **Walkthrough: [docs/extending.md](docs/extending.md).**

---

## The architecture (one core, many shells)

```
        airplane-core  (Rust · portable · the "repatriated workload")
        rules executor · verifier gate · pipeline · pack loader
        depends only on PORTS:  InferenceProvider · SecureStore · Capture · Sink
                 ▲                      ▲                       ▲
         web shell (live)          CLI shell             MCP · iOS (planned)
        browser · laptop-edge  llama-server · file      same core, more reach
        "Beat 1, on any phone"  "numbers reproduce"     "an agent / a device too"
```

The **identical** recall-critical logic runs across every shell — which is what makes a reproduced number meaningful and the repatriation real, not asserted. The model is an `InferenceProvider` **port**, not baked in ([ADR-014](files/adr-014-portable-rust-core.md)). Today only `InferenceProvider` is wired; `SecureStore`/`Capture`/`Sink` are the contract the native on-device shell will fulfill.

---

## Honest about the model

We don't overclaim Bonsai: PrismML's "intelligence density" is a **self-coined** metric that loses on raw benchmarks; "1-bit" is **sign-only weights with grouped scale factors**; frontier cloud still wins peak quality. The bet isn't that this beats GPT — it's that **the most sensitive work should run where the data lives**, and a 1-bit model makes that possible on hardware you already own.

---

## How this was built · what's next

Built as a **harnessed loop** (`AGENTS.md` + `backlog/` + `gates/`), reproducible end-to-end. The design canon lives in `files/` and is indexed by **[CANON.md](CANON.md)**; the architecture decisions are ADR-001…015. The remaining work — Beat 2 (the five-file reveal), the two ethical gates, the follow-up loop, the native iOS shell — is tracked in the **[issues](../../issues)**.

| Path | What |
|---|---|
| `crates/airplane-core/` | the portable Rust trust core (rules · gate · pipeline · pack loader) |
| `shells/web/` · `shells/cli/` | the live Beat 1 demo · the reproduction front door |
| `packs/coach-session/` | the reference pack + 20 golden notes |
| `eval/golden-run.txt` | the committed reproduction target |
| `docs/` | model setup · phone runbook · extending guide · architecture spec |
| `files/` · `CANON.md` | the design canon (RFCs, ADRs) and its index |
| `run.sh` | one entrypoint: `eval · scrub · gates · web` |

*Built to make intelligence come to the data — not the other way around.*
