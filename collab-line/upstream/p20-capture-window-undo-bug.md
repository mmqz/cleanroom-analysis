# [DRAFT — ready to file] UndoManager: partial undo + redo panic with two managers and capture_timeout > 0 on interleaved same-origin writes

> Target repo: yjs/y-crdt · Component: `yrs::undo::UndoManager`
> yrs version: **0.28.0** (latest on crates.io at the time of writing)
> Reproduces: always (deterministic)

## Summary

With **two `UndoManager` instances** on the same document (each tracking a
different origin, both expanded over the same scope) and
**`capture_timeout_millis > 0`**, a same-window interleaved write pattern
(`H1/A1/H2/A2`) leads to two distinct failures:

1. **Partial undo**: two writes from the same origin that fall into the same
   capture window (`H1`, `H2`) are merged into one stack item, but undo of
   that item only applies the first segment — the document ends up in
   `"A1H2A2"` instead of the expected `"A1A2"` after undoing both human
   writes.
2. **Redo panic**: after the partial undo, a subsequent `redo_blocking()`
   panics with `Option::unwrap()` on `None` inside
   `yrs::store::Store::materialize` (reached via `UndoManager::pop_blocking`).

With `capture_timeout_millis = 0` the same sequence behaves correctly
(one stack item per transaction, full undo, clean redo).

## Minimal reproducer

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

use yrs::{
    ClientID, Doc, GetString, Options as DocOptions, Origin, Text, Transact, UndoManager,
};

fn main() {
    let mut opts = DocOptions::with_client_id(ClientID::new(1));
    opts.skip_gc = true;
    let doc = Doc::with_options(opts);

    let text = doc.get_or_insert_text("text");

    let mut mk = |origin: &str| {
        let mut m = UndoManager::with_options(yrs::undo::Options {
            capture_timeout_millis: 1000,
            ..Default::default()
        });
        m.include_origin(Origin::from(origin));
        m.expand_scope(&doc, &doc.get_or_insert_text("text"));
        m
    };
    let mut human = mk("human");
    let mut ai = mk("ai");

    let append = |origin: &str, s: &str| {
        let mut txn = doc.transact_mut_with(Origin::from(origin));
        let end = text.len(&txn);
        text.insert(&mut txn, end, s);
        txn.commit();
    };

    // Interleaved writes: H1 and H2 land in the same capture window,
    // separated by an A1 write from the other origin.
    append("human", "H1");
    append("ai", "A1");
    append("human", "H2");
    append("ai", "A2");
    assert_eq!(text.get_string(&doc.transact()), "H1A1H2A2");

    // (1) Partial undo: undoing the human stack down to empty leaves "A1H2A2".
    while human.undo_blocking() {}
    println!("after undo-human: {:?}", text.get_string(&doc.transact()));

    while ai.undo_blocking() {}
    println!("after undo-ai: {:?}", text.get_string(&doc.transact()));

    // (2) Redo panic.
    let r = catch_unwind(AssertUnwindSafe(|| {
        while human.redo_blocking() {}
    }));
    if r.is_err() {
        println!("redo-human PANICKED — bug symptom 2");
    } else {
        println!("redo-human ok: {:?}", text.get_string(&doc.transact()));
    }
}
```

Observed output (yrs 0.28.0, rustc 1.99.0, Linux x86_64, **release** profile):

```text
after undo-human: "A1H2A2"   # partial undo: H2 survives (expected "A1A2")
after undo-ai: "A1H2A2"      # AI stack undo leaves the text unchanged here
thread 'main' panicked at store.rs:340:50:
called `Option::unwrap()` on a `None` value   # during redo_blocking()
redo-human PANICKED — bug symptom 2
```

Note: in **debug** builds the same reproducer panics even earlier, with a
different symptom: `attempt to subtract with overflow` at `block.rs:652`
inside the first `undo_blocking()` call (arithmetic overflow check on the
same buggy path). We are reporting the release-profile behavior as the
primary symptom; the debug overflow may be useful as an additional
locator for maintainers.

## Configuration matrix (from our investigation)

| config | managers | capture_timeout | result |
|---|---|---|---|
| 1 | two (per-origin) | 1000 ms | **partial undo + redo panic** |
| 2 | two (per-origin) | 0 ms | correct (full undo, clean redo) |
| 3 | one (both origins tracked) | 1000 ms | partial undo (merge across window) |
| 4 | one (both origins tracked) | 0 ms | correct |

The failing ingredient is the **capture window merging** in combination
with **multiple managers** sharing one scope: the merged stack item
apparently only records the first same-origin segment of the window, and
the redo path later fails to materialize the partially-tracked structs.

## Expected behavior

- Undoing all transactions of one origin should remove **all** of that
  origin's inserts from the window, not just the first segment.
- Redo should never panic on state produced by the library's own undo.

## Workaround

We currently run with `capture_timeout_millis: 0` (one stack item per
transaction), which avoids both failures. We kept a regression test that
re-enables the window and fails if the interleaved pattern ever breaks
again, so we can lift the workaround once this is fixed upstream.

## Environment

- yrs 0.28.0 (crates.io), features: `sync`
- rustc 1.99.0 (stable), Linux x86_64
- Also observed with `skip_gc: true`; `skip_gc` is not required for the
  panic (the partial-undo symptom reproduces without it).

---

*Attachment note for filing*: the matrix probe with four isolated
configurations and per-config panic isolation is available on request
(~150 lines, self-contained, no external dependencies beyond `yrs`).
