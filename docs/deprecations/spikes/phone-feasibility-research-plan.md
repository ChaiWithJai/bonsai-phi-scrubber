# Phone Inference Feasibility Research Plan

This is the research we should do before writing a real WebGPU or MLX spike.
The goal is to decide what to measure first, not to win an argument by taste.

## Question

Can Airplane Mode move the sensitive inference path from the Mac edge node onto
Jai's iPhone without weakening the trust boundary?

The trust invariant is unchanged:

```text
raw note -> local inference -> parse / validate / clamp -> verifier gate -> clean egress
```

The runtime can change. The gate cannot.

## What We Already Know

From the repo:

- The web demo is live and uses the Mac as the current edge node.
- The core correctness path is `llama-server` plus `airplane-core`.
- The iOS scaffold is simulator-only and has a clean inference seam:
  `TextInferenceProviding.complete(...)`.
- The shared backend contract exists under `docs/contracts/`.
- The current user-facing promise is scrubbed/redacted, not legally conclusive
  de-identification and not HIPAA compliance.

From current ecosystem evidence:

- WebLLM runs LLM inference in the browser with WebGPU and an OpenAI-compatible
  API surface.
- Transformers.js supports WebGPU-backed browser inference.
- Hugging Face hosts a community Bonsai WebGPU demo that loads
  `onnx-community/Bonsai-1.7B-ONNX` through `@huggingface/transformers` with
  `device: "webgpu"` and `dtype: "q1"`.
- ONNX Runtime Web has a WebGPU execution provider.
- Safari 26 / iOS 26 adds WebGPU support.
- MLX and MLX Swift are the Apple-native path, and MLX Swift examples include
  language-model apps.
- PrismML publishes Bonsai MLX text artifacts, but that does not by itself prove
  our exact model, prompt shape, structured output, and device target.

## First Phone URL To Try

Open this directly on the iPhone:

<https://huggingface.co/spaces/webml-community/bonsai-webgpu>

Record:

- iPhone model;
- iOS version;
- Safari version;
- whether the page loads;
- whether it reports WebGPU unsupported;
- model load progress and total download size if shown;
- whether generation starts;
- tokens/sec if shown;
- whether the tab survives backgrounding and reload;
- whether a second load is warm from cache.

What this proves if it works:

- the phone/browser can expose WebGPU;
- Transformers.js can execute a Bonsai-family ONNX artifact in-browser;
- Q1 WebGPU execution is plausible on this device class.

What it does **not** prove:

- our PHI scrub workflow runs locally;
- structured span JSON can be forced reliably;
- the verifier gate is wired in-browser;
- the model can run offline after first load;
- native iPhone / MLX Swift viability;
- Secure Enclave or Keychain storage.

## Means, Motivation, Opportunity

Use this frame to avoid confusing desire with feasibility.

### Means

Means are the practical resources we can use.

For Airplane Mode:

- Existing Rust trust core and eval harness.
- Existing web shell and phone UX.
- Existing Swift simulator seam for `TextInferenceProviding.complete(...)`.
- Existing backend response schema and fixture.
- Jai's iPhone as the actual measurement device.
- Public ecosystem runtimes: WebLLM, Transformers.js, ONNX Runtime Web, MLX
  Swift, and PrismML's model artifacts.

Means test:

- Can the runtime load any model on this phone?
- Can it produce a raw string from a synthetic prompt?
- Can that string be made schema-shaped enough for the core to parse and clamp?
- Can the page/app stay responsive?
- Can we cache or bundle the model without creating a worse demo burden?

### Motivation

Motivation is why the project should spend effort here.

For Airplane Mode:

- End-user value: the strongest version of the demo is "sensitive text is
  processed on the phone before it reaches shared tools."
- Builder value: a measured phone path is the missing contribution target for
  the edge-inference audience.
- PrismML value: Bonsai becomes a reference workflow, not just a model release.
- Demo value: phone-local inference tightens the story and reduces reliance on
  the Mac-as-edge explanation.

Motivation test:

- Does this improve the actual adoption story, or only the technical prestige?
- Does the result become easier for a healthcare hackathon participant to run?
- Does it produce a number PrismML, builders, and cautious adopters can respect?

### Opportunity

Opportunity is the external opening that makes the work timely.

For Airplane Mode:

- Safari/iOS WebGPU support makes browser-local inference newly plausible.
- WebLLM gives a browser LLM API shape close to the current OpenAI-compatible
  `llama-server` path.
- MLX Swift gives a native Apple path aligned with PrismML's positioning.
- The unwired Bonsai text path is a public, useful contribution target.
- The repo already has a trust harness, so runtime work can focus on inference
  instead of inventing safety logic.

Opportunity test:

- Is there an ecosystem path that does most of the runtime work for us?
- Can we make Bonsai fit that path without custom kernel work?
- If not Bonsai first, can a smaller model validate the runtime path cheaply?
- Which path yields a credible case study faster?

## Option 1: Browser WebGPU

### Research Before Spike

