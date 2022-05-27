#![feature(start)]
#![feature(inline_const)]
#![feature(thread_local)]

use std::cell::RefCell;
use std::sync::Arc;

use nalgebra::{Matrix2, Vector2};

struct Thread {
    _inner: Arc<()>,
}

struct ThreadInfo {
    // not sure why but the size here matters: Option<u32> doesn't crash
    _guard: Option<(u32, u32)>,
    _thread: Thread,
}

// without #[thread_local], crash does not occur
#[thread_local]
static THREAD_INFO: RefCell<Option<ThreadInfo>> = const { RefCell::new(None) };

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    linker_fix_3ds::init();
    pthread_3ds::init();

    // crash seems to happen here due to "already borrowed", meaning the RefCell
    // was initialized in the wrong state, I guess?
    let _borrow = THREAD_INFO.borrow_mut();

    let v = Vector2::<f32>::new(1.0, 1.0);
    let mul = Matrix2::new(1.0, 0.0, 0.0, 1.0);

    // Unfortunately the matrix Mul implementation is very complicated and
    // not easy to inline for further minimization:
    let _ = mul * v;

    0
}
