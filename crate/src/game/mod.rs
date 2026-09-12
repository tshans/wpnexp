
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
    crate::game::weaponlevel::install_wlvl_hooks();
    crate::game::rewind::install_rewind_hooks();

    // Plugins which use OVERWRITE must implement WEXP functionality in their own BattleDetail$$CalcBattle hook
    if !*crate::misc::statics::OVERWRITE.lock().unwrap() {
        crate::game::battledetail::install_detail_hooks();
    }
}