1. **Device capability**
   - Confirm iPhone model, iOS version, Safari version.
   - Open the Hugging Face Bonsai WebGPU Space:
     <https://huggingface.co/spaces/webml-community/bonsai-webgpu>.
   - Open a WebGPU report page and record `navigator.gpu`, adapter name if
     exposed, supported limits, and `shader-f16` support.
   - Check whether WebGPU works in Safari only or also installed iOS browser
     shells.

2. **Runtime fit**
   - Compare WebLLM, Transformers.js, and ONNX Runtime Web against our required
     method: `complete(messages, json_schema, sampling) -> String`.
   - Prefer the runtime with the smallest adapter surface.
   - Identify whether the runtime supports JSON mode, grammar, constrained
     decoding, or only raw generation.

3. **Artifact fit**
   - Determine whether Bonsai's available artifacts can run directly:
     - GGUF is not a browser format by default.
     - MLX is not a browser format.
     - ONNX would require conversion and validation.
     - WebLLM usually expects MLC-compiled model artifacts.
   - If direct Bonsai is not feasible, choose a small browser-native model first
     to validate runtime feasibility separately from Bonsai conversion.

4. **Data and storage fit**
   - Estimate model download size.
   - Verify caching behavior: first load, reload, offline reload, storage quota,
     and eviction risk.
   - Confirm no raw note is logged to network, analytics, or service worker
     caches.

5. **Safety fit**
   - Confirm output still returns as untrusted text.
   - Keep verifier gate host-side in the web app.
   - Test failure modes: invalid JSON, partial generation, timeout, tab
     backgrounding, page reload mid-run.

### Likely Advantages

- Fastest install-free way to measure phone-local inference.
- Reuses the current web shell and demo flow.
- Strong hackathon ergonomics: open a URL, no TestFlight.
- WebLLM's OpenAI-compatible shape may minimize adapter friction.

### Likely Risks

- iPhone 11 may not have iOS 26 / WebGPU support.
- Browser memory and storage limits may block Bonsai-sized artifacts.
- Bonsai artifact conversion may become the real work.
- Browser cannot give the same Secure Enclave / Keychain story as native iOS.
- Offline proof is weaker because browser cache behavior is harder to control.

### WebGPU Kill Criteria

Stop before full spike if:

- the target iPhone cannot expose WebGPU;
- no candidate runtime can generate text on-device;
- model artifact conversion requires custom compiler/runtime work beyond the
  demo scope;
- cold start or storage makes the healthcare hackathon path worse than the Mac
  edge node.

## Option 2: Native iOS + MLX Swift

### Research Before Spike

1. **Device and OS fit**
   - Confirm exact iPhone model, RAM class, iOS version, and Xcode deployment
     target.
   - Confirm whether MLX Swift examples build and run on that device.

2. **Runtime fit**
   - Inspect MLX Swift language-model examples and `mlx-swift-lm`.
   - Identify the minimum code path for prompt -> generated string.
   - Verify whether model loading can be isolated behind
     `TextInferenceProviding.complete(...)`.

3. **Artifact fit**
   - Confirm the exact Bonsai MLX text repo and weight format.
   - Confirm tokenizer format.
   - Confirm whether `mlx-swift-lm` can load the architecture without custom
     model code.
   - If not, estimate the model adapter work before touching app UI.

4. **Structured output fit**
   - Determine whether MLX Swift supports constrained decoding.
   - If not, define validate-and-retry limits and failure semantics.
   - Keep the core assumption: model text is untrusted until parsed, validated,
     clamped, and gated.

5. **App and storage fit**
   - Confirm signing/deploy path to Jai's phone.
   - Confirm model bundling vs download.
   - Define where raw note and redaction map live: Keychain/Secure Enclave where
     appropriate, never logs.

### Likely Advantages

- Best long-term fit for the "on-device iPhone" proof.
- Strongest path to Keychain/Secure Enclave storage.
- Cleaner PrismML / Apple / MLX story.
- Stronger future radio-off and app lifecycle claims.

### Likely Risks

- More setup friction before the first measurement.
- Text architecture support may require custom model code.
- Structured decoding is ours to implement or approximate.
- iPhone 11/A13 performance may not be acceptable even if it loads.

### MLX Kill Criteria

Stop before full spike if:

- MLX Swift examples cannot run on the target phone;
- Bonsai MLX text architecture is not supported without substantial runtime
  work;
- memory pressure prevents loading the smallest target artifact;
- signing/deployment blocks iteration during the demo window.

## Worked Example: Airplane Mode Feasibility Case Study

### Scenario

We want the phone to scrub this synthetic note locally:

```text
Jordan Lee, member COACH-4821, met with coach Maya on March 12. Jordan wants to
practice a five minute breathing routine before Monday standup.
```

The expected local inference result is not the final record. It is a raw,
untrusted JSON-ish span proposal:

```json
{
  "spans": [
    { "text": "Jordan Lee", "entity": "PERSON" },
    { "text": "Maya", "entity": "PERSON" },
    { "text": "COACH-4821", "entity": "MEMBER_ID" },
    { "text": "March 12", "entity": "DATE" }
  ]
}
```

