#![feature(start)]
#![feature(thread_local)]

use ctru::services::soc::Soc;

#[repr(align(4))]
pub struct Align4([u8; 3]);

const INIT: [u8; 3] = [0, 1, 2];

#[thread_local]
static BUF_4: Align4 = Align4(INIT);

#[repr(align(16))]
struct Align16([u8; 3]);

#[thread_local]
static BUF_16: Align16 = Align16([0; 3]);

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    linker_fix_3ds::init();
    pthread_3ds::init();

    let mut soc = Soc::init().unwrap();
    let _ = soc.redirect_to_3dslink(true, true);

    // We have to access BUF somehow to reproduce:
    dbg!(&BUF_16.0);

    if BUF_4.0 == INIT {
        eprintln!("nothing to see here");
    } else {
        eprintln!("reproduced!");
        dbg!(BUF_4.0);
    }

    0
}
