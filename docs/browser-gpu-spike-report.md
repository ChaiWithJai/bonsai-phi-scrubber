# Browser GPU Spike Report

Status: spike in progress, demo path narrowed.

## Executive Decision

Use the **browser GPU path first** for the next phone-local inference attempt,
but keep the sensitive workflow on a **first-party network**: phone to laptop,
hotspot, LAN, or IT-managed VPN. Keep native MLX Swift as a later proof path,
not the critical path for the demo.

Why: Jai validated the Hugging Face Bonsai WebGPU demo on his phone. That changes
the means/opportunity calculation. The ecosystem already has a browser path for
Bonsai-family inference; our job is now to adapt the trust harness around that
path, not to build a native iOS stack before proving the browser route fails.

This is a simplification, not a retreat:

```text
phone browser -> first-party network -> browser GPU/local edge -> parse/clamp -> verifier gate -> Slack
```

The trust invariant remains unchanged:

```text
raw note -> local inference -> host-side validation -> default-deny gate -> clean egress
```

## Hubris We Are Retiring

We overbuilt the next move by assuming the important fork was native iOS versus
Mac edge. That made the path look like:

```text
Swift app -> MLX Swift -> model loading -> constrained decoding -> UniFFI ->
Keychain -> radio proof -> app signing -> device measurement
```

That path is valuable, but it is not the fastest route to the demo standard we
actually set:

- show the sensitive workflow moving toward the edge;
- use Bonsai where it matters;
- keep egress default-deny;
- post only a scrubbed record to Slack;
- give adopters a low-friction way to reproduce and extend the pattern;
- give builders a clean contribution target.

The browser ecosystem already built much of the runtime surface. Ignoring that
would be expensive pride.

## Means, Motivation, Opportunity

### Means

What we now have:

- A live web shell on the phone.
- A working Slack sink with webhook route to `#coach-records`.
- A local verifier-gated Rust core.
- A browser capability endpoint: `POST /api/client-capability`.
- A phone-visible capability panel in the demo UI.
- A selectable web backend path: `Mac edge` or `Browser GPU`.
- A validated external Bonsai WebGPU demo:
  <https://huggingface.co/spaces/webml-community/bonsai-webgpu>.
- Documentation that the HF demo uses `onnx-community/Bonsai-1.7B-ONNX` through
  Transformers.js/WebGPU with `dtype: "q1"`.

What we do not have yet:

- Local browser inference inside our web shell. The UI can select `Browser GPU`,
  but it honestly records the gap and falls back to Mac-edge Bonsai until the
  adapter is wired.
- A structured span contract running in browser GPU.
- Offline model caching proof.
- A measured phone-local scrub on our synthetic coaching note.

### Motivation

For the end user, the browser path lowers adoption friction. A healthcare
hackathon team can open a URL, run a workflow, and understand the safety boundary
without installing an iOS app.

For the ecosystem, the browser path creates a cleaner case study: Bonsai can move
into an existing web workflow, with the verifier gate preserving trust. Builders
can still contribute MLX later, but the first win is where the runtime already
works.

For PrismML, this demonstrates "intelligence comes to the data" with less
platform ceremony. It is a density story: useful intelligence per unit of
friction, not just per byte or watt.

### Opportunity

The opportunity is immediate because:

- WebGPU is available in the modern Safari path Jai tested.
- Hugging Face already hosts a Bonsai WebGPU demo.
- Transformers.js already exposes the model loading/generation surface.
- Our app already has the web UX and Slack egress path.

The next optimal move is to wrap that browser model path in our trust harness.
The network move is equally important: no third-party tunnel for notes. Secure
context should come from local HTTPS or an IT-managed private network.

## Density Of Intelligence, Dogfooded

PrismML's density framing asks whether a model delivers useful intelligence per
unit of cost, energy, and deployment burden.

We should apply the same standard to our own engineering choices:

| Path | Intelligence gained | Burden added | Density judgment |
| --- | --- | --- | --- |
| Native MLX first | Strong native proof if it works | app signing, MLX text adapter, structured decoding, Keychain, device measurement | high eventual value, high friction |
| Browser GPU first | Phone-local runtime evidence now | browser model adapter, cache/offline measurement, HTTPS/local constraints | best next density |
| Mac edge only | working demo today | requires explaining laptop-as-edge | useful baseline, weaker phone-local proof |

The dense move is browser GPU first. It gives us the most validated learning per
unit of engineering effort, as long as we keep the network sovereign.

## Spike Evidence So Far

### External Phone Capability

Jai validated this URL on his phone:

<https://huggingface.co/spaces/webml-community/bonsai-webgpu>

Interpretation:

- WebGPU/Bonsai-family browser inference is plausible on the phone.
- The ecosystem path exists.
- This does not yet prove Airplane Mode's scrub/gate workflow in-browser.

### Local Demo Capability Endpoint

