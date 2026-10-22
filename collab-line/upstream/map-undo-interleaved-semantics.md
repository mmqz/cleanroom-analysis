# [DRAFT — ready to file] Map undo semantics with interleaved writes from two origins: step-wise rollback + one undo() consuming multiple stack items — intended?

> Target repo: yjs/y-crdt · Component: `yrs::undo::UndoManager` / Map types
> yrs version: **0.28.0** (latest on crates.io at the time of writing)
> Type: semantics clarification / documentation request (behavior observed
> and stable — we pinned it in tests; we are asking whether it is intended
> and would appreciate documentation)

## Summary

We observed two Map-undo behaviors that were surprising to us and could
not find documented. Both concern **per-origin `UndoManager`s on the same
document** with **interleaved writes to the same map key** from two
origins (in our product: a human editor and an AI assistant, each with
their own undo stack — undoing must never undo *the other party's* edits).

**Behavior 1 — step-wise rollback ("undo restores the value at that
operation's historical point, shadowing later writes that are still
present")**

Sequence on key `k` (each write its own transaction, capture window 0):

```text
H1: k=1   A1: k=101   H2: k=2   A2: k=102   H3: k=3   A3: k=203
```

Final value: `k = 203`. Now undo the **A** stack three times:

| undo call | observed `k` | naive expectation (ours) |
|---|---|---|
| undo A3 | 3 (H3's value) | 3 ✔ |
| undo A2 | **2 (H2's value)** | 3 — H3's write is still present, so removing A2's contribution should leave H3 as the latest write |
| undo A1 | **1 (H1's value)** | 3 |

So undoing an *older* interleaved write of origin A restores the value
that was current **at that operation's point in time**, even though later
writes from origin H are still in the document. Mechanically this looks
like "the undo inserts the old value as a fresh write with a newer
clock", which then wins LWW against H3's still-present write.

**Behavior 2 — one `undo()` call consumes multiple stack items
("pop-until-productive")**

After the three A-undos above, the H stack still holds 4 items
(`[AddNode, H1, H2, H3]`). A single `undo()` on the H stack returns
`true` **once** but pops through H3 and H2 (both produce no observable
change — their content was superseded by the A-undo chain) until H1
produces a change. The observable document jumps straight to H1's
historical point (in our repro: the node's creation position), and the
stack depth drops by 3 in one call.

## Questions

1. Are both behaviors intended semantics of the current undo
   implementation (as opposed to known limitations)?
2. If intended, could the docs (or doc comments on `UndoManager`) spell
   out:
   - undo of a map write inserts the historical value as a **new**
     operation (so it shadows concurrent-but-later writes that are still
     present), and
   - `undo()` pops stack items until it finds one with observable
     output (a single call may consume several items)?
3. Is there a recommended configuration for the "two parties, two undo
   stacks, same document" pattern (beyond per-origin managers +
   `capture_timeout_millis: 0`, which is what we run today) that yields
   "undo removes my contribution; the other party's latest write wins"
   semantics instead of step-wise rollback?

## Minimal reproducer

```rust
use yrs::types::ToJson;
use yrs::{ClientID, Doc, Map, Options as DocOptions, Origin, Transact, UndoManager};

fn main() {
    let mut opts = DocOptions::with_client_id(ClientID::new(1));
    opts.skip_gc = true;
    let doc = Doc::with_options(opts);
    let map = doc.get_or_insert_map("m");

    let mut mk = |origin: &str| {
        let mut m = UndoManager::with_options(yrs::undo::Options {
            capture_timeout_millis: 0,
            ..Default::default()
        });
        m.include_origin(Origin::from(origin));
        m.expand_scope(&doc, &doc.get_or_insert_map("m"));
        m
    };
    let mut human = mk("h");
    let mut ai = mk("a");

    let set = |origin: &str, v: i64| {
        let mut txn = doc.transact_mut_with(Origin::from(origin));
        map.insert(&mut txn, "k", v);
        txn.commit();
    };
    let get = || {
        let txn = doc.transact();
        match map.get(&txn, "k") {
            Some(v) => v.to_json(&txn).to_string(),
            None => "<none>".to_string(),
        }
    };

    for i in 1..=3 {
        set("h", i);          // H1..H3
        set("a", 100 + i);    // A1..A3
    }
    println!("final: {}", get()); // 103

    // Behavior 1: step-wise rollback of the "a" stack.
    ai.undo_blocking();
    println!("after undo A3: {}", get()); // 3  (expected)
    ai.undo_blocking();
    println!("after undo A2: {}", get()); // 2  <-- H3 (k=3) is still present
    ai.undo_blocking();
    println!("after undo A1: {}", get()); // 1  <-- H2/H3 still present

    // Behavior 2: one human undo() consumes multiple stack items.
    println!("h stack depth before: {}", human.undo_stack().len()); // 3
    human.undo_blocking();
    println!(
        "after one human undo: {} (depth {})",
        get(),
        human.undo_stack().len()
    );
}
```

Observed output (yrs 0.28.0, rustc 1.99.0, Linux x86_64, release profile):

```text
final: 103
after undo A3: 3
after undo A2: 2
after undo A1: 1
h stack depth before: 3
after one human undo: <none> (depth 0)
```

The last line shows behavior 2 in its strongest form: a single
`undo_blocking()` consumed **all three** remaining stack items, and
because H1 was the first write of the key, the undo removed the key
entirely (`<none>`) instead of restoring any value.

## Context

- Cross-document (remote) edits are *not* affected: an update applied via
  `apply_update` does not enter local undo stacks, and undoing local
  edits after remote merges behaves as expected. The two behaviors above
  are specific to **same-session interleaved writes with per-origin
  managers**.
- We currently treat both behaviors as pinned, stable semantics in our
  test suite (they are deterministic), so this issue is a clarification
  request rather than a bug report. If the behaviors are *not* intended,
  we are happy to file a focused bug report with reduced cases.
