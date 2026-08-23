#![feature(lock_value_accessors)]

use std::sync::{Mutex, OnceLock};

use engage::app::CalculatorManager;
use unity::{Cast, OptionalMethod};

use crate::game::data::{
    BattleWEXP, UnitWexpData, WexpData, WexpRewindLedger, read_default_dyn_unit_data, read_default_unit_data, read_weapon_data,
};

pub mod game;
pub mod misc;
pub mod ui;

// I like the current WEXP setup:
//     Total WEXP = Static + Dynamic + Job
//     Earned WEXP = Weapon + Unit's Innate Proficiency + Emblem Favored Weapon

// Add WEXP_DATA values to the Fer/Grt line of WdwItemHelp/HelpParamSetter? Shouldn't be too hard...
// though this will have to wait till the plugin is merged with crtdmg. Maybe I can use a cobapi Service?

// BattleGrow -> GainExp (C) + End
//  - UnitGrowSequence -> Prepare (C) + GainExp (C) + Label (0) + CheckLevelUp (C) + LevelUp (C)
//                          + Label (1) + Label (2) + CheckClassChange (C) + ClassChange (C)
//                          + SetWeapon (C) + Label (C) + Label (5) + End
//      - ExpSequence -> WaitLoad (T) + Open (C) + Yield + WaitAnime (T) + SoundStart (C)
//                          + Tick (T) + SoundStop (C) + Yield + WaitAnime (T) + Release (C) + End
//      - LevelUpSequence -> Prepare (C) + Effect (C) + IsLoadingRes (W) + Open (C) + Label (0)
//                              + Yield + WaitTime + CheckParamChange (C) + CalcTalkMid (C) + Reflect (C)
//                              + Talk (C) + KeyWait (T) + Yield + LearnJobSkill (C) + WaitAnime (C)
//                              + Release (C) + ReloadActor (C) + WaitReloadActor (T) + End


// I've successfully shifted the WdwExpRoot prefab changes from a function hook to an Event Listener...
// What else can be moved over? (Obviously, I'll want to repeat this process for the prefab changes in crtdmg.)

// MainSequence.Label => ChapterSave/SaveDataLoad 15/18
// MainMenuSequence.Label => SaveDataCopy/Delete/ToStartGame/ToContinueGame 20/21/28/29

// RewindSequence.Label => ExecuteRewind 2


// Investigate these to replace the ACall hooks?
// SaveDataMenuSequence.Label => 
//          LoadMenu/SaveMenuFromMenu/SaveMenuFromPeriod/SaveMenuFromEnding/SuspendMenu/CopyMenu/DeleteMenu


// Finish up armsscroll.rs
//  - It works!



// Helper function
pub fn option_null<T: Cast>(this: T) -> Option<T> {
    if this.is_null() {
        None
    } else {
        Some(this)
    }
}


// This is set statically during plugin launch, loading WEXP values for every weapon from a preset config JSON.
pub static WEXP_DATA: OnceLock<WexpData> = OnceLock::<WexpData>::new();

// These are set to a default value on plugin launch and their contents are replaced with (written as) data from (in)
// a modular config JSON on game load (save). Their values change dynamically throughout gameplay.
pub static UNIT_WEXP_DATA: Mutex<Option<UnitWexpData>> = Mutex::<Option<UnitWexpData>>::new(None);
pub static TEMP_UNIT_WEXP_DATA: Mutex<Option<UnitWexpData>> = Mutex::<Option<UnitWexpData>>::new(None);
pub static WEXP_LEDGER: Mutex<Option<WexpRewindLedger>> = Mutex::<Option<WexpRewindLedger>>::new(None);

pub static BATTLE_WEXP: Mutex<Option<BattleWEXP>> = Mutex::<Option<BattleWEXP>>::new(None);

#[unity::hook("App", "UnitCalculator", "AddCommand")]
fn add_command_hook(calculator: CalculatorManager, method_info: OptionalMethod) {
    call_original!(calculator, method_info);
    crate::game::wexp::register_weapon_level_calculator_commands(calculator);
    crate::game::wexp::register_wexp_calculator_commands(calculator);
}

