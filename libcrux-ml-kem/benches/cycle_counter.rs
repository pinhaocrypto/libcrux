use std::sync::Once;

extern "C" {
    fn enable_cyclecounter();
    fn disable_cyclecounter();
    fn get_cyclecounter() -> u64;
}

static INIT: Once = Once::new();

pub fn init_cycle_counter() {
    INIT.call_once(|| unsafe {
        enable_cyclecounter();
    });
}

pub fn read_cycles() -> u64 {
    unsafe { get_cyclecounter() }
}

pub fn cleanup_cycle_counter() {
    unsafe {
        disable_cyclecounter();
    }
}