#![allow(unreachable_code)]

use drop_cell::defer;
fn main() {
    none();
    single();
    multiple();
    bindings();
}

fn none() {
    let (tx, rx) = std::sync::mpsc::channel();
    defer! {
        tx.send("world").unwrap();
        assert_eq!(rx.recv().unwrap(), "hello");
        assert_eq!(rx.recv().unwrap(), "world");
    }
    tx.send("hello").unwrap();
    return;
    tx.send("unreachable").unwrap();
}

fn single() {
    let v = vec![];
    defer! { v =>
        v.push("world");
        assert_eq!(v.as_slice(), &["hello", "world"]);
    };
    v.push("hello");
    return;
    v.push("unreachable");
}

fn multiple() {
    let v1 = vec![];
    let v2 = vec![];
    defer! { v1, v2 =>
        v1.push("world");
        v2.push("world");
        assert_eq!(v1.as_slice(), &["hello", "world"]);
        assert_eq!(v2.as_slice(), &["hello", "world"]);
    };
    v1.push("hello");
    v2.push("hello");
    return;
    v1.push("unreachable");
    v2.push("unreachable");
}

fn bindings() {
    let v1 = vec![];
    let foo = vec![];
    defer! { v1, v2 @ foo, v3 @ vec![] =>
        v1.push("world");
        v2.push("world");
        v3.push("world");
        assert_eq!(v1.as_slice(), &["hello", "world"]);
        assert_eq!(v2.as_slice(), &["hello", "world"]);
        assert_eq!(v3.as_slice(), &["hello", "world"]);
    };
    v1.push("hello");
    v2.push("hello");
    v3.push("hello");
    return;
    v1.push("unreachable");
    v2.push("unreachable");
    v3.push("unreachable");
}
