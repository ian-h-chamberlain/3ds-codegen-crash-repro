#![feature(asm_sym)]
#![feature(start)]
#![feature(thread_local)]

use std::arch::asm;
use std::cell::{RefCell, UnsafeCell};
use std::sync::Arc;

use ctru::services::soc::Soc;

#[derive(Debug)]
pub struct ThreadInfo {
    // not sure why but the size or align here matters: Option<u32> doesn't crash
    _guard: Option<(u32, u32)>,
    _thread: Arc<()>,
}

// without #[thread_local], crash does not occur
#[thread_local]
static LOCAL_STATIC: RefCell<Option<ThreadInfo>> = RefCell::<Option<ThreadInfo>>::new(None);

const MASK_BUF_SIZE: usize = 287;

#[repr(align(32))]
struct MaskBuffer {
    _buffer: [u8; MASK_BUF_SIZE],
}

#[thread_local]
static BUF: UnsafeCell<MaskBuffer> = UnsafeCell::new(MaskBuffer {
    _buffer: [0; MASK_BUF_SIZE],
});

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    linker_fix_3ds::init();
    pthread_3ds::init();

    let mut soc = Soc::init().unwrap();
    let _ = soc.redirect_to_3dslink(true, true);

    let mut offset: u32;
    unsafe {
        asm!(
            "ldr {offset}, ={local_static}(TPOFF)",
            local_static = sym LOCAL_STATIC,
            offset = out(reg) offset,
            // tmp = out(reg) _,
        );
    }
    eprintln!("LOCAL_STATIC(TPOFF) = {offset:#X}");

    unsafe {
        asm!(
            "ldr {offset}, ={local_static}(TPOFF)",
            local_static = sym BUF,
            offset = out(reg) offset,
            // tmp = out(reg) _,
        );
    }
    eprintln!("BUF(TPOFF) = {offset:#X}");

    // We have to access BUF somehow to reproduce:
    dbg!(&BUF);

    if LOCAL_STATIC.try_borrow_mut().is_err() {
        eprintln!("reproduced!");
    } else {
        eprintln!("nothing to see here");
    }

    0
}
