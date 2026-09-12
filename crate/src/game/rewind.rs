
use engage::app::Force_Type;
use engage::app::GameUserData;
use engage::app::IGameUserData;
use engage::app::IGameVariable;
use engage::app::IMapHistory_Base_1;
use engage::app::IMapHistory_Rewind;
use engage::app::IUnitMethods;
use engage::app::MapHistory_Mode;
use engage::app::MapHistory_Rewind;
use engage::app::Unit;

use engage::system::collections::generic::IDictionary_2Methods;
use unity::Cast;
use unity::OptionalMethod;

use crate::game::data::read_default_dyn_unit_data;
use crate::misc::statics::TEMP_UNIT_WEXP_DATA;
use crate::misc::statics::WEXP_LEDGER;


pub(crate) fn write_to_ledger(command_num: i32) {
    println!("The ledger has been appended due to wexp gain on count {}.", command_num);

    let mut guard = TEMP_UNIT_WEXP_DATA.lock().unwrap();
    let temp_wexp = match guard.as_mut() {
        Some(data) => data,
        None => panic!("Dynamic unit wexp data failed to initialize."),
    };

    let mut led_guard = WEXP_LEDGER.lock().unwrap();
    match led_guard.as_mut() {
        Some(data) => {
            data.ledger.insert(command_num, temp_wexp.clone());
        },
        None => panic!("WEXP ledger failed to initialize."),
    };
}

pub(crate) fn read_from_ledger(command_num: i32) {
    println!("The ledger is to be reset to count {}.", command_num);

    let mut led_guard = WEXP_LEDGER.lock().unwrap();
    match led_guard.as_mut() {
        Some(data) => {
            let mut count = 0;
            while command_num - count >= 0 {
                println!("The current count is {}.", command_num - count);
                if let Some(temp_data) = data.ledger.get(&(command_num - count)) {
                    TEMP_UNIT_WEXP_DATA.set(Some(temp_data.clone())).expect("Failed to set rewind TempUnitWexpData.");

                    data.ledger.retain(|&k, _| k <= command_num - count);
    
                    println!("The ledger has been reset to count {}.", command_num - count);

                    return
                } else {
                    count += 1;
                }
            }
            println!("No ledger entries found prior to count {}, resetting dynamic and ledger data.", command_num);

            let default_dyn_unit_data = read_default_dyn_unit_data().ok();
            TEMP_UNIT_WEXP_DATA.set(default_dyn_unit_data).expect("Failed to set TempUnitWexpData's default data on rewind.");

            println!("Dynamic data reset. {}", command_num);

            data.ledger.clear();

            println!("Ledger data reset. {}", command_num);
        },
        None => panic!("WEXP ledger failed to initialize."),
    }
}

#[unity::hook("App", "MapHistory.Rewind", "ClassChange")] // 0x7102716410
fn rewind_class_change_hook(
    this: MapHistory_Rewind,
    unit: Unit,
    method_info: OptionalMethod,
) {
    if this.m_mode() == MapHistory_Mode::write() {
        let count = this.m_num_command() + 1;

        if !unit.is_null() && unit.get_force_type() == Force_Type::player() {
            write_to_ledger(count);

            println!("The ledger has been appended due to a class change on count {}.", count);
        }
    }

    call_original!(this, unit, method_info)
}

#[unity::hook("App", "MapHistory.Rewind", "PreviewDecide")] // 0x710270D860
fn rewind_preview_decide_hook(
    this: MapHistory_Rewind,
    method_info: OptionalMethod,
) {
    read_from_ledger(this.m_preview_index());

    call_original!(this, method_info)
}

// This is required to ensure that dynamic wexp data is not lost for experience gained on a map prior to obtaining the time
// crystal. For example, the time crystal is obtained at the start of chapter 4 turn 2 in vanilla. Without the following
// hook, rewinding to the start of turn 2 would set the dynamic data to zero, losing anything gained during turn 1.
#[unity::hook("App", "GameUserData", "set_IsRewindEnable")] // 0x7102516760
fn game_user_data_set_rewind_hook(
    this: GameUserData,
    value: bool,
    method_info: OptionalMethod,
) {
    if value && this.m_variable().m_dictionary().get_item("G_所持_IID_竜の時水晶".into()).number == 0 {
        write_to_ledger(0);
    }

    call_original!(this, value, method_info)
}

pub fn install_rewind_hooks() {
    skyline::install_hook!(crate::game::rewind::rewind_class_change_hook);
    skyline::install_hook!(crate::game::rewind::rewind_preview_decide_hook);
    skyline::install_hook!(crate::game::rewind::game_user_data_set_rewind_hook);
}