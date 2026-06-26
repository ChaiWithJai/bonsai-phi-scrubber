# System, Network, and Data Flows

This is the review map for the phone demo at `http://<mac-lan-ip>:8099`.

## 1. System Architecture

```mermaid
flowchart LR
  Phone["iPhone browser\ncapture + demo UI"]
  Web["airplane-web\nRust web shell"]
  Core["airplane-core\nrules + verifier + pipeline + pack loader"]
  Pack["coach-session pack\nrecognizers / schema / policy / sink / eval"]
  Model["Bonsai via llama-server\nlocal inference port"]
  Keychain["macOS Keychain\nSlack webhook or bot token"]
  Traj["local trajectory store\n.airplane/trajectories.jsonl"]
  Slack["Slack #coach-records\nclean record only"]

  Phone -->|"synthetic note over local HTTP"| Web
  Web --> Core
  Core --> Pack
  Core -->|"scrubbed prompt only"| Model
  Model -->|"JSON-ish structured output"| Core
  Core -->|"gate-clean record"| Web
  Web -->|"credential lookup"| Keychain
  Web -->|"only after Slack post succeeds"| Traj
  Web -->|"verifier-approved Slack payload"| Slack
```

Key boundary: `airplane-core` owns rules, verifier, pipeline, and pack loading. The model, capture UI, storage, and Slack are ports around it.

## 2. Network Topology

```mermaid
flowchart TB
  subgraph PhoneNet["Same Wi-Fi or iPhone Personal Hotspot LAN"]
    Phone["iPhone Safari\nhttp://192.168.x.x:8099"]
    MacWeb["Mac airplane-web\n0.0.0.0:8099"]
  end

  subgraph MacOnly["Mac loopback only"]
    Model["llama-server\n127.0.0.1:8080"]
    Keychain["Keychain secrets"]
    LocalStore["local files\npacks + trajectories"]
  end

  subgraph Internet["Internet egress"]
    SlackAPI["hooks.slack.com\nSlack API"]
  end

  Phone -->|"local HTTP"| MacWeb
  MacWeb -->|"loopback HTTP"| Model
  MacWeb -->|"local OS call"| Keychain
  MacWeb -->|"local file IO"| LocalStore
  MacWeb -->|"HTTPS, clean payload only"| SlackAPI
```

Use `http://192.168.1.88:8099/` for the current Mac on this network. If the phone cannot connect, switch both devices onto the iPhone Personal Hotspot and use the `172.20.10.x:8099` address printed by `./run.sh web`.

Do not use a public tunnel for the dictation path during the demo. The raw synthetic note should stay on the local phone-to-Mac link.

## 3. Critical Data Flows

```mermaid
sequenceDiagram
  participant Phone as iPhone browser
  participant Web as airplane-web
  participant Core as airplane-core verifier
  participant Model as local Bonsai server
  participant Store as local trajectory store
  participant Slack as Slack #coach-records

  Phone->>Web: Synthetic raw note
  Web->>Core: Scrub request
  Core->>Core: Rules recognizers redact direct identifiers
  Core->>Model: Scrubbed text + schema prompt
  Model-->>Core: Candidate structured JSON
  Core->>Core: Strip/parse/validate/clamp model output
  Core->>Core: Verifier rescans outgoing record
  Core-->>Web: Gate-clean care record or block
  Phone->>Web: Send to Slack
  Web->>Core: Reverify exact Slack payload
  alt residual identifiers found
    Core-->>Web: Block egress
    Web-->>Phone: Not posted
  else zero residual identifiers
    Web->>Slack: HTTPS post, de-identified payload only
    Slack-->>Web: Accepted
    Web->>Store: Append gate-clean trajectory tuple
    Web-->>Phone: Posted + trajectory count
  end
```

What never leaves the edge:

- Raw note text.
- Redaction map.
- Matched identifier strings.
- Pack internals that would let a sink bypass the verifier.

What can leave after the verifier passes:

- Client pseudonym.
- Themes.
- Commitment text.
- Follow-up text.
- Risk flags and next-touch status.
- Gate-clean footer stating no name/member ID was sent.

Current live proof commands:

```bash
curl http://127.0.0.1:8099/api/status | jq '{slack:.slack, model:.model}'
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
```
