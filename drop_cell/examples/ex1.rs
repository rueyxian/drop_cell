#![allow(unreachable_code)]

use drop_cell::defer;
fn main() {
    none();
    one();
    many();
    binding();
}

fn none() {
    let (tx, rx) = std::sync::mpsc::channel();
    defer! {
        tx.send(3).unwrap();
        tx.send(4).unwrap();
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 3);
        assert_eq!(rx.recv().unwrap(), 4);
    }
    tx.send(1).unwrap();
    return;
    tx.send(2).unwrap();
}

fn one() {
    let v = vec![];
    defer!(v => {
        v.push(3);
        assert_eq!(v.as_slice(), &[1, 3]);
    });
    v.push(1);
    return;
    v.push(2);
}

fn many() {
    let v1 = vec![];
    let v2 = vec![];
    defer!((v1, v2) => {
        v1.push(3);
        v2.push(3);
        assert_eq!(v1.as_slice(), &[1, 3]);
        assert_eq!(v2.as_slice(), &[1, 3]);
    });
    v1.push(1);
    v2.push(1);
    return;
    v1.push(2);
    v2.push(2);
}

fn binding() {
    let v1 = vec![];
    let vvvv = vec![];
    defer!((v1, v2 @ vvvv, v3 @ vec![]) => {
        v1.push(3);
        v2.push(3);
        v3.push(3);
        assert_eq!(v1.as_slice(), &[1, 3]);
        assert_eq!(v2.as_slice(), &[1, 3]);
        assert_eq!(v3.as_slice(), &[1, 3]);
    });
    v1.push(1);
    v2.push(1);
    v3.push(1);
    return;
    v1.push(2);
    v2.push(2);
    v3.push(2);
}
