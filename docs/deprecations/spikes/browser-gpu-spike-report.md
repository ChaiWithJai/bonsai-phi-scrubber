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
- A local module worker at `/bonsai-worker.js` that mirrors the Hugging Face
  Space's implementation: `@huggingface/transformers`, `device: "webgpu"`,
  `dtype: "q1"`, `onnx-community/Bonsai-1.7B-ONNX`.
- Browser model telemetry reported back through `/api/client-capability` and
  visible in `/api/status`.
- A verifier-gated browser span finalizer at `POST /api/browser-spans`.
- Browser GPU mode now attempts:
  `phone/browser q1 Bonsai -> span JSON -> /api/browser-spans -> gate -> record`.
- A validated external Bonsai WebGPU demo:
  <https://huggingface.co/spaces/webml-community/bonsai-webgpu>.
- Documentation that the HF demo uses `onnx-community/Bonsai-1.7B-ONNX` through
  Transformers.js/WebGPU with `dtype: "q1"`.
- A first-party browser runtime cache at `/vendor/transformers.js`.
- A first-party q1 Bonsai model cache at
  `/models/onnx-community/Bonsai-1.7B-ONNX/...`.

What we do not have yet:

- A measured successful local browser generation on Jai's phone from our own
  web shell.
- A measured browser-span finalization from Jai's phone through
  `/api/browser-spans`.
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

### Why This Is WebGPU, Not WebGL

Jai said "WebGL" as shorthand for "browser GPU." The implementation truth is
more specific: the working ecosystem path is **WebGPU**. The Hugging Face Bonsai
Space uses Transformers.js with `device: "webgpu"` and q1 ONNX weights. WebGL is
useful as a capability signal and legacy graphics/inference substrate, but it is
not the path the Bonsai browser demo has validated for this LLM workload.

So the product language can say "browser GPU" when speaking to end users. The
engineering language should say "WebGPU" when specifying the adapter.

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
- `./run.sh phone-observe` waits for the phone browser to report sanitized
  capability/model telemetry and writes `.airplane/phone-capability-latest.json`
- server-side capability telemetry includes `observed_from`, the remote LAN
  address that posted the report
- phone UI shows a Browser inference probe card
- phone UI can select `Mac edge` or `Browser GPU`
- selecting `Browser GPU` attempts a local q1 Bonsai span-generation probe with
  a bounded timeout, parses span JSON, and sends valid proposals to
  `/api/browser-spans`
- `/api/browser-spans` exact-matches browser spans against the note, unions them
  with deterministic rules, requires contextual browser evidence, redacts,
  verifier-gates, and returns the normal scrub response shape
- if browser generation, JSON parsing, span finalization, or the gate fails, the
  UI fails closed to Mac-edge Bonsai and displays that truth in the scrub/record
  states
- the app serves `/bonsai-worker.js`, a no-build module worker based on the HF
  Space's Transformers.js/WebGPU shape
- model load/generation status is stored as sanitized browser telemetry, not raw
  note telemetry
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

After wiring the real browser worker, the safety profile is:

| Step | Bound |
| --- | --- |
| Browser q1 model load | 120s max, then interrupt/fallback |
| Browser span generation | 60s max, then interrupt/fallback |
| Raw note egress | first-party phone/laptop edge only; no public tunnel |
| Slack egress | still verifier-gated; browser model output is never trusted raw |

The live server was then reprofiled after the worker route was added:

| Step | Result |
| --- | --- |
| `/bonsai-worker.js` | served the q1 WebGPU Bonsai worker from the local edge |
| `/api/scrub` with `backend: browser-gpu` | 15.79s, `gate_pass: true`, `residual_count: 0`, 4 redactions |
| `/api/send` | 0.27s, Slack accepted |
| `/api/trajectory` | 0.04s, gate-clean trajectory stored as `local-000007` |

After adding server-side phone observation telemetry and restarting the web
shell, the same endpoint flow was reprofiled:

