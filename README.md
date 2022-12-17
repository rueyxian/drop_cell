# drop_cell

An alternative way of implementing `Drop` in Rust.

## Overview

This library provides the `defer!` macro to defer execution until the end of the stack frame.

The `defer!` macro emulates [Golang Defer statements](https://go.dev/ref/spec#Defer_statements).

```rust
drop_cell::defer!(println!("world"));
println!("hello");
return;
println!("unreachable");
```

Output:

```markdown
hello
world
```

## Borrowing rules

The following code won't compile.

```rust
    let mut v = vec![1, 2];
    defer!(assert_eq!(v, &[1, 2, 3]));
//                    ┗━ immutable borrow occurs here ━━━━━━━━━━┓
    v.push(3); //                                               ┃
//  ┗━ mutable borrow occurs here                               ┃
//                                                              ┃
// immutable borrow will be used at the end of the stack frame ━┛
```

We want to run `assert_eq!(v, &[1, 2, 3])` at the end of the stack frame, but it breaks the [borrowing rules]([References and Borrowing - The Rust Programming Language](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html#mutable-references).

To work around, we need to pass `v` into `defer!`.

```rust
    let v = vec![1, 2];
//      ┗━ consumes it ━┓
//         ┏━━━━━━━━━━━━┛
    defer!(v => assert_eq!(v, &[1, 2, 3]));        
    v.push(3);
```

## Example

```rust
use drop_cell::defer;
use std::io::Write;
use std::sync::mpsc;

fn main() {
    no_arg();
    args();
    bind();
}

fn no_arg() {
    let (tx, rx) = mpsc::channel();
    defer! {
        assert_eq!(rx.recv().unwrap(), "hello");
        assert_eq!(rx.recv().unwrap(), "world");
    };
    tx.send("hello").unwrap();
    tx.send("world").unwrap();
}

fn args() {
    let (v1, v2) = (vec![], vec![]);
    defer! { v1, v2 =>
        assert_eq!(v1, b"hello");
        assert_eq!(v2, b"world");
    }
    write!(v1, "hello").unwrap();
    write!(v2, "world").unwrap();
}

fn bind() {
    let ss = vec![];
    defer! { v @ Some(ss) =>
        let v = v.take().unwrap();
        assert_eq!(v.as_slice(), ["hello", "world"]);
    }
    v.as_mut().unwrap().push("hello");
    v.as_mut().unwrap().push("world");
}
```

## When and when not

###### When to use

- When you want a [Finalizer](https://en.wikipedia.org/wiki/Finalizer) but reluctant to create a `struct` for it.

###### When NOT to use

- When [RAII]([Resource acquisition is initialization - Wikipedia](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)) pattern is preferable. e.g. Lock Guard, Reference Counting.
- When the code is written inside a method, using `defer!` might complicate the code.
