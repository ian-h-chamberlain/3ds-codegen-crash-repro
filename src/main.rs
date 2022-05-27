#![feature(start)]
#![feature(inline_const)]
#![feature(thread_local)]

use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use nalgebra::{Matrix2, Vector2};

struct ThreadInfo {
    _guard: Option<Range<usize>>,
    _thread: Thread,
}

struct Thread {
    inner: Arc<()>,
}

pub trait Shape {
    fn compute_aabb(&self);
}

impl Shape for Vector2<f32> {
    fn compute_aabb(&self) {
        let mul = Matrix2::new(1.0, 0.0, 0.0, 1.0);

        // Unfortunately the matrix Mul implementation is very complicated and
        // not easy to inline for further minimization:
        let _ = mul * self;
    }
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // without #[thread_local], crash does not occur
    #[thread_local]
    static THREAD_INFO: RefCell<Option<ThreadInfo>> = const { RefCell::new(None) };

    linker_fix_3ds::init();
    pthread_3ds::init();

    // crash seems to happen here due to "already borrowed":
    let mut info = THREAD_INFO.borrow_mut();
    info.get_or_insert_with(|| ThreadInfo {
        _guard: None,
        _thread: Thread {
            inner: Arc::new(()),
        },
    });

    let v = Vector2::<f32>::new(1.0, 1.0);
    let a = Arc::new(v);

    // without this cast present, crash does not occur
    let _b = a as Arc<dyn Shape>;

    0
}
