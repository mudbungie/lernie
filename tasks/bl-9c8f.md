+++
title = "Lost wakeup in the 2.11 deposit-probe-launch protocol: a deposit racing a driver's last inbox read is stranded forever"
created = 1785130626
updated = 1785131150
claimant = "Gudgeon"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
A deposit can be permanently stranded: the depositing writer sees `Busy` and declines to launch, while the lease-holder it deferred to has *already taken its last look at the inbox*. Nothing drives the agent again until a hand-run `lernie scan`. PROVEN by intervention, not inference (see below).

## The race (ARCH §2.11 deposit → probe → launch)

Holder H (a driver) and writer W (`lernie message`, `lernie dispatch`, or a terminal result deposit):

```
H: acquire lease
H: drain::pending(inbox)  -> []          <- H's LAST inbox read
W:                            deposit user-001.md
W:                            try_acquire -> fails (H holds) => ProbeOutcome::Busy, no launch
H: ... release lease
   => mail pending, no holder, no driver, nothing scheduled. Stranded.
```

`inbox::probe_and_launch` (src/prompt/inbox/mod.rs) is correct in isolation — `Busy` means "another executor holds the lock; it will drain at its next step boundary" — but that is only true if the holder has an inbox read left. The **pin-1 no-op driver** has none: it acquires, finds nothing, and exits silently with "no step, no epitaph, no further launch" (§2.11 pin 1, `dispatch/advance.rs` `Warrant::NothingDue => Ok(AdvanceOutcome::NothingToDo)`), releasing the lease with no post-release hook at all. The `Terminal` path at least funnels through `terminal::exit_launch`; the pin-1 path funnels through nothing.

The effective read is `drain::pending()` *inside* `drain::drain` (`dispatch/drain.rs`), not the earlier count in `driver::deliver` — widening at the latter does not reproduce; widening at the former does, 100%.

## Reproduction (decisive, 100%)

In `src/prompt/dispatch/drain.rs::drain`, hoist the enumeration and sleep between it and the delivery loop:

```rust
    recover_strays(worktree, conv_id, git)?;
    let widened = pending(inbox)?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    for msg in widened {
```

Then `cargo test --lib e2e::advance_cli::message_launches -- --test-threads=1`. Fails every time. With `probe_and_launch` traced, the forensics are unambiguous:

```
"/tmp/.tmpOt41Qd/conv/agents/<id>/messages/003-user.md" never appeared, and
/tmp/.tmpOt41Qd/conv went untouched for 60s - nothing is driving it
  inbox ".../conv/inbox/<id>" holder=Ok(None) entries="user-001.md"
  message stderr: "TEMP-PROBE <id> start\nTEMP-PROBE <id> BUSY\n"
```

Undelivered mail + no lock holder + writer probe returned BUSY = the diagram above.

## In the wild

This is the actual root cause of **bl-2bf0** (`e2e::advance_cli::message_launches_a_detached_advance_chain_that_batons_through_tools`), whose premise — "a slow success outran the 120s bound" — is now disproved. In the filer's iteration the whole `cargo test --lib` took 120.97s, i.e. the 120s was consumed by this one test while the other ~900 finished inside it: the chain was **dead**, not slow. Reproduced unaided on the landed tree twice in ~17 full `e2e::` runs at load 40-55 (~1 in 8), with the identical forensic signature. Sequence in that test: exchange 1's `lernie prompt` exit-launches the pin-1 terminator; the test then runs `lernie message`, whose deposit lands after the terminator's `pending()` and whose probe lands before the terminator's release.

Not test-only. Any `lernie message` / `lernie dispatch` / child result deposit that races a driver's last inbox read loses the wakeup, in production.

## Proposed fix — the dual of the deposit rule

§2.11 states one edge: *a deposit into a quiescent agent starts a driver* (the writer probes and launches). The missing edge is its dual: **quiescence arriving while mail pends must start a driver too.** So: whoever releases a lease re-reads the inbox *after* releasing, and launches a driver at its own agent if anything is pending.

Airtight, because the two edges partition the timeline: if W's probe succeeded, nobody held the lease and W launches; if it failed, the holder still held the lease at T_probe > T_deposit, so the holder's post-release re-read (T_rel > T_probe) necessarily sees that deposit. Termination is preserved — a launch happens only when mail is actually pending, and delivering consumes it, so the pin-1 recursion terminator still terminates (no mail, no launch).

It is one rule, not a patch per site, and it is not a special case: it is the same invariant (*mail pending + nobody driving => launch*) observed from the other end.

Sites that release a lease: `dispatch/advance.rs::run` (both `NothingToDo` returns and the `Terminal` arm), `dispatch/mod.rs::run_exchange`'s tail, `dispatch/driver.rs::drive`. Prefer funnelling them through one post-release helper over three copies.

Interaction with **§2.11 pin 2** (epitaph-valued launches: `stopped` and `budget-exhausted` never relaunch) needs deciding, not assuming: pin 2 forbids relaunching with *nothing new*, whereas a deposit that landed during the final step is new work and would have launched a driver of its own had it arrived a millisecond later (§2.9/§2.10 make messaging a stopped branch the resume path). Argue it explicitly in ARCH §2.11 rather than leaving it implied.

## Deliverables

- The post-release re-read, at one site, all release paths funnelled through it.
- ARCH §2.11: amend pin 1's "no further launch" and state the dual rule and its pin-2 interaction.
- Unit coverage of the race, deterministically (hold a lease, deposit, probe -> Busy, release -> assert a launch was requested against a recording launcher) — no sleeps, no e2e flake-hunting.
- Bar: `e2e::advance_cli` 30 consecutive passes under ~40 spinners, plus the widening diff above reproducing green (with the fix, the widened window must no longer strand).