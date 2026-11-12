+++
title = "Re-vendor the corpus at edition 27: enroll and the doctor reply's rendezvous counters are retired upstream, so their leftovers leave the seat"
created = 1794460156
updated = 1794460156
parent = "bl-3719"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
yog deleted the enroll act (yog bl-7a14, REMOTE 15.4). Its corpus lost request/enroll, reply/enrolled, the doctor reply's rendezvous object and the /enroll help row, and shapes.json gained a top-level retired map stamping each removal with the edition it went at (27). The edition is now the largest stamp over the signatures and the retired map.

bl-1f57 left three temporary things for this moment: the enrolled reply fixture filed under corpus/unpainted/, one enroll line in parity.toml, and an edition constant of 26.

Work:
- re-vendor with make corpus (scripts/refresh-corpus.sh), never by hand
- the enrolled fixture and the parity.toml line go
- the edition ledger drops the rows for retired paths; the edition is 27, and the gate test reads the retired map as well as the signatures
- any seat code painting the doctor reply's rendezvous counters goes
- docs/DESIGN.md amended where it describes any of this