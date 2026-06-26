# Airplane Mode

**A coaching scribe that runs entirely on your phone. Even a 2019 one. Even with the radios off.**

Talk through a session, and the note gets de-identified *on the device* — then a clean, structured record posts to your workspace and the client gets a genuinely useful follow-up. The client's name and disclosures never leave the phone. Not deleted-after-upload. Never uploaded.

This is a demo. It exists to make one bet visible enough to argue about.

---

## The bet

> "AI's future will not be defined by who can build the largest datacenters. It will be defined by who can deliver the most intelligence per unit of energy and cost." — Vinod Khosla

If that's true, the most sensitive workloads should run where the data already is — on the device in someone's hand — instead of shipping the data to the compute. Mental-health coaching is the sharpest test: deeply personal, and (because a coach isn't a HIPAA-covered entity) governed by consumer-health-data law with a private right of action and no margin for a leak.

So we ran it on a 1-bit model on an iPhone 11. Offline. It works.

---

## 60 seconds

1. **Airplane mode on.** Consent, then talk through a (synthetic) session. The phone physically can't transmit.
2. **It scrubs on-device.** Watch it catch the name buried in a sentence, not just the obvious ones.
3. **It structures.** Themes, commitments, follow-ups, risk-flags — a real record.
4. **It refuses to send** — still offline. Then airplane mode off, and *only the de-identified record* posts to Slack. The name never left.
5. **A follow-up note** goes to the client, tied to their own commitment — not a generic reminder.
6. **One clean trajectory** is recorded, and the counter ticks up. The system has something to learn from — and it learned it without any raw data leaving the device.

---

## Why this is different

Everyone else's privacy story is *cloud + fast deletion + a signed agreement*. Ours is *the data never leaves*. That's not a stronger promise of the same kind — it's a different kind of promise. Airplane mode is the proof you can watch.

And the value isn't the note — it's the **loop**. Capture → clean → record → follow-up → outcome → a little smarter next time. The loop compounds. The privacy holds because everything that compounds is de-identified first.

---

## Make it yours in five files

The hard, correctness-critical core is signed and you never touch it. Everything that's specific to *your* practice is a **pack** — five declarative files, no code:

```
coach-pack/
├── recognizers/   your identifiers (member IDs, partner names)
├── schema/        your note shape
├── policy/        what to redact, which model size
├── sink/          where clean records go
└── eval/          your golden notes — proves it doesn't leak
```

Edit them, run the eval gate (it has to pass before it ships), and you're running the same core with your identifiers. No fork. A pack can't see raw data, the redaction map, or the gate — so it's safe to write, and safe to share.

---

## Pull it down

```
git clone <repo>
cd airplane-mode
./run.sh        # loads the model, the coach-session pack, and the demo
```

You'll need an iPhone (A13 / iPhone 11 or newer) and a Slack channel. Runs offline after first setup.

---

## This is a demo, not a product

Be honest about what it is: synthetic data only, one phone, one channel. The environment **grows** (it accumulates de-identified trajectories) — it does **not** train an agent yet. We're showing the *shape* of what's possible, not a finished system.

---

## Help us with the hard parts

The unsolved questions are the invitation. If the bet interests you, these are where it gets real:

- **Reward without surveillance.** The right reward is *the client needing us less* — autonomy, never engagement. How do you measure that cheaply and honestly?
- **Safe learning across people.** De-identified trajectories can still leak via linkage. What's the right floor — minimum cohorts, differential privacy — before any pooling?
- **Where the policy improves.** On-device, federated, or pooled? Each is a different privacy bargain.
- **The autonomy reward, formalized.** How do you build a follow-up loop that's structurally incapable of optimizing for dependence?

Open an issue. Pull the demo. Tell us where the bet breaks.

*Built to make intelligence come to the data — not the other way around.*
