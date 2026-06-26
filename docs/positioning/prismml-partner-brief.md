# PrismML Partner Brief: Airplane Mode

Draft for a respectful founder/team conversation. This is not a press release and
not a demand. It is a clear offer: turn Airplane Mode into a useful reference
integration for Bonsai adopters who care about edge inference, privacy-sensitive
workflows, and auditable harnesses.

## Short Version

Airplane Mode is a reference architecture for running a sensitive workflow at the
edge with Bonsai, then proving that only a scrubbed record crosses the boundary.
It is deliberately small: a Rust trust core, a Bonsai inference port, a verifier
gate, a declarative pack, and thin shells for web, CLI, MCP, and iOS simulator
work.

The value to PrismML is not that this one healthcare demo becomes important on
its own. The value is that it shows application developers how to make Bonsai
useful in the kind of workflow they already own:

```text
raw workflow data -> local Bonsai-assisted scrub -> verifier gate -> clean egress
```

That is the adoption story Bonsai needs in regulated and privacy-sensitive
verticals: not "a smaller model exists," but "here is how to move a real workflow
off the datacenter without asking the model to be trusted raw."

## Why This Helps PrismML

PrismML already has a strong technical thesis: intelligence density, open weights,
edge deployment, and value migrating away from centralized inference. Airplane
Mode gives that thesis a concrete workflow shape.

It helps in four ways:

1. **A reference integration for application developers.**
   The repo shows how to wrap Bonsai in a deterministic harness: schema-shaped
   output, host-side parsing and clamping, recall-first redaction, and a
   default-deny verifier over the outbound payload.

2. **A credible edge-inference story for cautious adopters.**
   Healthcare, coaching, benefits navigation, and community-health teams do not
   buy benchmarks first. They need a trust boundary, a runbook, and a way to
   adapt the workflow without rewriting the core.

3. **A clean contribution path for the MLX text gap.**
   The repo now has an iOS simulator backend selector and backend-compatible DTOs.
   The next useful contribution is precise: replace the `mlx-swift` mock with a
   real text adapter, then measure on the oldest available iPhone before making
   claims about iPhone 11/A13 performance.

4. **A story builders can respect.**
   It names the real gaps. The current correctness path is laptop/llama-server.
   The iOS path is simulator choreography and interop scaffolding. The hardware
   proof is still a measurement gate. That honesty improves credibility with the
   Rust, llama.cpp, MLX, and harness-engineering communities.

## What I Would Do

I would position Airplane Mode as a Bonsai reference architecture and make it
useful for both sides of the adoption loop.

For end users:

- package the demo as a CNCF-style journey/runbook;
- keep the language inside the substantiation envelope: scrubbed/redacted, not
  legally conclusive "de-identified," and never "HIPAA-compliant";
- show how a team changes a pack for its own workflow without touching the core;
- create synthetic worked examples for intake notes, coaching recaps, benefits
  navigation, and referral routing.

For builders:

- document the `InferenceProvider` port as the stable adapter contract;
- make the `mlx-swift` text path a visible, well-scoped contribution;
- keep eval and gate output reproducible;
- publish the harness details: fixed-seed passes, schema constraints, output
  hygiene, verifier gate, pack-blindness, and ethical gates.

For PrismML:

- make Bonsai look practical in a workflow, not just impressive in isolation;
- give PrismML a concrete adoption artifact to point to when founders, app
  teams, or edge builders ask "what do I build with this?";
- feed back integration footguns respectfully, especially around MLX text,
  structured output, and stock-vs-fork runtime behavior;
- produce a small number of high-signal artifacts: reference architecture,
  healthcare hackathon starter, demo video/runbook, and contribution guide.

## What I Would Not Say

I would avoid framing that sounds like PrismML has a problem only we can fix.
Better framing:

- "This is a reference integration that makes the thesis concrete."
- "This gives application developers a pattern to copy."
- "This identifies a useful next adapter for the MLX text path."
- "This is a small, honest bridge between the model release and adoption in
  privacy-sensitive workflows."

Avoid:

- "PrismML lacks adoption proof."
- "The current messaging is wrong."
- "This demo proves iPhone 11 text inference."
- "This is HIPAA-ready."

## The Ask

The best first ask is modest:

> I am building Airplane Mode as an open reference architecture for Bonsai in
> privacy-sensitive edge workflows. I would value PrismML's review of the adapter
> contract and the MLX text path assumptions, especially where the simulator mock
> should give way to a real `mlx-swift` implementation and device measurement.

That ask gives the team something concrete to react to without demanding a
partnership, endorsement, or roadmap change.

## The Contribution Boundary

The project will stay honest about what is proven:

- Proven now: laptop edge path, llama-server integration, gates, Slack egress,
  pack extension, simulator backend-selection scaffold.
- Not proven yet: real `mlx-swift` text inference, iPhone 11/A13 throughput and
  memory, Secure Enclave storage, real radios-off proof.

The work becomes valuable precisely because it does not blur those boundaries.
For adopters, that reduces risk. For builders, it earns trust.
