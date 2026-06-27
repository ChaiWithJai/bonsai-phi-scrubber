# Jai Bhagat — First External Hire: Developer Relations at PrismML

*Presentation for Babak Hassibi, Sahin Lale, Omead Pooladzandi, and Reza Sadri*

---

## Part 1: The Work Already Done

> Open with the live repo, not this slide.

### What I shipped before we met

| Artifact | What it proves | Link |
|----------|---------------|------|
| **Bonsai PHI Scrubber** | A full reference architecture: synthetic note -> Bonsai scrub on first-party edge -> Rust verifier gate -> clean Slack egress. 21 synthetic notes, 100% recall, 0 leakage. Model-as-port, declarative packs, reproducible eval. | [repo](https://github.com/ChaiWithJai/bonsai-phi-scrubber) |
| **Bonsai Demo deep-wiki** | Source-validated founder arc and proof timeline covering Hassibi's OBS lineage (1992) through Bonsai Image 4B (May 2026). Every claim carries a Proved/Inferred/Needs-confirmation label. Cross-linked to whitepaper section. | [deep-wiki](https://github.com/ChaiWithJai/Bonsai-demo/tree/main/docs/deep-wiki) |
| **PrismML partner brief** | "Briefcase Technique" document: openings I see from outside, what I've built, 90-day plan, guardrails on claims, one ask. | [partner brief](https://github.com/ChaiWithJai/bonsai-phi-scrubber/blob/main/docs/positioning/prismml-partner-brief.md) |
| **Bonsai ecosystem plan** | Public note explaining who I am, why ecosystem, how I teach (Means/Motivation/Opportunity), and why I'm in. | [ecosystem plan](https://github.com/ChaiWithJai/bonsai-phi-scrubber/blob/main/docs/bonsai-ecosystem-plan.md) |
| **Loom demo walkthrough** | Publish-ready video showing the full scrub-gate-Slack flow. | [Loom](https://www.loom.com/share/bf03c0fcc09d40bd811af794d7b7481c) |
| **trydogfooding.com** | My product studio. Ship the thing, in public, turn the build into something other people can learn from. | [trydogfooding.com](https://trydogfooding.com) |
| **KVibe Studios** | 8,000 sq ft production space in Jersey City. Venue, content, and event infrastructure for community motion already exists. | Partnership with STG (Taso Z, Khoa Le) |

**The point:** I'm not pitching a plan. I'm showing work that already exists and
asking to do it *with* PrismML rather than *next to* PrismML.

---

## Part 2: Why DevRel Is PrismML's Highest-Leverage Hire Right Now

### The category logic

When the weights are Apache-2.0, the model is no longer the moat. **The scarce
asset is the ecosystem:** reference integrations, education, community, and proof
that real builders do real things with Bonsai locally.

Every comparable company has learned this:

| Company | Model strategy | DevRel / ecosystem layer | What it created |
|---------|---------------|--------------------------|-----------------|
| **Mistral** | Open-weight + commercial tiers | Sophia Yang as Head of DevRel; ambassador program; hackathon-led community; "la Plateforme" developer docs | Category-defining mindshare in European AI; 650K+ Discord members |
| **Ollama** | OSS wrapper around llama.cpp | Zero formal DevRel; community self-organized | 168K GitHub stars, 52M monthly downloads — but no ecosystem depth; "Docker for LLMs" without a developer journey |
| **HuggingFace** | Platform for all models | Decentralized DevRel ("Switzerland of ML"); no OKRs, no meetings; every engineer is DevRel | 200% community growth/year; default model distribution layer |
| **Together AI** | Open-model inference | Developer-first content; benchmark-led credibility; integration partnerships | Developer trust through transparency |

**PrismML's position is unique:** you have the model *and* the thesis
(intelligence density), the Caltech credibility, the Khosla/Cerberus validation,
and the edge deployment story. What you don't yet have is the **reference
integration layer** — the canonical projects that show builders "here's what you
build with Bonsai, and here's how." That's the DevRel gap, and it's the
highest-leverage gap to close.

### Why now, specifically

Three forcing functions make this the right quarter:

1. **llama.cpp merged Q1_0** — Bonsai's format is now native to the largest local
   inference ecosystem. Developers will discover it. The question is whether they
   find a reference project or a bare model card.
2. **Bonsai Image 4B + Bonsai Studio shipped** — PrismML now has an iOS app and
   an image model running on iPhones. The "text on phone" story needs a reference
   to match.
3. **The AI DevRel flywheel is time-sensitive** — early reference implementations
   get forked, and forks compound. The first canonical Bonsai project that shows a
   real workflow (not a benchmark) will define how the community thinks about
   the model.

---

## Part 3: The Natural Flywheels

### Flywheel 1: The AI DevRel Flywheel (industry standard)

```
Notebooks/Quickstarts
       |
       v
Open-Source Reference Implementations  <-- Bonsai PHI Scrubber is already here
       |
       v
Hackathons & Build Nights  <-- KVibe Studios is the venue
       |
       v
Content DevRel (technical writeups, demos, videos)
       |
       v
Integration Guild (LangChain, LlamaIndex, Vercel AI SDK, Ollama)
       |
       v
...feeds back to Notebooks
```

**Where PrismML is today:** Surface 0 (the model exists). I've already built
Surface 2 (reference implementation) independently. The 30/60/90 plan fills
the remaining surfaces.

**Compounding metric:** Fork velocity on reference repos. When >30% of new
Bonsai-related GitHub projects derive from a PrismML reference, the flywheel
is working.

### Flywheel 2: The Adoption Flywheel (PrismML-specific)

```
Intelligence density thesis (PrismML)
       |
       v
Reference integration proves it in a vertical (PHI scrubber, NPC dialogue, etc.)
       |
       v
Adopter journey report ("we ran this at a clinic with synthetic data")
       |
       v
Builder sees proof, forks, adapts to their vertical
       |
       v
New reference integration (community-generated)
       |
       v
PrismML's thesis gets proven in another domain
       |
       v
...compounds into "Bonsai is how you do edge AI"
```

**This is the HashiCorp pattern.** They didn't win by shouting. They won by
giving builders a reference for every problem and letting the community turn
references into adoption.

### Flywheel 3: The Cinematic Flywheel (orthogonal strategy)

This is what makes the plan worth documenting, not just executing.

```
Real builder ships a real thing with Bonsai
       |
       v
KVibe Studios films the build (documentary, not marketing)
       |
       v
Film earns attention from non-developer audiences
       |
       v
Non-developers discover PrismML through story, not specs
       |
       v
Story drives inbound from verticals PrismML hasn't targeted yet
       |
       v
New vertical = new reference = new film
```

**Why this works now (2026):**

Business Insider just profiled the rise of startup documentary filmmaking
(May 2026): "The documentary is just kind of like a higher-end version of
building in public." Offscript, a studio producing founder documentaries for
startups, reports 50K+ views on single pieces and 1M+ on launch videos.

**Why PrismML's story is uniquely filmable:**

- **30-year arc:** Hassibi's journey from OBS (1992) to Bonsai (2026) is a
  real narrative — not a pivot story, but a "the math finally met the hardware"
  story. That's rare and cinematic.
- **Caltech origin:** Labs, whiteboards, the physics of compression. Visual.
- **Edge as rebellion:** "Ship the compute to the data, not the data to the
  compute" is a counter-narrative to the datacenter arms race. That's a story
  audiences care about.
- **Real people, real workflows:** A coaching scribe protecting session notes.
  A community health worker with a phone and no cloud. These aren't hypothetical
  users — they're characters.

**I already have the production infrastructure:**

- **KVibe Studios** — 8,000 sq ft production space in Jersey City
- **STG partnership** — Taso Zafirakos ([IMDB](https://www.imdb.com/name/nm3073611/))
  and Khoa Le ([IMDB](https://www.imdb.com/name/nm3357359/)) at KVibe Studios
- **chaiwithjai.com** — cinematic documentaries and real use cases for AI

The cinematic strategy is **orthogonal** because it reaches audiences that
technical DevRel never will: healthcare administrators, investors evaluating edge
AI, policy people thinking about data sovereignty, and builders who learn
through story, not docs.

---

## Part 4: The 30/60/90 Day Plan

### Days 0-30: Establish the Reference

**Goal:** Bonsai PHI Scrubber becomes *the* canonical Bonsai edge-workflow
reference.

| Action | Metric | Already done? |
|--------|--------|---------------|
| Ship real `mlx-swift` Bonsai text adapter behind existing `TextInferenceProviding.complete(...)` | Adapter merged, honest measurement on oldest practical iPhone | Contract exists |
| Publish Bonsai PHI Scrubber as reference architecture (scrub -> gate -> clean egress) | First 10 forks | Repo is live |
| Write "How we made a 1-bit model useful inside a verifier-gated workflow" | Published, shared on HN/Reddit/X | Draft exists in repo docs |
| Open first contribution paths: MLX text adapter, structured output constraints, pack extensions | 3 good-first-issues filed, 1 external PR | Not yet |
| Map every place developers talk about Bonsai (GitHub, Discord, HN, Reddit, X) | Community map documented | Partially (Discord observed) |
| Have 15 conversations with developers using/evaluating Bonsai | Interview notes, top 5 pain points | Not yet |
| Complete the deep-wiki whitepaper analysis (the "crown jewel") | Published, cross-linked | Stub exists |

**Milestone:** The reference exists and is discoverable. Builders evaluating
on-device text find a real project, not just a model card.

### Days 30-60: Convene and Open the Funnel

**Goal:** Recurring community, early contributors, first adopter journey.

| Action | Metric |
|--------|--------|
| Host first NYC edge-inference build night at KVibe Studios | 20+ attendees, 5+ builders start projects |
| Turn repo into contributor surface: good-first issues, adapter notes, pack-extension docs, short demo video | Fork velocity >5/week |
| Start first adopter journey (synthetic data, healthcare coaching or benefits navigation vertical) | Journey in progress, documented |
| Run weekly office hours for builders porting the pattern to their runtime/vertical | 4 sessions held |
| Publish first cinematic piece: short documentary on the Bonsai edge story (KVibe x STG) | 10K+ views across platforms |
| Integration outreach: LangChain, LlamaIndex, Ollama model registry, Vercel AI SDK | 2 integration conversations started |
| Submit Bonsai PHI Scrubber as a reference at CNCF / AI Engineer meetups | 1 talk accepted |

**Milestone:** A recurring NYC room, early contributors, and the first adopter
journey in progress. The cinematic piece creates non-technical inbound.

### Days 60-90: Compound into Proof and Presence

**Goal:** Reference status. Builders cite the project. Adopters have a journey
report. PrismML has an ecosystem motion.

| Action | Metric |
|--------|--------|
| Publish first regulated-vertical journey report (synthetic data, explicit non-claims) | Report published, cited by 3+ external builders |
| Establish content cadence: biweekly technical writeup, monthly demo video, weekly office hours | 6 posts, 2 videos, 8 office hours |
| Anchor Bonsai presence at NYC Tech Week / AI Engineer World's Fair healthcare conversations | 1 keynote/panel, 1 booth or demo station |
| Package reference architecture for other verticals ("What do I build with Bonsai?") | 2 new pack templates (beyond healthcare) |
| Launch contributor recognition: highlight external contributors, feature community projects | 5 contributors recognized |
| Film long-form documentary: Hassibi's 30-year arc from OBS to Bonsai | Filming complete, edit in progress |
| Run second build night (larger, potentially co-sponsored with Caltech alumni NYC) | 40+ attendees |

**Milestone:** Builders cite the reference. Adopters point to a journey report.
PrismML has a concrete ecosystem motion both online and in NYC. The documentary
is in production.

---

## Part 5: Beyond 90 Days — The Compound Machine

### Quarter 2 (Months 4-6)

- **Scale the reference:** 3 vertical packs live (healthcare, education, legal),
  each with a community maintainer
- **Ambassador program:** Modeled on Mistral's — free API credits, feature
  preview, recognition, community leadership
- **Documentary release:** Feature the Caltech-to-product arc, the edge
  rebellion thesis, and real builders using Bonsai
- **Conference circuit:** AI Engineer World's Fair, CNCF events, Caltech
  symposia, Apple developer events (WWDC adjacent)
- **Integration milestones:** Listed in LangChain, LlamaIndex, and Ollama docs

### Quarter 3-4 (Months 7-12)

- **Developer certification:** "Bonsai Edge Developer" — practical, project-based
- **Bonsai Bootcamp:** Weekly cohort program modeled on Reza's AI Bootcamp
  methodology, adapted for external builders
- **Enterprise DevRel:** Reference architectures for regulated industries,
  co-developed with compliance partners
- **Hardware co-design community:** Bridge builders between PrismML's model team
  and silicon partners (Apple NPU, Qualcomm, etc.)
- **Annual Bonsai Summit:** Half-day event at KVibe, streamed, archived

---

## Part 6: How We'd Know It's Working

| Signal | 30 days | 60 days | 90 days | 6 months |
|--------|---------|---------|---------|----------|
| Reference repo forks | 10 | 30 | 75 | 200+ |
| External contributors (PRs merged) | 1 | 5 | 15 | 40+ |
| NYC build night attendance | -- | 20 | 40 | 60 (recurring) |
| Adopter journey reports | -- | 1 in progress | 1 published | 3 published |
| Integration listings | -- | 2 conversations | 2 live | 5 live |
| Content reach (views across platforms) | 5K | 25K | 75K | 250K+ |
| Cinematic content views | -- | 10K (short piece) | 25K | 100K+ (documentary) |
| Inbound from verticals PrismML hasn't targeted | -- | -- | 1 | 5+ |

The goal is not vague awareness. The goal is **reference status** — when someone
asks "how do I build with Bonsai?", they find our project first.

---

## Part 7: Why Me, Specifically

### The combination that's hard to hire for

| Dimension | Evidence |
|-----------|----------|
| **I build the reference myself** | Bonsai PHI Scrubber: Rust core, verifier gate, declarative packs, reproducible eval, iOS scaffold. Not a marketing site — a working architecture. |
| **I know the infrastructure audience** | Shipped Nomad UI at HashiCorp. I understand how infrastructure builders evaluate tools and what standard they apply. |
| **I teach** | Adjunct faculty. Started by coaching kids in my sister's driveway, now building a school. Turning people into a community of practice is the through-line, not a side quest. |
| **I make films** | KVibe Studios + STG partnership. The cinematic layer is not aspirational — the production infrastructure exists and the creative partners are named. |
| **I already understand PrismML's thesis deeply** | Source-validated proof timeline, ecosystem brief, positioning docs, and the partner brief in this repo demonstrate I've done the research and understand the guardrails. |
| **I'm honest about limits** | Not "on-device proven" until measured. Not "HIPAA-compliant." Not "intelligence density beats the field." Naming limits is what lets cautious adopters trust the work. |

### What I'm not

- I'm not a marketer who'll overclaim.
- I'm not a pure content person who can't build.
- I'm not someone who needs to be taught what PrismML does — I've already
  built a working system on top of it.

---

## Part 8: The One Ask

I'm building the edge-inference ecosystem around Bonsai either way: the
reference integration, the writeups, the NYC room, the films, and the adopter
path.

**I'd rather do it with PrismML than next to PrismML.**

The first step can be small: review the MLX text adapter boundary with me,
validate what PrismML wants represented accurately, and let the work earn the
next step.

---

## Appendix: Guardrails on Claims

These are non-negotiable. Credibility is the product.

- Not "on-device proven" until 1.7B text runs on a real measured device.
- Not "de-identified" or "HIPAA-compliant." The repo says "scrubbed," and so do I.
- Not "intelligence density beats the field" as a benchmark claim. I use it as
  framing for value migration and local workflow ownership.
- Not "healthcare production-ready." It's a synthetic, verifier-gated starter
  and reference architecture.
- Google is a compute-grant provider, not an investor or strategic partner
  (unless stated otherwise by PrismML).
- "1-bit" is sign-only weights with grouped scale factors. Frontier cloud still
  wins peak quality. The bet is situational, not absolute.

Naming these limits is not a weakness. It's what lets cautious adopters trust
the work and serious builders respect it.
