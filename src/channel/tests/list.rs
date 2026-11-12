//! **The address list** (yog's `docs/REMOTE.md` §15.3; DESIGN §4.49): an
//! entry's endpoints are dialled in order, and the one that worked is tried
//! first for the rest of the run.

use std::net::TcpListener;
use std::sync::mpsc;

use super::{answered, wired};
use crate::channel::Channel;
use crate::channel::hello::PROTOCOL;
use crate::channel::material::{ADDRESS, Material, Whose, read_dir};
use crate::test_support::server::Server;
use crate::test_support::{Scratch, mint};
use serde_json::{Value, json};

/// The entry in `scratch` with `first` stated ahead of whatever address its
/// server wrote.
fn ahead(scratch: &Scratch, first: &str) -> Material {
    let file = scratch.join(ADDRESS);
    let served = std::fs::read_to_string(&file).expect("the server's address");
    std::fs::write(&file, format!("{first}\n{served}\n")).expect("the list");
    read_dir(scratch.path(), Whose::Own)
        .expect("readable")
        .expect("provisioned")
}

/// A loopback listener that takes each connection, says so, and hangs up
/// without a byte: something that answers and is not the server.
fn stranger() -> (String, mpsc::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let at = listener.local_addr().expect("bound").to_string();
    let (took, taken) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if took.send(()).is_err() {
                return;
            }
            drop(stream);
        }
    });
    (at, taken)
}

fn ask(channel: &Channel) -> Vec<Value> {
    channel
        .ask(&json!({"op": "workspaces"}))
        .expect("an endpoint answered")
}

/// **An endpoint nothing listens at is passed over**, and the next one in the
/// list answers. Port 1 is unbindable without privilege, so nothing answers
/// there.
#[test]
fn a_dial_falls_through_a_dead_endpoint_to_the_next_in_the_list() {
    let (scratch, server, _) = wired(PROTOCOL, vec![vec![answered()]]);
    let held = ahead(&scratch, "127.0.0.1:1");
    assert_eq!(held.addresses.len(), 2);
    let channel = Channel::open(&held).expect("opened");
    assert!(channel.address().starts_with("127.0.0.1:1, 127.0.0.1:"));
    assert_eq!(ask(&channel), vec![answered()]);
    assert_eq!(server.heard().len(), 2, "the preface and the one request");
}

/// **What answers and is not the server is passed over too, and the endpoint
/// that worked is tried first from then on** — in RAM, for the run. The
/// stranger at the head of the list is dialled once and never again.
#[test]
fn the_endpoint_that_worked_is_dialled_first_for_the_rest_of_the_run() {
    let (scratch, _server, _) = wired(PROTOCOL, vec![vec![answered()], vec![answered()]]);
    let (at, taken) = stranger();
    let channel = Channel::open(&ahead(&scratch, &at)).expect("opened");
    assert_eq!(ask(&channel), vec![answered()]);
    assert_eq!(taken.try_recv(), Ok(()), "the head of the list was tried");
    assert_eq!(ask(&channel), vec![answered()]);
    assert!(taken.try_recv().is_err(), "and not a second time");
}

/// **A server under another CA is not the server**: its handshake does not
/// verify, nothing is said to it, and the list goes on.
#[test]
fn an_endpoint_that_cannot_present_the_operators_ca_is_passed_over() {
    let (scratch, _server, _) = wired(PROTOCOL, vec![vec![answered()]]);
    let elsewhere = Scratch::new();
    mint::material(elsewhere.path());
    let other = Server::start(elsewhere.path(), PROTOCOL, vec![vec![answered()]]);
    let theirs = std::fs::read_to_string(elsewhere.join(ADDRESS)).expect("its address");
    let channel = Channel::open(&ahead(&scratch, &theirs)).expect("opened");
    assert_eq!(ask(&channel), vec![answered()]);
    assert_eq!(other.heard(), Vec::<Value>::new(), "nothing was said to it");
}

/// **A list that reached nothing says why of every line in it**, and nothing
/// crossed.
#[test]
fn a_list_that_reaches_nothing_names_every_endpoint() {
    let scratch = Scratch::new();
    let held = mint::provisioned(scratch.path(), "127.0.0.1:1\n127.0.0.1:2\n");
    let reach = Channel::open(&held)
        .expect("opened")
        .ask(&json!({"op": "workspaces"}))
        .expect_err("refused");
    assert!(!reach.crossed());
    let said = reach.said();
    assert!(said.contains("connect 127.0.0.1:1: "), "{said}");
    assert!(said.contains("; connect 127.0.0.1:2: "), "{said}");
}

/// A line that is not an endpoint refuses when the channel opens, naming it.
#[test]
fn a_line_that_names_no_server_refuses_the_whole_entry() {
    let scratch = Scratch::new();
    let held = mint::provisioned(scratch.path(), "server.example:9000\nnot a name:9000\n");
    let refusal = Channel::open(&held).expect_err("refused");
    assert!(refusal.contains("not a name:9000"), "{refusal}");
}
