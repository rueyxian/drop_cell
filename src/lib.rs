//! An alternative way of implementing [`Drop`] in Rust.
//!
//! ## Overview
//! This library provides the [`defer!`] macro to defer execution until the end of the stack frame.
//!
//! The [`defer!`] macro emulates [Golang Defer statements][golang_defer].
//!
//! ```rust
//! defer!(println!("world"));
//! println!("hello");
//! return;
//! println!("unreachable");
//! ```
//! Output:
//! ```markdown
//! hello
//! world
//! ````
//!
//! ## Borrowing rules
//! The following code won't compile.
//!
//! ```rust
//!     let mut v = vec![1, 2];
//!     defer!(assert_eq!(v, &[1, 2, 3]));
//! //                    └─ immutable borrow occurs here ──────────┐
//!     v.push(3); //                                               │
//! //  └─ mutable borrow occurs here                               │
//! //                                                              │
//! // immutable borrow will be used at the end of the stack frame ─┘
//! ```
//! We want to run `assert_eq!(v, &[1, 2, 3])` at the end of the stack frame, but it breaks the [borrowing rules][book_mut_ref].
//!
//! To work around, we need to pass `v` into [`defer!`].
//! ```rust
//!     let v = vec![1, 2];
//! //      └─ consumes it ─┐
//! //         ┌────────────┘
//!     defer!(v => assert_eq!(v, &[1, 2, 3]));        
//!     v.push(3);
//! ```
//! ## Example
//! ```rust
//! use drop_cell::defer;
//! use std::io::Write;
//! use std::sync::mpsc;
//!
//! fn main() {
//!     no_arg();
//!     args();
//!     bind();
//! }
//!
//! fn no_arg() {
//!     let (tx, rx) = mpsc::channel();
//!     defer! {
//!         assert_eq!(rx.recv().unwrap(), "hello");
//!         assert_eq!(rx.recv().unwrap(), "world");
//!     };
//!     tx.send("hello").unwrap();
//!     tx.send("world").unwrap();
//! }
//!
//! fn args() {
//!     let (v1, v2) = (vec![], vec![]);
//!     defer! { v1, v2 =>
//!         assert_eq!(v1, b"hello");
//!         assert_eq!(v2, b"world");
//!     }
//!     write!(v1, "hello").unwrap();
//!     write!(v2, "world").unwrap();
//! }
//!
//! fn bind() {
//!     let ss = vec![];
//!     defer! { v @ Some(ss) =>
//!         let v = v.take().unwrap();
//!         assert_eq!(v.as_slice(), ["hello", "world"]);
//!     }
//!     v.as_mut().unwrap().push("hello");
//!     v.as_mut().unwrap().push("world");
//! }
//! ```
//!
//! ## When and when not
//! ###### When to use
//! - When you want a [Finalizer][wiki_finalizer] but reluctant to create a `struct` for it.
//!
//! ###### When NOT to use
//! - When [RAII][wiki_raii] pattern is preferable. e.g. [Lock][wiki_lock] and [Reference Counting][wiki_ref_count].
//! - When the code is written inside a method, using [`defer!`] might complicate the code.
//!
//! [golang_defer]: https://go.dev/ref/spec#Defer_statements
//! [book_mut_ref]: https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html#mutable-references
//! [wiki_finalizer]: https://en.wikipedia.org/wiki/Finalizer
//! [wiki_raii]: https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization  
//! [wiki_lock]: https://en.wikipedia.org/wiki/Lock_(computer_science)
//! [wiki_ref_count]: https://en.wikipedia.org/wiki/Reference_counting

#![no_std]

/// A macro to defer execution until the end of the stack frame.
///
/// No argument:
/// ```rust
/// // body ──────┐
///     defer! { ... };
/// ```
/// Single argument:
/// ```rust
/// //     body ───────┐
/// // ident ────┐     │
///     defer! { v => ... };
/// ```
/// Multiple arguments:
/// ```rust
/// //         body ────────┐
/// //     ident ────┐      │
/// // ident ────┐   │      │
///     defer! { v1, v2 => ... };
/// ```
/// Bindings:
/// ```rust
/// //             body ───────────────────────┐
/// //         expr ──────────────────┐        │
/// //         ident ──────────┐      │        │
/// //     expr ───────────┐   │      │        │
/// //     ident ────┐     │   │      │        │
/// // ident ────┐   │     │   │      │        │
///     defer! { v1, v2 @ foo, v3 @ vec![] => ... };
/// ```
#[macro_export]
macro_rules! defer {
    ( $($i:ident $(@ $e:expr)?),* => $($body:tt)* ) => {
        let $crate::__internal::__defer! { @cell_ident __drop_cell $($i),* } =
            drop_cell::__internal::__DropCell::__new(
                $crate::__internal::__defer! { @pass_args $($i $(@ $e)?),* },
                | $crate::__internal::__defer! { @closure_args_mut $($i),* } | { $($body)* }
            );
        $crate::__internal::__defer! { @bind_args_mut __drop_cell $($i),* }
    };
    ( $($body:tt)* ) => { defer! { => $($body)* } };
}

/// Not intended for use at the client level.
///
/// Use [`defer!`] macro instead.
#[doc(hidden)]
pub mod __internal {
    use core::mem::MaybeUninit;

    /// Not intended for use at the client level.
    ///
    /// Use [`defer!`] macro instead.
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __defer {
        ( @ident_or_expr $i:ident ) => { $i };
        ( @ident_or_expr $i:ident @ $e:expr ) => { $e };

        ( @pass_args ) => { () };
        ( @pass_args $i:ident $(@ $e:expr)? ) => { $crate::__internal::__defer!{ @ident_or_expr $i $(@ $e)? } };
        ( @pass_args $($i:ident $(@ $e:expr)?),+ ) => { ($($crate::__internal::__defer!{ @ident_or_expr $i $(@ $e)? }),+) };

        ( @closure_args_mut ) => { _ };
        ( @closure_args_mut $i:ident ) => { $i };
        ( @closure_args_mut $($i:ident),+ ) => { ($($i),+) };

        ( @cell_ident $cell:ident ) => { $cell };
        ( @cell_ident $cell:ident $($i:ident),+ ) => { mut $cell };

        ( @bind_args_mut $cell:ident ) => { };
        ( @bind_args_mut $cell:ident $i:ident ) => { let $i = $cell.__args_mut(); };
        ( @bind_args_mut $cell:ident $($i:ident),+ ) => { let ($($i),+) = $cell.__args_mut(); };
    }
    pub use __defer;

    /// Not intended for use at the client level.
    ///
    /// Use [`defer!`] macro instead.
    #[doc(hidden)]
    pub struct __DropCell<T, F: FnOnce(&mut T)> {
        args: T,
        f: MaybeUninit<F>,
    }
    impl<T, F: FnOnce(&mut T)> __DropCell<T, F> {
        /// Not intended for use at the client level.
        ///
        /// Use [`defer!`] macro instead.
        #[doc(hidden)]
        pub fn __new(args: T, f: F) -> Self {
            let f = MaybeUninit::new(f);
            Self { args, f }
        }
        /// Not intended for use at the client level.
        ///
        /// Use [`defer!`] macro instead.
        #[doc(hidden)]
        pub fn __args_mut(&mut self) -> &mut T {
            &mut self.args
        }
    }
    impl<T, F: FnOnce(&mut T)> Drop for __DropCell<T, F> {
        fn drop(&mut self) {
            (unsafe { self.f.assume_init_read() })(&mut self.args);
        }
    }
}
