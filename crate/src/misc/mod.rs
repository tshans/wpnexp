pub mod api;
pub mod calculator;
pub mod event;
//pub mod service; Currently unused.
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

pub fn misc_init(overwrite: bool, index: usize, unit_apt: bool, god_apt: bool) {
    crate::misc::statics::init_statics(overwrite, index, unit_apt, god_apt);

    if !*crate::misc::statics::OVERWRITE.lock().unwrap() {
        skyline::install_hook!(crate::misc::calculator::add_command_hook);
    } else {
        crate::misc::api::init_api();
    }

    crate::misc::event::register_listener();
}