# Deep Agent Pack Builder Research

Status: research memo  
Date: June 27, 2026  
Question: should we turn the PHI-scrubber lessons into a Deep Agents-powered
pack builder for healthcare/gluework users?

## Executive Read

Build a **Pack Builder Deep Agent**, not a "PHI agent."

The agent's job is to help an adopter describe a workflow, generate a declarative
pack, create synthetic evals, run gates, and write the business-case/runbook. The
Bonsai model should be consumed as a **specialized local inference tool behind the
verifier**, not as the primary planning model for the agent.

The strongest first use case is:

```text
healthcare workflow interview
-> draft pack
-> synthetic eval set
-> run gates
-> produce README + business-case memo
-> explain claim boundary
```

This reuses the pattern we learned the hard way: models are useful components
inside harnesses, not oracles.

## Citation Extraction

| Source | Extracted fact | Why it matters here |
| --- | --- | --- |
| HashiCorp RFC template | RFCs are for proposing a solution and getting feedback; changes affecting many stakeholders should use the RFC to drive clarity and consensus. Source: https://www.hashicorp.com/en/how-hashicorp-works/articles/rfc-template | Our Deep Agent work should start as an RFC because it affects end-user workflow, pack format, agent safety, and ecosystem contribution paths. |
| HashiCorp RFC template | Background should let a newcomer follow links and understand why the change is necessary; proposal explains the solution; abandoned ideas should be preserved with reasons. Source: same as above | We should include source evidence, context, and rejected approaches, not just a product idea. |
| HashiCorp RFC template | RFC implementation sections should explain API/package/surface area; UX sections should cover external API, config, CLI output, compatibility, and whether the change feels like the project. Source: same as above | The Pack Builder has to specify generated files, CLI, gates, and user journey, not only agent prompts. |
| Nomad Pack README | Nomad Pack is a templating and packaging tool for Nomad, used to deploy popular apps, reuse common patterns, and share job specs with the community. Source: https://github.com/hashicorp/nomad-pack/blob/main/README.md | This is the closest analogy: package a repeatable operational pattern so a user can configure it instead of inventing it. |
| Nomad Pack docs | A pack registry contains README, CHANGELOG, `packs/<PACK-NAME>`, metadata, variables, outputs, and templates; users can scaffold packs and deploy them. Source: https://developer.hashicorp.com/nomad/tutorials/archive/nomad-pack-writing-packs | Our pack registry should mirror this: metadata, variables, README, evals, policy, schema, sink, outputs. |
| Packer templates docs | Packer templates are configuration files that determine behavior, declaring which plugins to use, how to configure them, and in what order to run them. Source: https://developer.hashicorp.com/packer/docs/templates | Our generated pack should be an executable recipe for a privacy boundary, not prose. |
| Packer HCL block docs | Packer uses built-in block types such as `build`, `source`, `provisioner`, `post-processor`, `variable`, and `locals`; sensitive variables can be obfuscated. Source: https://developer.hashicorp.com/packer/docs/templates/hcl_templates/blocks | The analogy is a "workflow boundary build": sources are workflow inputs, provisioners are recognizers/model passes, post-processors are clean egress/runbooks. |
| Deep Agents JS overview | Deep Agents is an agent harness with built-in planning, file systems, subagent spawning, long-term memory, context management, tools, and HITL. Source: https://docs.langchain.com/oss/javascript/deepagents/overview | It supplies the harness primitives we kept rebuilding around this demo. |
| Deep Agents customization docs | `createDeepAgent` accepts model, tools, system prompt, middleware, interpreters, subagents, backends, HITL, skills, memory, profiles, and structured output. Source: https://docs.langchain.com/oss/javascript/deepagents/customization | The Pack Builder can be expressed cleanly as a configured harness plus domain tools. |
| Deep Agents overview | Virtual filesystem operations include `ls`, `read_file`, `write_file`, `edit_file`, `glob`, `grep`; `execute` is available with sandbox backends. Source: https://docs.langchain.com/oss/javascript/deepagents/overview | Pack authoring maps naturally to filesystem artifacts, while test execution should require sandbox/approval. |
| Deep Agents overview | Permissions can restrict filesystem read/write paths and protect sensitive files such as `.env` or credentials. Source: same as above | This matches our hard rule: packs must not see raw PHI or secrets. |
| Deep Agents overview | Skills are loaded progressively; memory is loaded as persistent project context. Source: same as above | Use skills for workflow templates; use memory for stable project constitution. |
| Deep Agents overview | Subagents isolate long-running subtasks and return compact final reports. Source: same as above | Use subagents for interview synthesis, eval design, claim critique, and docs generation without bloating the main context. |
| Deep Agents overview | Human-in-the-loop interrupts pause for approval on sensitive tool calls. Source: same as above | Use HITL before writing packs, running evals, touching sinks, or creating outbound claims. |
| Hamel/Shankar evals FAQ | A trace is the full record of actions, messages, tool calls, retrievals, and intermediate steps from query to final response. Source: https://hamel.dev/blog/posts/evals-faq/ | Pack Builder outputs must include trace/run evidence, not just generated files. |
| Hamel/Shankar evals FAQ | Minimum viable eval starts with manual error analysis over 20-50 outputs and domain-expert judgment; they recommend looking at failures rather than dashboards. Source: same as above | First version should generate reviewable synthetic traces and a small annotation loop before full automation. |
| This repo, commit `7c6b114` | Runtime path had to be explicit before scrub. | Pack Builder should force users to declare runtime/sink/policy before generating claims. |
| This repo, commit `61a123c` | Scrub latency needed heartbeat and expectation-setting. | Pack Builder should generate latency/run-observability sections automatically. |
| This repo, commit `6d3e301` | Phone HTTPS path needed observability and proof commands. | Pack Builder should create proof commands for every generated workflow. |
| This repo, commit `dda0346` | Browser GPU warmup crashed mobile Chrome and became benchmark-only. | Pack Builder must record environment assumptions and benchmark/deferred paths honestly. |
| This repo, commit `51973b9` | Publish orientation needed claim guardrails. | Pack Builder should produce publish-safe copy and "avoid saying" sections. |