| Step | Result |
| --- | --- |
| `/api/status` | model reachable, Slack configured, `client_capability: null` until phone refresh |
| `./run.sh phone-observe` | timed out after 12s because the phone had not refreshed the new server process |
| `/api/scrub` with `backend: browser-gpu` | 10.89s, `gate_pass: true`, `residual_count: 0`, 4 redactions |
| `/api/send` | 0.94s, Slack accepted |
| `/api/trajectory` | 0.04s, gate-clean trajectory stored as `local-000008` |

After wiring `/api/browser-spans`, the server-side finalizer was verified with
synthetic browser proposals:

| Step | Result |
| --- | --- |
| `/api/browser-spans` | accepts exact contextual browser spans plus rules, returns `backend: browser-gpu`, `gate_pass: true`, `residual_count: 0` |
| `/api/browser-spans` with only structured/rules-like spans | rejects with fallback-required error because contextual browser evidence is missing |

The live browser-span contract was then profiled end to end with synthetic
browser proposals:

| Step | Result |
| --- | --- |
| `/api/browser-spans` | 2.50s, `backend: browser-gpu`, `gate_pass: true`, `residual_count: 0`, 4 redactions |
| `/api/send` | 0.18s, Slack accepted |
| `/api/trajectory` | 0.04s, gate-clean trajectory stored as `local-000009` |

The same path now has a repeatable smoke command:

```bash
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh browser-span-smoke
AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke
```

The local HTTPS proxy was verified for the browser path:

| Step | Result |
| --- | --- |
| `https://127.0.0.1:8443/` | served the app over local TLS |
| `https://127.0.0.1:8443/bonsai-worker.js` | served the q1 WebGPU worker |
| `AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh browser-span-smoke` | `browser_spans_seconds: 5.20`, Slack accepted, trajectory `local-000011` |
| `AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke` | `browser_spans_seconds: 2.72`, Slack accepted, trajectory `local-000010` |

Supply-chain truth: the worker tries to import a local first-party runtime from
`/vendor/transformers.js` and falls back to the CDN only when the local runtime
is absent. Vendor it for the sovereign demo path:

```bash
./run.sh vendor-browser-runtime
```

This writes `.airplane/browser-vendor/transformers.js` plus the package's shipped
browser sidecar files and keeps those artifacts out of git. The current verified
local browser runtime files are:

| Local route | Size |
| --- | --- |
| `/vendor/transformers.js` | 431,974 bytes |
| `/vendor/ort-wasm-simd-threaded.jsep.mjs` | 46,490 bytes |

Raw notes do not go to the CDN in either mode, but regulated end-user
deployments should self-host or vendor that runtime behind the same first-party
network.

Model-artifact truth: Transformers.js resolves local model files from
`/models/{model}/{file}`. The web worker sets `env.localModelPath = "/models/"`
and the web server serves nested model artifacts from `.airplane/browser-models`.
Warm that cache before a live demo:

```bash
./run.sh vendor-browser-model
```

The verified q1 cache for `onnx-community/Bonsai-1.7B-ONNX` contains:

