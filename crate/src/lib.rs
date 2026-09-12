#![feature(lock_value_accessors)]

extern crate self as wpnexp_lib;

mod game;
mod misc;
mod ui;

pub use crate::misc::statics::VERSION;
pub use crate::misc::statics::OVERWRITE;

pub use crate::misc::misc_init;
pub use crate::game::game_init;
pub use crate::ui::ui_init;

pub use crate::game::data::DEFAULT_FILE_PATH;
pub use crate::game::data::SAVE_FILE_PATH;
pub use crate::game::data::get_item_wexp;
pub use crate::game::data::calculate_side_earned_wexp;
pub use crate::game::data::read_config;

pub use crate::game::wexp::register_weapon_level_calculator_commands;
pub use crate::game::wexp::register_wexp_calculator_commands;