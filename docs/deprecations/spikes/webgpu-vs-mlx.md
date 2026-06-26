# WebGPU vs. MLX For The Phone Path

This note compares two ways to make the phone do the sensitive work locally:

- **Native iOS + MLX Swift:** a signed iOS app loads Bonsai MLX text weights and
  calls the Swift inference port.
- **Browser + WebGPU:** the phone browser runs a WebGPU-backed model runtime and
  calls the same scrub/gate contract from the web shell.

The goal is not to pick the harder path. The goal is to reuse the ecosystem
where it is already strong, then measure honestly on the real device.

For the deeper pre-spike research method and worked Airplane Mode case study,
see [`phone-feasibility-research-plan.md`](phone-feasibility-research-plan.md).

## Current State In This Repo

The repo is already shaped for either option:

- `airplane-core` owns the trust logic: parse, validate, clamp, scrub, gate.
- The model is a port: `complete(messages, json_schema, sampling) -> String`.
- `shells/ios` has a simulator-safe `TextInferenceProviding.complete(...)`
  boundary and backend-compatible DTOs.
- `shells/web` already runs the phone-facing demo, but today the model work runs
  on the Mac edge node through `llama-server`.

What is not proven yet:

- real Bonsai 1.7B text inference on iPhone;
- memory and tokens/sec on the actual device;
- on-device eval parity with the CLI;
- browser offline behavior with model weights cached locally.

## Option A: Native iOS + MLX Swift

Use this when the headline must be a real native iPhone app with explicit control
over local storage, app lifecycle, and eventual radio/offline proof.

What it gives us:

- best alignment with PrismML's Apple/MLX story;
- a clean place to implement `TextInferenceProviding.complete(...)`;
- direct access to Keychain/Secure Enclave APIs for raw-note and redaction-map
  storage;
- a future path to a stronger "runs inside the app" proof.

Costs and risks:

- we must wire `mlx-swift` for text, not just image demos;
- iOS structured decoding is our responsibility because there is no
  `response_format: json_schema` server on device;
- TestFlight/signing/device deployment becomes part of the build loop;
- this path may fail the iPhone 11/A13 performance target.

Best spike:

1. Add a tiny signed iOS app target.
2. Load the smallest practical Bonsai MLX text artifact.
3. Implement one `TextInferenceProviding.complete(...)` call.
4. Run one synthetic prompt.
5. Record: does it load, peak memory, first-token latency, tokens/sec.

## Option B: Browser + WebGPU

Use this when we want to exploit the web inference ecosystem and keep the demo
install-free. This is not "reinvent the wheel"; it is trying the wheel the web
community is already standardizing around.

Ecosystem paths worth testing:

- **Bonsai WebGPU Hugging Face Space:** a community demo already runs
  `onnx-community/Bonsai-1.7B-ONNX` in-browser through Transformers.js/WebGPU.
  This is the first URL to open on the phone:
  <https://huggingface.co/spaces/webml-community/bonsai-webgpu>.
- **WebLLM / MLC:** browser LLM runtime over WebGPU with an OpenAI-style API
  surface. This is the closest conceptual match to the current `llama-server`
  route.
- **Transformers.js:** Hugging Face's browser/runtime library with WebGPU support
  for transformer pipelines.
- **ONNX Runtime Web:** WebGPU execution provider for ONNX models in browser
  contexts.

What it gives us:

- no App Store, TestFlight, or Xcode signing loop for the first real phone
  measurement;
- the existing phone web shell can become the measurement harness;
- a single browser demo can serve laptop, tablet, and phone paths;
- easier community adoption for hackathon users who already understand web apps.

Costs and risks:

- WebGPU support depends on browser and OS version; older iPhones may not work;
- model artifact conversion may be harder than the UI work;
- browser memory limits and tab eviction are real constraints;
- offline model caching, persistence, and cold-start time must be measured;
- Secure Enclave/Keychain storage is not available from the browser.

Best spike:

1. Add a separate `shells/webgpu-spike/` or feature-flagged web route.
2. Detect `navigator.gpu` and report browser/device capability.
3. Run the smallest WebGPU model that can produce JSON spans.
4. Try a Bonsai-compatible artifact only after the runtime path is proven.
5. Record: load time, cache size, peak memory proxy, first-token latency,
   tokens/sec, and whether the page survives backgrounding.

## Comparison

| Question | Native MLX Swift | Browser WebGPU |
| --- | --- | --- |
| Fastest path to real phone measurement | Medium | Likely fastest if device/browser supports WebGPU |
| Best PrismML Apple story | Strong | Medium |
| Best healthcare hackathon ergonomics | Medium | Strong |
| App install required | Yes | No |
| Uses existing web shell | No | Yes |
| Uses existing iOS scaffold | Yes | No |
| Structured decoding burden | Ours | Runtime-dependent, still ours to validate |
| Secure local storage | Strong | Weak |
| Radio/offline proof | Stronger long term | Weaker; browser network/cache behavior is harder to claim |
| Risk on iPhone 11/A13 | High | High, and browser support may be the blocker |

## Recommendation

The first recommendation was to measure both paths. Jai has now validated the
external Bonsai WebGPU demo on his phone, so the near-term decision changes:

1. **Browser GPU first.** It is now the highest-density next step because it can
   reuse the web shell, the phone UX, and the existing Slack/gate loop.
2. **Native MLX Swift second.** It remains the stronger long-term proof if
   browser storage, HTTPS, offline behavior, or structured output are not good
   enough.

The shared invariant stays the same either way:

```text
raw note -> local inference port -> host-side parse/clamp -> verifier gate -> clean egress
```

The model runtime can change. The gate does not.

For the current spike result and narrowed demo path, see
[`browser-gpu-spike-report.md`](browser-gpu-spike-report.md).

## Sources To Check During The Spike

- [WebLLM documentation](https://webllm.mlc.ai/)
- [MLC WebLLM GitHub](https://github.com/mlc-ai/web-llm)
- [Bonsai WebGPU Hugging Face Space](https://huggingface.co/spaces/webml-community/bonsai-webgpu)
- [Bonsai WebGPU Space source](https://huggingface.co/spaces/webml-community/bonsai-webgpu/tree/main)
- [Transformers.js documentation](https://huggingface.co/docs/transformers.js)
- [ONNX Runtime Web documentation](https://onnxruntime.ai/docs/get-started/with-javascript/web.html)
- [WebKit WebGPU updates](https://webkit.org/blog/)
- [MLX Swift GitHub](https://github.com/ml-explore/mlx-swift)
