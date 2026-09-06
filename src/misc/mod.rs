pub mod calculator;
pub mod event;
pub mod service;
pub mod statics;

use unity::Cast;

// Helper function
pub fn option_null<T: Cast>(this: T) -> Option<T> {
    if this.is_null() {
        None
    } else {
        Some(this)
    }
}

pub fn misc_init() {
    skyline::install_hook!(crate::misc::calculator::add_command_hook);
    crate::misc::statics::init_statics();
}