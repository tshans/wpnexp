pub mod battledetail;
pub mod data;
pub mod hooks;
pub mod rewind;
pub mod save;
pub mod weaponlevel;
pub mod wexp;


pub fn game_init() {
    crate::game::save::install_save_hooks();

    crate::game::hooks::install_hook_hooks();

    skyline::install_hook!(crate::game::weaponlevel::weapon_level_get_kind_hook);

    skyline::install_hook!(crate::game::battledetail::battle_detail_calc_battle_hook);

    skyline::install_hook!(crate::game::rewind::rewind_class_change_hook);
    skyline::install_hook!(crate::game::rewind::rewind_preview_decide_hook);
    skyline::install_hook!(crate::game::rewind::game_user_data_set_rewind_hook);
}