## Delivery Patterns From Reference Projects

### Pattern 1: RFC Before Surface Area

HashiCorp's RFC pattern is not just documentation; it is a delivery control.
Before code, it captures background, proposal, implementation surface, UX impact,
abandoned ideas, and stakeholder clarity.

For us:

```text
deepagent-pack-builder-rfc.md
  summary
  background / evidence
  proposal
  user journeys
  generated artifact format
  safety / HITL
  abandoned ideas
  verification
```

Value: prevents agent enthusiasm from silently changing trust boundaries.

### Pattern 2: Pack As Reusable Operational Pattern

Nomad Pack packages reusable deployment patterns into registries. The user changes
variables and runs a known pack instead of writing a job spec from scratch.

For us:

```text
packs/care-navigation-intake/
  README.md
  metadata.yaml
  variables.yaml
  recognizers/
  schema/
  policy/
  sink/
  eval/
  outputs.md
```

Value: a community member contributes a regulated-workflow pattern; an adopter
configures it for their context without touching the Rust core or verifier.

### Pattern 3: Recipe With Ordered Stages

Packer makes image builds reproducible through ordered sources, builders,
provisioners, and post-processors.

For us:

```text
source: synthetic workflow description
recognize: rules + Bonsai proposals
verify: default-deny residual scan
structure: clean record schema
post-process: Slack payload + business memo
publish: screenshots + claim guardrails
```

Value: the pack is not just "config"; it is a repeatable build of a trust boundary.

### Pattern 4: Harness With Built-In Control Surfaces

Deep Agents brings the primitives we need:

- planning via todos;
- filesystem artifacts;
- subagents for isolated analysis;
- skills/memory for project constitution;
- permissions and HITL;
- sandbox execution when commands are needed.