Implemented:

- `POST /api/client-capability`
- `/api/status` now includes `client_capability`
- phone UI shows a Browser inference probe card
- phone UI can select `Mac edge` or `Browser GPU`
- selecting `Browser GPU` currently fails closed to Mac-edge Bonsai and displays
  that truth in the scrub/record states
- UI links to the HF Bonsai WebGPU Space

Note: local LAN HTTP may not be a secure context on iPhone Safari. A phone can
therefore show WebGPU unavailable in the local UI while the HTTPS Hugging Face
Space works. That is not a contradiction; it is a deployment constraint. Because
we cannot assume a Cloudflare BAA, the project path is local HTTPS, not a public
tunnel for the scrub workflow.

### Endpoint Profile On Current Demo

Profiled on the current Mac-edge path, using the same endpoints the phone UI
calls:

| Step | Result |
| --- | --- |
| `/api/status` | model reachable, Slack configured via webhook |
| `/api/scrub` | 16.53s, `gate_pass: true`, `residual_count: 0`, 4 redactions |
| `/api/send` | 0.26s, Slack accepted |
| `/api/trajectory` | 0.03s, gate-clean trajectory stored |
| `./run.sh slack-smoke` | accepted through app-originated gated send |

After adding the selectable `Browser GPU` backend, the current honest fallback
path was profiled again:

| Step | Result |
| --- | --- |
| `/api/scrub` with `backend: browser-gpu` | 13.60s, falls back to Mac-edge Bonsai, `gate_pass: true`, `residual_count: 0`, 4 redactions |
| `/api/send` | 0.27s, Slack accepted |
| `/api/trajectory` | 0.03s, gate-clean trajectory stored as `local-000006` |

The scrub result caught:

- `PERSON` via Bonsai
- `MEMBER_ID` via rules
- `DATE` via Bonsai
- `FAMILY_DETAIL` via Bonsai

This proves the existing egress loop still works while we move inference closer
to the phone.

### Device Observation

macOS sees the paired phone:

```text
Lakshmi, iPhone12,3, available (paired)
```

Remote Safari launch through `devicectl` was blocked because Developer Mode is
disabled. That does not block the demo. It only means phone observation must come
through server telemetry unless Developer Mode is enabled.

## Optimal Path From Here

### Step 1: Keep The Current Demo Stable

The current Mac-edge path remains the fallback demo. It already posts to Slack
and has the verifier gate.

Do not break it while building browser GPU.

### Step 2: Replace Browser GPU Fallback With A Real Adapter

The selectable backend exists:

```text
Backend: Mac edge | Browser GPU spike
```

The next build slice is to replace the honest fallback with a real browser
adapter. Browser GPU mode should:

- load the Bonsai ONNX/WebGPU model in the browser;
- return raw span JSON proposals;
- send only span proposals and scrubbed candidates through the existing local
  gate path;
- keep the same `scrubbed_text`, `redactions`, `gate_pass`, `residual_count`,
  `record` response shape.

### Step 3: Solve HTTPS For Local Phone WebGPU

Because WebGPU may require a secure context:

- use the Hugging Face Space as external proof;
- test local HTTPS for our app with `./run.sh https-proxy`;
- do not tunnel `/api/scrub`, `/api/send`, or `/api/trajectory` through a public
  provider;
- document when local HTTP reports WebGPU unavailable even though the phone can
  run HF WebGPU over HTTPS.

### Step 4: Measure Browser GPU On The Same Note

Use the same synthetic note and scorecard:

- model load time;
- warm-cache load time;
- generation latency;
- span JSON validity;
- residual count after gate;
- Slack post success;
- tab background/reload behavior.

### Step 5: Reclassify Native MLX

Native MLX Swift remains valuable for:

- stronger native privacy proof;
- Keychain/Secure Enclave;
- future radio/offline claim;
- PrismML Apple-native contribution.

But it is no longer the fastest path to the next demo.

## Constitution Check

- **Default-deny egress:** unchanged. Slack and trajectory stay behind the gate.
- **Model is never trusted raw:** browser model output must be treated as raw
  text, parsed, validated, clamped, and re-scanned.
- **Packs stay declarative:** browser GPU cannot alter recognizers, policy, or
  verifier behavior.
- **Synthetic data only:** all spike notes stay synthetic.
- **No overclaiming:** HF WebGPU success means "browser path plausible," not
  "Airplane Mode is fully phone-local."

## Next Build Slice

Build a feature-flagged browser GPU backend mode that can run a tiny local
adapter against the same sample note. It should fail closed to Mac-edge mode if:

- WebGPU is unavailable;
- the model fails to load;
- generation times out;
- span JSON is invalid;
- the verifier finds residual identifiers.

The demo standard is not "the model runs." The demo standard is:

```text
phone-local inference proposes spans -> verifier proves clean -> Slack receives only scrubbed record
```
