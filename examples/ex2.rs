use drop_cell::defer;

fn main() {
    std::panic::set_hook(Box::new(|_info| println!("PANIC!!")));
    let stream = stream();
    while let Msg::Done(i) = stream.recv().unwrap() {
        println!("recv: {:?}", i);
    }
    println!("graceful end");
}

fn stream() -> std::sync::mpsc::Receiver<Msg> {
    let (tx, rx) = std::sync::mpsc::channel::<Msg>();
    let _ = std::thread::spawn(move || {
        defer! {
            tx.send(Msg::Kill).unwrap();
        }
        for i in 0..1000 {
            let done = emulate_panic(i);
            tx.send(done).unwrap();
        }
    });
    rx
}

enum Msg {
    Done(usize),
    Kill,
}

fn emulate_panic(i: usize) -> Msg {
    std::thread::sleep(std::time::Duration::from_millis(50));
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    if epoch % 19 == 0 {
        panic!();
    }
    Msg::Done(i)
}
