extern crate core;

use core::fmt::{Write, self};
use core::panic::PanicInfo;

use libc;

#[derive(Default)]
struct ErStderr;

impl Write for ErStderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        libc::write(libc::STDERR_FILENO, s as *const str as *const libc::c_void, s.len());
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut host_stderr = ErStderr::default();

    writeln!(host_stderr, "{}", info).ok();

    libc::exit(1);
}
