#![feature(lang_items)]
#![feature(asm)]
#![feature(naked_functions)]

#![no_main]
#![no_std]

extern crate cortex_m;

enum TaskState {
    Ready,
    NotReady,
}

struct TaskControlBlock<'a> {
    stack: &'a mut [u32],
    top_of_stack: *mut u32,
    state: TaskState,
}

impl<'a> TaskControlBlock<'a> {
    // TODO is extern "C" required?
    // TODO does the stack need a certain alignment?
    fn new(func: extern fn(), stack: &'a mut [u32]) -> Self {
        let null_ptr = unsafe { 0usize as *const () }; // TODO
        let func_ptr = func as *const extern "C" fn() as *const ();
        let top_of_stack = unsafe {
            initialize_stack(stack.as_mut_ptr(), func_ptr, null_ptr)
        };
        TaskControlBlock {
            stack: stack,
            top_of_stack: top_of_stack,
            state: TaskState::Ready,
        }
    }
}

/// Simulate the stack frame as it would be created by a context switch interrupt.
// TODO should the pointers be all *mut ()?
unsafe fn initialize_stack(top_of_stack: *mut u32, func_ptr: *const (), params: *const ()) -> *mut u32 {
    const START_ADDRESS_MASK: u32 = 0xfffffffe;
    const TASK_RETURN_ADDRESS: u32 = 0; //TODO
    const INITIAL_XPSR: u32 = 0x01000000;

    // Offset added to account for the way the MCU uses the stack on entry/exit of interrupts.
    *(top_of_stack.offset(-1)) = INITIAL_XPSR; // XPSR
    *(top_of_stack.offset(-2)) = func_ptr as usize as u32 & START_ADDRESS_MASK; // PC
    *(top_of_stack.offset(-3)) = TASK_RETURN_ADDRESS; // LR
    *(top_of_stack.offset(-8)) = params as usize as u32; // R0
    return top_of_stack.offset(-16);
}

fn start_scheduler(tasks: &mut [TaskControlBlock]) -> ! {
    loop {
    }
}

fn enable_systick() {
    let mut systick = unsafe { cortex_m::peripheral::syst_mut() };

    // Stop and clear the SysTick
    systick.csr.write(0);
    systick.cvr.write(0);

    const CLOCK_RATE_HZ: u32 = 8_000_000;
    let tick_rate_hz = 1;
    systick.rvr.write((CLOCK_RATE_HZ / tick_rate_hz) - 1);
    const SYSTICK_CLOCK_BIT: u32 = 1 << 2;
    const SYSTICK_INT_BIT: u32 = 1 << 1;
    const SYSTICK_ENABLE_BIT: u32 = 1 << 0;
    systick.csr.write(SYSTICK_CLOCK_BIT | SYSTICK_INT_BIT | SYSTICK_ENABLE_BIT);
}

extern "C" fn task1_func() {
    loop {
    }
}

#[export_name = "_reset"]
pub extern "C" fn main() -> ! {
    let mut task1_stack = [0; 1024];
    let mut task1 = TaskControlBlock::new(task1_func, &mut task1_stack);
    enable_systick();
    loop {}
}

mod lang_items {
    #[lang = "panic_fmt"]
    extern "C" fn panic_fmt() {}
}

mod exception {
    pub extern "C" fn handler() {
        unsafe {
            asm!("bkpt");
        }

        loop {}
    }

    #[naked]
    pub unsafe extern "C" fn pend_sv_handler() {
        asm!("mrs r0, psp");
        asm!("isb");
        asm!("ldr	r3, pxCurrentTCBConst");
        asm!("ldr	r2, [r3]");
        asm!("stmdb r0!, {r4-r11}");
        asm!("str r0, [r2]");
        asm!("stmdb sp!, {r3, r14}");
        asm!("mov r0, 3");
        asm!("msr basepri, r0");
        asm!("bl vTaskSwitchContext");
        asm!("mov r0, #0");
        asm!("msr basepri, r0");
        asm!("ldmia sp!, {r3, r14}");
        asm!("ldr r1, [r3]");
        asm!("ldr r0, [r1]");
        asm!("ldmia r0!, {r4-r11}");
        asm!("msr psp, r0");
        asm!("isb");
        asm!("bx r14");
        asm!(".align 4");
        asm!("pxCurrentTCBConst: .word pxCurrentTCB");
    }

    pub extern "C" fn systick_handler() {
    }

    #[export_name = "_EXCEPTIONS"]
    pub static EXCEPTIONS: [Option<extern "C" fn()>; 14] = [Some(handler), // NMI
                                                            Some(handler), // Hard fault
                                                            Some(handler), // Memmanage fault
                                                            Some(handler), // Bus fault
                                                            Some(handler), // Usage fault
                                                            None, // Reserved
                                                            None, // Reserved
                                                            None, // Reserved
                                                            None, // Reserved
                                                            Some(handler), // SVCall
                                                            None, // Reserved for Debug
                                                            None, // Reserved
                                                            Some(handler), // PendSV
                                                            Some(systick_handler)]; // Systick
}
