#![feature(start)]
#![feature(inline_const)]
#![feature(thread_local)]

use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use nalgebra::Vector2;

mod shape;

struct ThreadInfo {
    _guard: Option<Range<usize>>,
    _thread: Thread,
}

struct Thread {
    inner: Arc<()>,
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

    let v = Vector2::new(1.0, 1.0);
    let c = shape::Cuboid { half_extents: v };
    let a = Arc::new(c);

    // without this cast present, crash does not occur
    let _b = a as Arc<dyn shape::Shape>;

    0
}
