+++
title = "a QR code the seat draws itself: the byte-mode encoder, and the module matrix it answers"
created = 1788147675
updated = 1788148754
claimant = "OrderScribe2"
parent = "bl-1f10"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
DELIVERED INSIDE bl-1f10, which is where it had to land.

The encoder is `src/qr/` — seven files, byte mode at correction level M, the smallest version that fits, and no new dependency. Its rulings, the two places two reference implementations disagree with each other, and the proof are all recorded in bl-1f10's body and in the modules' own doc comments.

**Why it could not be its own delivery.** A crate that denies dead code refuses a module no production line calls — correctly, and with no lawful local override: an inline suppression is forbidden (policy lives in the manifest) and a crate-wide `dead_code = "allow"` is a real relaxation bought for one module. Marking it `pub` to dodge that would have been a lie about what the library surface is. So the encoder landed in the same commit as the line that calls it.

The record is worth keeping: **land a module with its caller, or the gate will tell you to.**