The host then applies replacements, runs the verifier, and only then allows a
scrubbed record to leave.

### Means In This Case

We already have:

- a sample note;
- expected span shape;
- DTOs for `scrubbed_text`, `redactions`, `gate_pass`, `residual_count`, and
  `record`;
- simulator tests proving a raw span provider can drive the scrub backend;
- web UI flow for capture -> scrub -> gate -> send.

The research job is not "build the product." It is to answer the smallest hard
questions:

- Can this phone run any local text-generation runtime?
- Can it return the span JSON shape?
- Can it do so under a tolerable latency budget?
- Can it keep the raw note off the network?

### Motivation In This Case

If WebGPU works:

- the demo becomes easier to distribute;
- healthcare hackathon users can try phone-local inference without installing an
  app;
- the repo becomes a stronger web reference architecture.

If MLX works:

- the demo gets a stronger native iPhone proof;
- the PrismML Apple story becomes more concrete;
- future secure storage and airplane-mode proof become credible.

If neither works on the target phone:

- the Mac edge-node story remains honest;
- the repo still has a useful reference architecture;
- the phone-local claim stays out of the README until measured.

### Opportunity In This Case

WebGPU is timely because current browser ML frameworks have converged around it.
MLX is timely because PrismML's Bonsai artifacts include MLX formats and Apple
hardware is the natural edge story.

The opportunity is not to pick one permanently. It is to make Airplane Mode the
place where both paths are compared under the same trust harness.

## Research Matrix

| Question | WebGPU Evidence To Gather | MLX Evidence To Gather | Decision Impact |
| --- | --- | --- | --- |
| Can the phone expose the runtime? | `navigator.gpu`, adapter, limits, f16 | MLX Swift example app runs | Go/no-go for spike |
| Can we run any model? | smallest WebLLM / Transformers.js model | LLMBasic / LLMEval prompt works | Runtime viability |
| Can Bonsai fit? | MLC/ONNX/browser artifact path exists | Bonsai MLX architecture loads | Bonsai-specific viability |
| Can output fit the gate? | raw JSON spans or retryable text | raw JSON spans or retryable text | Trust compatibility |
| Is latency demoable? | cold load, warm load, tokens/sec | load time, memory, tokens/sec | Demo story |
| Is local privacy credible? | no network during inference after load | no network during inference | Claim boundary |
| Is storage acceptable? | cache quota/offline reload | bundle/download + local files | Hackathon ergonomics |
| Does it improve adoption? | no-install phone run | stronger native proof | Positioning |

## Recommended Research Order

1. **Desk-check device support.**
   Confirm the exact iPhone and OS. If it is not on iOS 26 or cannot expose
   WebGPU, WebGPU drops behind MLX for this device.

2. **Desk-check artifact path.**
   Identify whether Bonsai can run in WebLLM/Transformers.js/ONNX Runtime Web
   without deep compiler work. In parallel, identify whether Bonsai MLX text can
   load through `mlx-swift-lm`.

3. **Run zero-model capability probes.**
   Browser: WebGPU adapter and limits. Native: MLX Swift example launch.

4. **Run tiny-model probes.**
   Browser: smallest known-good text model. Native: known-good MLX Swift LLM
   example.

5. **Only then attempt Bonsai.**
   If the runtime cannot pass the tiny-model probe, Bonsai failure will not teach
   enough.

6. **Compare with the same scorecard.**
   Use latency, memory, model size, offline behavior, output shape, and trust
   compatibility. Do not compare vibes.

## Decision Rule

Choose WebGPU first if:

- the target iPhone supports WebGPU;
- a known-good browser LLM runs acceptably;
- Bonsai conversion looks tractable;
- install-free distribution matters more than native storage guarantees.

Choose MLX first if:

- the target iPhone lacks reliable WebGPU;
- MLX Swift examples run cleanly;
- Bonsai MLX text loads with low adapter cost;
- native storage and the PrismML Apple story matter more than web distribution.

Keep the Mac edge-node path if:

- both phone-local paths fail the latency or artifact-fit checks;
- phone-local setup would make the hackathon demo harder to reproduce;
- the measured result would force overclaiming.

## Sources

- WebLLM: <https://webllm.mlc.ai/docs/>
- WebLLM GitHub: <https://github.com/mlc-ai/web-llm>
- Transformers.js WebGPU guide: <https://huggingface.co/docs/transformers.js/en/guides/webgpu>
- ONNX Runtime WebGPU execution provider: <https://onnxruntime.ai/docs/tutorials/web/ep-webgpu.html>
- Safari 26 WebGPU announcement: <https://webkit.org/blog/16993/news-from-wwdc25-web-technology-coming-this-fall-in-safari-26-beta/>
- Safari 26 release notes: <https://developer.apple.com/documentation/safari-release-notes/safari-26-release-notes>
- MLX Swift: <https://github.com/ml-explore/mlx-swift>
- MLX Swift examples: <https://github.com/ml-explore/mlx-swift-examples>
- PrismML Bonsai collection: <https://huggingface.co/collections/prism-ml/bonsai>