Value: our community gets an agentic authoring environment without giving the
agent arbitrary access to PHI, secrets, or production sinks.

### Pattern 5: Eval/Trace Before Claim

The inference-engineering lesson is that evidence precedes claims. Hamel/Shankar's
FAQ pushes trace review, error analysis, and domain-expert judgment before generic
metrics. This mirrors our own lesson: the video was ready only after the repo had
latency, HTTPS, Slack, and claim-boundary evidence.

Value: every generated pack should include an evidence bundle:

```text
eval/golden.jsonl
eval/golden-run.txt
proof/scrub-response.sample.json
proof/status.sample.json
publish/claims.md
publish/screenshots.md
```

## Means, Motivation, Opportunity

### Means

We can build this without solving phone-local inference first.

Already available:

- Rust verifier core and pack structure.
- `./run.sh gates-fast` and full eval path.
- Slack sink contract and proof docs.
- Contract fixtures in `docs/contracts/`.
- Phone HTTPS observability pattern.
- Commit history containing actual incidents and mitigations.
- Deep Agents harness with tools, filesystem, subagents, permissions, HITL, and
  optional sandbox execution.

Missing but tractable:

- Pack schema metadata for generated packs.
- A `pack validate` command or wrapper.
- A small synthetic workflow interview format.
- A Deep Agent skill for pack authoring.
- A Bonsai scrubber tool wrapper that only returns scrubbed/gated outputs.

### Motivation

The same end user profile still applies: people doing healthcare gluework in
Sheets, Slack, intake forms, and EHR-adjacent workflows.

They do not wake up wanting a Deep Agent. They want:

- a credible hackathon demo;
- a safer way to move sensitive notes into team tools;
- a business-case memo for leadership;
- a template that avoids overclaiming HIPAA/de-identification;
- a way to adapt the workflow without becoming a Rust/model-runtime expert.

### Opportunity

The ecosystem gap is not "another model demo." It is a **packaging and evidence
loop** for density-model workflows:

```text
workflow -> pack -> eval -> proof -> story -> contribution
```

Nomad Pack solved reusable workload deployment. Packer solved reproducible image
creation. A PHI Pack Builder would solve reusable regulated-AI workflow boundaries.

## Deep Agent Product Shape

### Agent Name

`bonsai-pack-builder`

### Primary User Journey

```text
1. User describes a workflow.
2. Agent asks bounded questions.
3. Agent writes a draft pack in scratch state.
4. Eval subagent generates synthetic notes.
5. Claim critic subagent writes allowed/blocked claims.
6. Agent asks HITL approval before writing into repo.
7. Sandbox/tool runs pack validation and gates.
8. Agent produces README, business-case memo, and next steps.
```

### Tools

Minimum tools:

- `read_pack_schema`
- `draft_pack_file`
- `validate_pack`
- `run_gates_fast`
- `scrub_synthetic_note`
- `summarize_gate_result`
- `write_business_case`

Do not expose:

- raw Slack webhook secrets;
- unrestricted filesystem;
- arbitrary shell without sandbox;
- real PHI input;
- production sink send by default.

### Subagents

| Subagent | Job | Tools |
| --- | --- | --- |
| Workflow interviewer | Convert messy user workflow into bounded requirements | no filesystem write |
| Pack drafter | Create recognizers/schema/policy/sink/eval files | write scratch only |
| Eval designer | Generate synthetic eval cases and failure hypotheses | scratch + schema read |
| Claim critic | Check copy against allowed claims and evidence | read docs/proof only |
| GitOps reporter | Summarize diff, gates, and next branch/PR step | git read only |

### Memory And Skills

Memory:

- synthetic-only;
- say scrubbed/redacted, not de-identified;
- no HIPAA-compliant claim;
- verifier gate is non-negotiable;
- packs are declarative and code-free.

Skills:

- `pack-authoring`
- `healthcare-claim-guardrails`
- `eval-design-for-sensitive-workflows`
- `business-case-memo`
- `incident-to-proof-runbook`

### Backend

Recommended first build:

