pub mod armsscroll;
pub mod expsequence;
pub mod unitstatus;

pub fn ui_init() {
    skyline::install_hook!(crate::ui::unitstatus::unit_status_set_wlvl_hook);
    skyline::install_hook!(crate::ui::unitstatus::help_param_set_wlvl_hook);

    skyline::install_hook!(crate::ui::expsequence::exp_sequence_create_hook);
    skyline::install_hook!(crate::ui::expsequence::exp_sequence_open_hook);
    skyline::install_hook!(crate::ui::expsequence::exp_sequence_tick_hook);
    skyline::install_hook!(crate::ui::expsequence::expset_expwin_setup_unit_hook);
    skyline::install_hook!(crate::ui::expsequence::expset_expwin_update_add_exp_hook);
    skyline::install_hook!(crate::ui::expsequence::expset_expwin_update_unit_hook);

    skyline::install_hook!(crate::ui::armsscroll::map::map_sub_menu_create_bind_hook);
    skyline::install_hook!(crate::ui::armsscroll::map::map_item_helper_can_use_hook);
    crate::ui::armsscroll::map::register_arms_scroll_map_menu_item();
    crate::ui::armsscroll::map::register_map_submenu();

    skyline::install_hook!(crate::ui::armsscroll::sortie::sortie_sub_menu_create_bind_hook);
    crate::ui::armsscroll::sortie::register_arms_scroll_menu_item();
    crate::ui::armsscroll::sortie::register_sortie_submenu();
}