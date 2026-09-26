+++
title = "the seat's call carries only its route-local addresses, which on a hotspot or carrier NAT are private: read the DHT-observed address (BEP 42 ip) and put it in the call, as yog bl-efae does for presence"
created = 1790394983
updated = 1790394989
claimant = "Urinalyses-Z7"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
yog REMOTE §13.2 rules an endpoint list must carry the OBSERVED endpoint; §13.8 measured that on a cellular path the observed one is the only one there is. bl-e437 ported yog's walk but skipped yog bl-efae's observed-address vote because a seat publishes no presence — but the seat's sealed CALL (src/channel/rendezvous/item.rs, built by the ladder's full-rendezvous rung) is an endpoint list too and lists only route-local addresses, so a laptop on a phone hotspot or any carrier NAT hands the engine an address it cannot punch to. Port yog bl-efae (yog 022253e0: krpc reply ip, both lengths; observed() per-family majority vote over the last walk, tie gives nothing, a dark walk clears it) and append each observed address not already listed to the call at the seat's punch port — address only, never the observed port (yog REMOTE §13.2; the port-rewrite carrier case stays open, §13.8). Tests: parser both lengths + garbage; 3-vs-1 majority, tie none; call with observed present/absent round-tripped through the unseal; no duplicate of a local address. State it in DESIGN. Companions: yog-android bl-544b, thrall bl-d340.