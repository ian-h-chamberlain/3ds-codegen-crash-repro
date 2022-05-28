#![feature(start)]
#![feature(inline_const)]
#![feature(thread_local)]

use std::cell::RefCell;
use std::sync::Arc;

use ctru::services::soc::Soc;
use nalgebra::{Matrix2, Vector2};

#[derive(Debug)]
struct ThreadInfo {
    // not sure why but the size here matters: Option<u32> doesn't crash
    _guard: Option<(u32, u32)>,
    _thread: Arc<()>,
}

// without #[thread_local], crash does not occur
#[thread_local]
static THREAD_INFO: RefCell<Option<ThreadInfo>> = const { RefCell::new(None) };

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    linker_fix_3ds::init();
    pthread_3ds::init();

    let mut soc = Soc::init().unwrap();
    let _ = soc.redirect_to_3dslink(true, true);

    let v = Vector2::<f32>::new(1.0, 1.0);
    let mul = Matrix2::new(1.0, 0.0, 0.0, 0.0);

    // Unfortunately the matrix Mul implementation is very complicated and
    // not easy to inline for further minimization:
    let _ = mul * v;

    if THREAD_INFO.try_borrow_mut().is_err() {
        eprintln!("reproduced!");
    } else {
        eprintln!("nothing to see here");
    }

    0
}
