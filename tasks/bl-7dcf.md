+++
title = "bare `lernie` opens the window and the help never says so — the usage line reads `lernie <verb>`, which says a verb is required"
created = 1788150501
updated = 1788150501
priority = 5
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["docs"]
+++
`lernie --help` opens with:

    usage: lernie <verb> [argument…]
           lernie start <workspace> <goal>
           lernie ask <envelope>
           lernie entries
           lernie help [<verb>]
           lernie [--version | --help]

Then a verb table: `workspaces`, `conversations`, `transcript`, `follow`,
`message`, `nudge`, `enroll`, `start`, `ask`, `entries`, `help`, `-V`, `-h`.

The window is in neither list. It is reached by running the binary with **no
arguments at all** (`src/cli.rs`: `[] => Decided::Window`), and the help's own
usage line says the shape is `lernie <verb>` — which reads as *a verb is
required*, the opposite of the truth.

This is the crate's headline capability. `Cargo.toml` calls eframe/egui "THE
WINDOW, and the largest approval this crate will ever" have; yog's `main.rs`
records that the window is now the seat crate's and that yog itself has "no
display stack in the process". A reader of `lernie --help` cannot learn that the
seat has a window, let alone how to open one — the only route is running the
binary wrong on purpose, or reading the source.

The prose paragraph under the usage block does not rescue it either: it says the
seat "dials in over mTLS, asks and acts, and **paints what comes back**", which
a reader will take as describing the CLI verbs' own output, because every
documented verb prints envelopes to stdout.

## Fix

A usage line and a table row. Something in the shape of:

    usage: lernie                       open the window on this box's channels
           lernie <verb> [argument…]

and a row in the verb table for the bare form, since that table is where a
reader looks for "what can this do". The `What it reads` paragraph at the foot
already describes the channel provisioning both faces share, so nothing else
needs to move.