#[skyline::main(name = "wpnexp")]
pub fn main() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };
        let err_msg = format!(
            "wpnexp has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        skyline::error::show_error(
            420,
            "wpnexp has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));


    skyline::install_hook!(add_command_hook);
    skyline::install_hook!(crate::game::save::game_user_restart_set_target_hook);
    skyline::install_hook!(crate::game::save::load_confirm_a_call_hook);
    skyline::install_hook!(crate::game::save::save_confirm_a_call_hook);
    skyline::install_hook!(crate::game::save::suspend_confirm_a_call_hook);
    skyline::install_hook!(crate::game::save::delete_confirm_a_call_hook);
    skyline::install_hook!(crate::game::save::copy_confirm_a_call_hook);
    skyline::install_hook!(crate::game::save::game_save_data_util_delete_all_hook);
    skyline::install_hook!(crate::game::save::map_auto_save_hook);
    skyline::install_hook!(crate::game::save::main_auto_save_hook);

    skyline::install_hook!(crate::game::hooks::class_change_get_disp_wlvl_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_weapon_level_hook);
    skyline::install_hook!(crate::game::hooks::unit_update_state_impl_hook);
    skyline::install_hook!(crate::game::hooks::unit_item_equip_hook);
    skyline::install_hook!(crate::game::hooks::unit_item_equip_options_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_breakable_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_enemy_engage_attack_hook);
    skyline::install_hook!(crate::game::hooks::unit_set_optimal_weapon_hook);
    skyline::install_hook!(crate::game::hooks::unit_item_add_on_dlc_evil_hook);
    skyline::install_hook!(crate::game::hooks::unit_equipable_item_add);
    skyline::install_hook!(crate::game::hooks::unit_has_equipable_item_hook);
    skyline::install_hook!(crate::game::hooks::unit_has_equipable_item_range_hook);
    skyline::install_hook!(crate::game::hooks::unit_has_equipable_item_kind_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_item_equip_uire_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_item_equip_idrw_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_item_equip_ire_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_use_cannon_xz_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_use_cannon_terrain_hook);
    skyline::install_hook!(crate::game::hooks::unit_next_item_equip_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_engage_equip_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_attack_range_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_attack_range_item_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_rod_range_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_rod_range_item_hook);
    skyline::install_hook!(crate::game::hooks::unit_get_revenge_weapon_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_item_use_hook);
    skyline::install_hook!(crate::game::hooks::unit_can_item_use_target_hook);
    skyline::install_hook!(crate::game::hooks::unit_is_draw_active_color);

    skyline::install_hook!(crate::game::weaponlevel::weapon_level_get_kind_hook);

    skyline::install_hook!(crate::game::battledetail::battle_detail_calc_battle_hook);

    skyline::install_hook!(crate::game::rewind::rewind_class_change_hook);
    skyline::install_hook!(crate::game::rewind::rewind_preview_decide_hook);
    skyline::install_hook!(crate::game::rewind::game_user_data_set_rewind_hook);

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

    cobapi::register_system_event_handler(crate::misc::event::listener);


    // We allocate memory for each Static dataset by initializing the corresponding safety primitive with a default value.
    // The general Wexp data is identical for every game, so it is initialized in a OnceLock.
    let wexp_data = read_weapon_data().unwrap();
    WEXP_DATA.set(wexp_data).expect("Failed to set WeaponExpData as static.");

    // The "Static" data's default is a hashmap filled with a unit key and a value slice containing base WEXP values for
    // every playable character, initialized as a Mutex. This memory allocation is only modified on chapter completion via
    // the autosave function or when the user saves their game.
    let default_unit_data = read_default_unit_data().ok();
    UNIT_WEXP_DATA.set(default_unit_data).expect("Failed to set UnitWexpData's default data.");

    // The "Dynamic" data's default is a hashmap filled with a unit key and a value slice of zeroes for every playable
    // character, initialized as a Mutex. This memory allocation will be dynamically modified during typical gameplay,
    // resetting to the default on chapter completion via autosave or manual game saves.
    let default_dyn_unit_data = read_default_dyn_unit_data().ok();
    TEMP_UNIT_WEXP_DATA.set(default_dyn_unit_data).expect("Failed to set TempUnitWexpData's default data.");

    // The Ledger's default is an empty hashmap, initialized as a Mutex. When game events occur that modify the
    // dynamic data, a copy of the that data is saved to the ledger. These copies are used to populate the dynamic
    // data with the correct values after the user initiates a time crystal rewind. The ledger is cleared upon chapter
    // completion or when the game is manually saved.
    let default_ledger = Some(WexpRewindLedger::new());
    WEXP_LEDGER.set(default_ledger).expect("Failed to set WexpLedger's default data.");

    // The Battle's default is an empty hashmap, initialized as a Mutex. This data is used to perform post-battle
    // WEXP calculations, particularly the gained exp/sp/wexp animation.
    let default_battle = Some(BattleWEXP::new());
    BATTLE_WEXP.set(default_battle).expect("Failed to set BattleWexp's default data.");
}