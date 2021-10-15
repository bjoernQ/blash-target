/// This is an oversimplified RTT implementation
/// supporting only one UP channel.
/// It doesn't care about overflowing the output.
/// (It just doesn't check the read_offset).
/// But for normal logging purposes that should be good enough for now.
use core::{fmt::Write, ptr};

static mut BUFFER: [u8; 1024] = [0u8; 1024];

#[repr(C)]
struct Buffer {
    name: *const u8,
    buf_start: *mut u8,
    size_of_buffer: u32,
    /// Position of next data to be written
    write_offset: u32,
    /// Position of next data to be read by host.
    read_offset: u32,
    /// In the segger library these flags control blocking
    /// or non-blocking behavior. This implementation
    /// is non-blocking.
    flags: u32,
}

#[repr(C)]
pub struct ControlBlock {
    /// Initialized to "SEGGER RTT"
    id: [u8; 16],
    /// Initialized to NUM_UP
    up_buffers: i32,
    /// Initialized to NUM_DOWN
    down_buffers: i32,
    /// Note that RTT allows for this to be an array of
    /// "up" buffers of size up_buffers, but here we only have one.
    up: Buffer,
}

static CHANNEL_NAME: &[u8] = b"Terminal\0";

#[no_mangle]
pub static mut _SEGGER_RTT: ControlBlock = ControlBlock {
    id: [0u8; 16],
    up_buffers: 1,
    down_buffers: 0,
    up: Buffer {
        name: &CHANNEL_NAME as *const _ as *const u8,
        buf_start: unsafe { &mut BUFFER as *mut u8 },
        size_of_buffer: 1024,
        write_offset: 0,
        read_offset: 0,
        flags: 0,
    },
};

pub struct Output {}

impl Output {
    pub fn new() -> Output {
        unsafe {
            ptr::copy_nonoverlapping(b"SEGG_" as *const u8, _SEGGER_RTT.id.as_mut_ptr(), 5);

            ptr::copy_nonoverlapping(
                b"ER RTT\0\0\0\0\0\0" as *const u8,
                _SEGGER_RTT.id.as_mut_ptr().offset(4),
                12,
            );
        }

        Output {}
    }

    fn write_str_internal(&mut self, s: &str) -> usize {
        let len = s.len();

        unsafe {
            let buf_len = BUFFER.len() as u32;
            let write_offset = _SEGGER_RTT.up.write_offset as isize;
            let count = usize::min(BUFFER.len() - write_offset as usize - 1, len);

            core::intrinsics::copy_nonoverlapping(
                s.as_ptr() as *const u8,
                BUFFER.as_mut_ptr().offset(write_offset),
                count,
            );

            let mut new_write_off = write_offset as u32 + count as u32;
            if new_write_off >= buf_len - 1 {
                new_write_off = 0;
            }

            _SEGGER_RTT.up.write_offset = new_write_off;

            count
        }
    }
}

impl Write for Output {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut offset = 0;
        let mut to_write = s.len();
        loop {
            let written = self.write_str_internal(&s[offset..]);

            if written == to_write {
                break;
            }

            offset = written;
            to_write -= written;
        }

        Ok(())
    }
}
