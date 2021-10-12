#![no_std]

use core::mem::MaybeUninit;

#[repr(C)]
struct TriggerBacktrace {
    trigger: u8,
    mepc: u32,
    sp: u32,
}

#[used]
#[no_mangle]
static mut _BLASH_BACKTRACE_TRIGGER: TriggerBacktrace = TriggerBacktrace {
    trigger: 0,
    mepc: 0,
    sp: 0,
};

pub static mut OUT: MaybeUninit<jlink_rtt::Output> = MaybeUninit::uninit();

pub fn out() -> &'static mut dyn core::fmt::Write {
    unsafe { &mut *(OUT.as_mut_ptr()) }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        riscv::interrupt::free(|_|{
            let writer = $crate::out();
            write!(writer, $($arg)*).ok();
            write!(writer, "\r\n").ok();
        });
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        riscv::interrupt::free(|_|{
            let writer = $crate::out();
            write!(writer, $($arg)*).ok();
        });
    };
}

#[macro_export]
macro_rules! init_print {
    () => {
        #[allow(unused_unsafe)]
        unsafe {
            let mut output = jlink_rtt::Output::new();
            unsafe {
                *($crate::OUT.as_mut_ptr()) = output;
            }
        }
    };
}

#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `println!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", core::file!(), core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[cfg(not(test))]
#[cfg(feature = "panic_backtrace")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC! {:?}", info);

    for _ in 0..50000 {}

    unsafe {
        _BLASH_BACKTRACE_TRIGGER.trigger = 1;
    }
    loop {}
}

#[cfg(feature = "exception_backtrace")]
#[export_name = "ExceptionHandler"]
fn custom_exception_handler(_trap_frame: &bl602_hal::interrupts::TrapFrame) -> ! {
    let mepc = riscv::register::mepc::read();
    let code = riscv::register::mcause::read().code() & 0xff;
    let meaning = match code {
        0 => "Instruction address misaligned",
        1 => "Instruction access fault",
        2 => "Illegal instruction",
        3 => "Breakpoint",
        4 => "Load address misaligned",
        5 => "Load access fault",
        6 => "Store/AMO address misaligned",
        7 => "Store/AMO access fault",
        8 => "Environment call from U-mode",
        9 => "Environment call from S-mode",
        10 => "Reserved",
        11 => "Environment call from M-mode",
        12 => "Instruction page fault",
        13 => "Load page fault",
        14 => "Reserved",
        15 => "Store/AMO page fault",
        _ => "Unknown",
    };
    println!("exception code {} ({}) at {:x}", code, meaning, mepc);

    for _ in 0..50000 {}
    unsafe {
        _BLASH_BACKTRACE_TRIGGER.trigger = 1;
        _BLASH_BACKTRACE_TRIGGER.mepc = riscv::register::mepc::read() as u32;
        _BLASH_BACKTRACE_TRIGGER.sp = _trap_frame.sp as u32;
    }
    loop {}
}
