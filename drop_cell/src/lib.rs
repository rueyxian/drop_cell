pub mod cell {
    use std::mem::MaybeUninit;
    pub struct DropCell<T, F: FnOnce(&mut T)> {
        args: T,
        f: MaybeUninit<F>,
    }
    impl<T, F: FnOnce(&mut T)> DropCell<T, F> {
        pub fn new(args: T, f: F) -> Self {
            let f = MaybeUninit::new(f);
            Self { args, f }
        }
        pub fn args_mut(&mut self) -> &mut T {
            &mut self.args
        }
    }
    impl<T, F: FnOnce(&mut T)> Drop for DropCell<T, F> {
        fn drop(&mut self) {
            (unsafe { self.f.assume_init_read() })(&mut self.args);
        }
    }
}

pub mod macros {
    pub use drop_cell_macros::*;
}
#[doc(hidden)]
pub use macros::*;
