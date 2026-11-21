/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * panic.rs
 *
 * Description:
 * Handles panics and faults in RAM that survives resets
 **************************************************************/

 /* Public Crates */
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::ptr::{addr_of, addr_of_mut};
use log::error;

/* Constants */
const PANIC_MAGIC: u32 = 0x5041_4E43;
const MSG_LEN: usize = 240;

#[repr(C)]
struct PanicRecord {
    magic: u32,
    len: u32,
    msg: [u8; MSG_LEN],
}

// .uninit is not zeroed at boot, so this survives a reset
#[unsafe(link_section = ".uninit.panic_record")]
static mut PANIC_RECORD: MaybeUninit<PanicRecord> = MaybeUninit::uninit();

/* Writes formatted text into a fixed buffer */
struct Cursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}
impl Write for Cursor<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let n = s.len().min(self.buf.len() - self.pos);
        self.buf[self.pos..self.pos + n].copy_from_slice(&s.as_bytes()[..n]);
        self.pos += n;
        Ok(())
    }
}

/* Stores message, marks record valid, resets chip */
fn record(args: core::fmt::Arguments) -> ! {
    unsafe {
        let rec = addr_of_mut!(PANIC_RECORD) as *mut PanicRecord;
        let mut c = Cursor {
            buf: &mut *addr_of_mut!((*rec).msg),
            pos: 0,
        };
        let _ = c.write_fmt(args);
        (*rec).len = c.pos as u32;
        (*rec).magic = PANIC_MAGIC;
    }
    cortex_m::peripheral::SCB::sys_reset()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    record(format_args!("{}", info))
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    record(format_args!(
        "HardFault pc={:#010x} lr={:#010x}",
        ef.pc(),
        ef.lr()
    ))
}

/* Call once at boot once logger is running */
pub fn report_last_panic() {
    unsafe {
        let rec = addr_of_mut!(PANIC_RECORD) as *mut PanicRecord;
        if core::ptr::read_volatile(addr_of!((*rec).magic)) == PANIC_MAGIC {
            let len = (core::ptr::read_volatile(addr_of!((*rec).len)) as usize).min(240);
            let msg = &*addr_of!((*rec).msg);
            if let Ok(s) = core::str::from_utf8(&msg[..len]) {
                error!("LAST PANIC: {}", s);
            }
            core::ptr::write_volatile(addr_of_mut!((*rec).magic), 0);
        }
    }
}