| Local route | Size |
| --- | ---: |
| `/models/onnx-community/Bonsai-1.7B-ONNX/config.json` | 1,981 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/generation_config.json` | 290 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/tokenizer.json` | 9,117,036 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/tokenizer_config.json` | 4,598 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/chat_template.jinja` | 4,063 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/onnx/model_q1.onnx` | 503,681 bytes |
| `/models/onnx-community/Bonsai-1.7B-ONNX/onnx/model_q1.onnx_data` | 290,552,764 bytes |

This does not prove phone-local generation yet; it proves the phone can load the
browser runtime/model from the first-party edge instead of from Hugging Face
during the demo.

The local artifact routes behave like browser-consumable static files:

- `HEAD` returns `Content-Length` and `Accept-Ranges: bytes`;
- ranged `GET` returns `206 Partial Content`;
- invalid ranges return `416 Range Not Satisfiable`.

Verified over local HTTPS:

| Check | Result |
| --- | --- |
| `HEAD /models/.../model_q1.onnx_data` | `Content-Length: 290552764` |
| `GET /models/.../model_q1.onnx_data` with `Range: bytes=0-31` | `206`, 32 bytes |
| `HEAD /vendor/transformers.js` | `Content-Length: 431974` |
| `GET /vendor/transformers.js` with `Range: bytes=0-15` | `206`, 16 bytes |

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

Remote Safari launch through `devicectl` was attempted:

```bash
xcrun devicectl device process launch \
  --device Lakshmi \
  com.apple.mobilesafari \
  --payload-url http://192.168.1.88:8099/ \
  --activate
```

It failed with CoreDevice error `10005`: Developer Mode is disabled. That does
not block the demo. It means phone observation comes through server telemetry:

```bash
./run.sh phone-observe
```

Then open or refresh the printed phone URL. For secure-context WebGPU testing,
use the HTTPS proxy URL, for example `https://192.168.1.88:8443` or the matching
LAN/hotspot IP printed by `./run.sh https-proxy`. A successful observation writes
`.airplane/phone-capability-latest.json`; until then `/api/status` correctly
reports `client_capability: null`.

For local HTTPS, `./run.sh phone-observe` uses
`.airplane/certs/airplane-local-ca.pem` automatically when that CA exists. If it
times out with a populated last status and `client_capability: null`, the edge is
reachable; the phone has simply not refreshed/reported against the current
server process yet.

There is also passive phone-request telemetry for cases where the browser loads
the app or artifacts but the JavaScript capability post does not complete:

```bash
AIRPLANE_WEB_URL=https://192.168.1.88:8443 ./run.sh phone-request-observe
```

`/api/status.browser_requests` records recent PHI-free browser surface requests:
route, path, client address, iPhone-like user-agent flag, range header, and
artifact byte size. It does not record note bodies or API payloads. A real phone
proof should show a non-local client with `looks_like_iphone: true` requesting
`/`, `/bonsai-worker.js`, `/vendor/transformers.js`, and ultimately
`/models/onnx-community/Bonsai-1.7B-ONNX/...`.

The main demo page now sends an immediate PHI-free client heartbeat before the
WebGPU adapter probe runs, then posts richer capability/model telemetry after
the probe completes. That avoids a false negative where an iPhone loads the page
but Safari stalls or rejects `navigator.gpu.requestAdapter()` before
`/api/client-capability` receives anything.

For live demos, open `/proof` or the short alias `/p` on the laptop. It is a
PHI-free operator view over `/api/status`: Slack health, local model health,
latest phone capability, and recent browser-surface requests. It also prints the
current phone URLs from the edge server's LAN interfaces. It is deliberately
separate from the scrub flow so proof gathering cannot capture note text.

Opening `/proof` from the phone also posts a PHI-free capability heartbeat, so it
can satisfy both proof channels: active `/api/client-capability` telemetry and
passive `/api/status.browser_requests` telemetry.

## Optimal Path From Here

### Step 1: Keep The Current Demo Stable

The current Mac-edge path remains the fallback demo. It already posts to Slack
and has the verifier gate.

Do not break it while building browser GPU.

### Step 2: Measure Browser Span Finalization On The Phone

The selectable backend exists:

```text
Backend: Mac edge | Browser GPU spike
```

The browser span contract exists in the web shell. Browser GPU mode attempts to
load and generate with Bonsai q1 in the browser, parse span JSON, and finalize
through `/api/browser-spans`. It still needs to be measured on Jai's phone:

- successful phone-local model generation;
- valid span JSON from that generation;
- `/api/browser-spans` acceptance from the phone-driven UI;
- Slack delivery from the resulting browser-span record.

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