- `StateBackend` for scratch work;
- `CompositeBackend` later for `/memories/` if we want persistent pack-building
  preferences;
- sandbox backend only for validation/eval execution;
- HITL before `write_file`, `edit_file`, `execute`, or external sink tools.

## Worked Example: Care Navigation Intake

Input:

```text
We collect referral notes in Google Sheets, then paste a short summary into Slack
for the care navigation team. We need to remove names, dates, phone numbers,
member IDs, addresses, clinic names, and family details before Slack.
```

Agent output:

```text
packs/care-navigation-intake/
  README.md
  metadata.yaml
  variables.yaml
  recognizers/member-id.yaml
  recognizers/contact.yaml
  schema/record.schema.json
  policy/egress.yaml
  sink/slack.yaml
  eval/golden.jsonl
  publish/business-case.md
  publish/claims.md
```

Verification:

```bash
./run.sh gates-fast
./run.sh eval PACK=packs/care-navigation-intake
```

Adopter value:

- They can show a safe Slack workflow.
- They get synthetic evals for leadership.
- They know exactly what is and is not proven.

Builder value:

- They can improve recognizers, model ports, or eval coverage.
- They can contribute a pack without touching the trust core.
- They can benchmark Bonsai in a workflow, not as an abstract chatbot.

## Worked Example: Healthcare Hackathon Starter

Input:

```text
I have 48 hours at a healthcare hackathon. I want a credible AI demo for intake
notes without sending raw sensitive text to a cloud app.
```

Agent output:

- a starter pack;
- a demo script;
- screenshots checklist;
- Slack setup checklist;
- false-claim guardrails;
- "what to say to judges" memo.

Value:

- The end user gets a demo that is coherent and safe enough to explain.
- The ecosystem gets reproducible examples and more contributors.

## Worked Example: Internal Business Case

Input:

```text
I need to explain to my hospital leadership why this local-edge pattern deserves
a small pilot.
```

Agent output:

- one-page business case;
- architecture diagram;
- risk table;
- BAA/compliance caveats;
- proof commands;
- adoption plan.

Value:

- The adopter can ask for a pilot without overclaiming.
- The builder community sees where local inference must improve to become usable.

## What Not To Build First

Do not start with:

- a fully autonomous PHI-processing agent;
- real patient-data ingestion;
- production Slack posting from agent decisions;
- long-term memory containing workflow examples that might include PHI;
- Bonsai as the planner model unless tool-calling reliability is proven.

These violate the lessons from the current repo.

## Research-To-Spike Path

1. Write RFC: `docs/rfcs/0001-deepagent-pack-builder.md`.
2. Add static pack schema metadata.
3. Create a Deep Agents skill for pack authoring.
4. Implement a mock agent script that uses no external secrets.
5. Generate one care-navigation pack from synthetic requirements.
6. Run gates.
7. Compare output to hand-authored `coach-session` pack.
8. Capture failures as trace review, not as hidden prompt edits.

## Open Questions

- Should packs use YAML-only metadata or keep current folder-specific formats?
- Should generated evals live inside the pack or in a sibling `eval/` registry?
- Should the agent write directly to repo files or only produce a patch artifact?
- What is the minimum "pack registry" interface for ecosystem contribution?
- Which Deep Agents runtime should we prototype first: TypeScript for LangChain JS,
  or Python because the Python docs are more mature?
- Can Bonsai serve a low-cost local critic role, or should it remain only the
  scrub/density-model demonstration tool?

## Recommendation

Proceed to RFC, not implementation.

The RFC should state:

> We are building a Deep Agents-powered pack authoring harness that turns a
> regulated workflow description into a verifier-gated, synthetic-eval-backed pack.
> The agent may draft files and run validations, but it cannot weaken the verifier,
> process real PHI, or claim compliance beyond evidence.

This is the most ecosystem-aligned move because it creates a contribution loop:

```text
adopter pain -> pack -> eval -> proof -> publish -> community extension
```

That is the same flywheel as the current demo, but generalized.
