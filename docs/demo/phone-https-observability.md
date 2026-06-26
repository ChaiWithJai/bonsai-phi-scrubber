# Phone HTTPS Demo Observability

This is the live-demo challenge: make a phone reach the first-party edge over HTTPS,
scrub a synthetic note, prove the verifier gate passed, and show where the time went.

## What "working" means

The working demo path is:

```text
phone browser -> https://<mac-lan-ip>:8443 -> laptop edge -> Bonsai scrub -> verifier gate -> Slack
```

The raw synthetic note crosses only the first-party phone/laptop boundary. It does
not go through a public tunnel. Browser GPU remains the phone-local benchmark path,
but first-load model warmup can pull hundreds of MB and compile kernels. For the
live demo, an unwarmed phone Browser GPU path must not block the scrub. The UI records
that as the challenge and falls back to the Mac-edge verifier path.

## Run it

Terminal 1:

```bash
./scripts/serve-model.sh
```

Terminal 2:

```bash
./run.sh web
```

Terminal 3:

```bash
./run.sh https-proxy
```

Open the printed HTTPS phone URL, currently shaped like:

```text
https://192.168.1.163:8443
```

Use the sample note for the live demo. It keeps the benchmark comparable.

## Verify it

Check edge health:

```bash
curl -k https://127.0.0.1:8443/api/status | jq '{model,slack,network}'
```

Run the same scrub through the HTTPS proxy:

```bash
time curl -k -sS -X POST https://127.0.0.1:8443/api/scrub \
  -H 'Content-Type: application/json' \
  --data '{"text":"Met with Maria Alvarez (CM-204815) at her place Tuesday. She is the one whose daughter just started college. Committed to a 10-min morning walk daily.","backend":"mac-edge"}' \
  | jq '{redactions: (.redactions|length), gate_pass, residual_count, observability}'
```

Known-good measurement on June 26, 2026:

```json
{
  "redactions": 4,
  "gate_pass": true,
  "residual_count": 0,
  "observability": {
    "passes": 5,
    "scrub_ms": 14024,
    "structure_ms": 2593,
    "total_ms": 16650,
    "word_count": 25
  }
}
```

The web result screen also shows the observed pass count, word count, total seconds,
and HTTPS phone URL.

## Observe a phone

Use the proof page during the demo:

```text
https://<mac-lan-ip>:8443/proof
```

It refreshes `/api/status` and shows:

- model reachability;
- Slack route;
- current phone HTTP/HTTPS URLs;
- the latest client capability report;
- recent PHI-free browser requests.

If the phone run exceeds the known-good HTTPS smoke by a lot, inspect:

```bash
tail -n 80 .airplane/web.log
curl -k https://127.0.0.1:8443/api/status | jq '.client_capability, .browser_requests[-12:]'
```

The server log emits lines like:

```text
scrub: backend=mac-edge words=25 passes=5 redactions=4 residual=0 scrub_ms=14024 structure_ms=2593 total_ms=16650
```

## Chrome/iOS dictation crash profile

Observed on June 26, 2026: Chrome on iPhone reached the HTTPS app and reported
WebGPU support, but the tab reset while loading the browser model. Telemetry showed:

- user agent: `CriOS` on iPhone;
- hardware concurrency: 4;
- `device_memory`: `0`;
- repeated `/bonsai-worker.js` and `/vendor/transformers.js` requests;
- Browser model status moving through `loading Bonsai q1 ONNX weights`,
  `download 100%`, then resetting to `idle` or timing out.

Interpretation: dictation was not the primary failure. The in-page Browser GPU
warm path was allocating enough model/runtime memory for WebKit to kill the tab.
For the live phone demo, Browser GPU is therefore benchmark-only. The UI disables
in-page Bonsai warmup on phone-like clients and routes the live scrub through the
Mac-edge path over first-party HTTPS.

To confirm the fix, open a fresh phone tab and watch:

```bash
curl -k https://127.0.0.1:8443/api/status | jq '.browser_requests[-20:]'
```

During dictation, a healthy live demo should show `/` and `/api/client-capability`
requests, not repeated `/bonsai-worker.js`, `/vendor/...`, or `/models/...` fetches.

## Interpret latency

"Local" means the note stays inside the first-party edge boundary. It does not
mean the model computation is instant.

For the current web demo, the verified path runs 5 seeded Bonsai passes plus one
clean-record structuring call. On the sample note, the expected range is roughly
15-30 seconds. Longer notes may take 30-60 seconds. If the phone is waiting longer
than that, the likely issue is Browser GPU first-load warmup, not Slack or HTTPS.

The demo posture is honest:

- **Proved today:** HTTPS phone-to-laptop edge, 5-pass Bonsai scrub, verifier gate,
  zero residuals, Slack-ready clean record.
- **Challenge to extend:** make Browser GPU or native MLX text inference finish
  inside the same observed envelope without weakening the verifier gate.
