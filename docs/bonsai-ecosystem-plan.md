# I want to help build Bonsai's ecosystem

*An open note to the Bonsai community — and to the PrismML team — about the work, and the plan. I'm pitching myself for **Developer Relations**: building the community and the ecosystem around Bonsai.*

— Jai · [trydogfooding.com](https://trydogfooding.com) · [chaiwithjai.com](https://chaiwithjai.com)

---

## Who I am

I'm a product engineer who builds by **dogfooding**: I ship the thing, in public, and turn the build into something other people can use and learn from. I run a product studio, **[trydogfooding.com](https://trydogfooding.com)**, and I make **cinematic documentaries and real use cases for AI** at **[chaiwithjai.com](https://chaiwithjai.com)** — in partnership with **STG @ KVibe Studios** (Taso Z and Khoa Le). I'm also **adjunct faculty**, which matters here more than it sounds: how I teach is how I'd build this community.

This note shows the work, not just describes it. The first case study — an on-device PHI scrubber built on Bonsai — already lives in this repo ([README](../README.md), [how the demo works](demo/how-the-demo-works.md)).

## Why ecosystem, and why this way

The clearest ecosystem I've watched up close is **HashiCorp's**. They didn't win by shouting; they won by giving builders a *reference* for every problem — a clean architecture, real docs, a method you could copy — and then letting the community turn those references into adoption. The product was free to learn; the value was the path from "interesting tool" to "thing I run in production."

That's the play for an open-weight model company. When the weights are free — and Bonsai is, **Apache-2.0** — the scarce asset stops being the model and becomes the **ecosystem**: the reference integrations, the education, the proof that real people now do real things locally. I build that the only way I trust: dogfood it into a reference case, teach the method, and tell the story so the next builder shows up.

## How I teach — and would build: Means, Motivation, Opportunity

As faculty, the most useful thing I give students is a filter for turning *desire* into *feasible, relevant work*:

- **Means** — what can we actually measure or build *now*?
- **Motivation** — does it move the real adoption story, or only the technical prestige?
- **Opportunity** — is there a timing or ecosystem opening that makes the move land *now*?

It keeps people honest. It's the difference between "wouldn't it be cool if Bonsai ran on an FPGA" and "here's the measured CPU path that ships NPC dialogue in a game today."

And it fits Bonsai specifically: **Means/Motivation/Opportunity is the adoption-side mirror of Bonsai's own thesis.** PrismML's idea is *intelligence density* — the most useful intelligence per unit of size and power. My filter is the same instinct one layer down: the most *adoption* per unit of builder effort, on hardware people already own. Do more with less, where the data already is.

## Why I'm in, for real

I'm inspired by what Babak Hassibi said about the work: *"We spent years developing the mathematical theory required to compress a neural network without losing its reasoning capabilities. We see 1-bit not as an endpoint, but as a starting point."* That's a company that did the slow, hard thing and then **gave it away to start something bigger** — exactly the bet Vinod Khosla framed: the future "will be defined by who can deliver the most intelligence per unit of energy and cost." I want to build the ecosystem that proves that bet true in the world, not just in a whitepaper.

I'll also stay honest about the model, because honesty is the only DevRel currency that compounds: "intelligence density" is PrismML's *self-coined* metric, "1-bit" is sign-only weights, and frontier cloud still wins peak quality. The bet isn't "Bonsai beats GPT." It's "the most sensitive, latency-bound work should run where the data is" — and a 1-bit model finally makes that possible on a phone, a browser, an old laptop.

## The strategy opens with a benchmark — because you're a model company

A model company earns trust by being **measured honestly against its peers**, not by marketing. So the first thing I'd put in front of the community is a benchmark, and the obvious peer is **Liquid AI**.

Here's what we already found, in the community's own words: the read isn't "competitor," it's **ally**. One member working through it landed on *"my current thinking is that they are allies"*, and another is already itching to fuse the two worlds: *"lfm2.5 350m quantized to bonzai 1 bit would be craack."* Liquid's efficient-architecture LFMs and Bonsai's 1-bit density are two routes up the same mountain — getting capable intelligence off the datacenter and onto the edge. So the benchmark isn't a fight; it's an **honest, reproducible map of the efficient-model frontier**, on the axis PrismML actually owns: *intelligence density at the edge* — capability per MB, per watt, per token/sec, on hardware people have. Allies, measured in the open. That's a model-company-grade opening move, and the community will rally around it.

## Why healthcare for the first case study

I started with the hardest, most convincing vertical on purpose. **Airplane Mode** is a synthetic mental-health coaching scribe: it scrubs identifiers locally and lets only a clean record leave. Healthcare is where the value-migration thesis is *easiest to buy* — the data is sensitive, the privacy bar is real, and "the raw note never goes to a datacenter" is a sentence a non-engineer immediately understands. It also rhymes with what builders here already want, including a community ask to build a case study for *"hospitals… reducing patient discharges that lead to readmissions."*

And I made it a *real* case study, gaps included ([eval/golden-run.txt](../eval/golden-run.txt), [ADR-015](../files/adr-015-airplane-mode-simulated-in-web-demo.md)): rules alone caught **26%** of identifiers; rules ∪ Bonsai-1.7B reached **100% recall, 0 leakage**, reproducible with `./run.sh eval`. The lesson is the principle, not the score — a 1-bit model is **~80% recall and stochastic on its own**, useless as a privacy gate, but inside a deterministic harness (grammar-constrained output, a verifier that re-scans the exact outbound payload *outside* the model) it clears the bar. **Deploy density models as components inside harnesses, not as oracles.** Honest limit: today it runs on a laptop edge node, not phone-local. Naming that is what makes it trustworthy — to a clinician and to a builder alike.

## How I'd run the community loop

A **Discord → Twitter → newsletter** loop, with one channel as the engine.

- I put content like this case study and the Liquid benchmark *into* the community — a `#community-hardware-lab` channel — as the reference bar.
- I work *with* the community to fill out a **backend matrix**: the runtimes you actually want (llama.cpp, MLX, ONNX/WebGPU, Rust, your weird hardware), captured in a shared benchmark format and a parity ladder so contributions count without overclaiming.
- I collect your benchmarks and **hardware offers** (the AMD cards, the Strix Halo, the FPGA, the old M1s already on the table here) into a living map of *who can now do what, locally.*
- And I share **behind-the-scenes** — the spikes that failed, the gap we couldn't close yet — so you see the direction, not just the highlight reel. BTS is how a community learns to trust you.

Benchmarks tell us whether a workflow is practical; case studies tell us why anyone cares. I'd keep the second one mandatory.

## The promise: online, NYC, then the globe — cinematically

I won't just post threads. I'll **make it real and make it beautiful.**

- **Online:** the loop above, run consistently, in the open.
- **NYC:** **demos and live events** around the three verticals where this bet bites hardest — **healthcare, finance, and inference** — designed to pull the *ecosystem and the community into the same room.* NYC has the regulated-industry density (health systems, banks) and the builder scene to make that land. With KVibe Studios these won't be a meetup with a projector — they'll be **cinematic**: the kind of thing people clip, share, and show up for.
- **Global:** then I take it to where the adoption base actually is, and source speaking/demo slots at the stages that matter:
  - *Research credibility (the 1-bit lineage):* NeurIPS / ICLR efficient-ML & quantization tracks, MLSys.
  - *Systems & inference:* PyTorch Conference, MLSys, on-device/edge tracks.
  - *Edge / on-device:* tinyML Summit, Edge AI events, Apple/MLX dev venues.
  - *Ecosystem & DevRel:* AI Engineer World's Fair / Summit (SF + NYC), KubeCon + CloudNativeCon (the CNCF end-user case-study angle).
  - *The runtime community already building Bonsai:* RustConf and the Rust-AI meetups.
  - *Local NYC engine:* the AI/ML meetups, Cornell Tech, and the hackathon circuit — where I'd seed the demos and recruit the next case studies.

The throughline, everywhere: **who can now do what, locally, because Bonsai works?** Find the answer, prove it with logs, name the gaps, and tell the story cinematically — until the answer is "almost everyone, almost anywhere."

That's the plan. I've already started. I'd love to do it with you.

---

*References: founder quotes (Hassibi, Khosla) are attributed to PrismML's launch materials; "intelligence density" is PrismML's self-coined metric. The Liquid AI "ally" read and the LFM→1-bit quant idea are from the PrismML Discord (`#ideas-and-feedback`). The case-study numbers are reproducible in this repo — [the demo](../README.md), [eval/golden-run.txt](../eval/golden-run.txt), [ADR-015](../files/adr-015-airplane-mode-simulated-in-web-demo.md). Substantiation note: say "scrubbed/redacted," not "de-identified"; the demo is synthetic-only and runs on a laptop edge node.*
