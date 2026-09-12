
use engage::app::BattleInfoSide;
use engage::app::BattleInfoSide_Status;
use engage::app::CalculatorCommand;
use engage::app::CalculatorManager;
use engage::app::Force_Type;
use engage::app::IBattleDetail;
use engage::app::IBattleInfoSide;
use engage::app::IBattleInfoSideMethods;
use engage::app::IBitFieldTemplate32_1Methods;
use engage::app::ICalculatorManagerMethods;
use engage::app::IUnitItem;
use engage::app::IUnitItemMethods;
use engage::app::IUnitMethods;
use engage::app::gamecalculatorcommand::GameCalculatorCommand;
use engage::app::unit::Unit;

use unity::Cast;
use unity::FromIlInstance;
use unity::Il2CppString;
use unity::OptionalMethod;

use crate::game::data::calculate_unit_earned_wexp;
use crate::game::data::get_once_wexp;
use crate::game::data::get_total_wexp;
use crate::game::data::set_once_wexp;
use crate::game::data::set_total_wexp;
use crate::game::data::set_weapon_kind;
use crate::misc::statics::ONCE_WEXP_INDEX;


pub extern "C" fn register_weapon_level_calculator_commands(calculator: CalculatorManager) {
    // We model the new commands after SystemCalculator.WeaponLevelDCommand:
    // SystemCalculator.WeaponLevelDCommand < CalculatorCommand
    let wpn_lvl_d_command = calculator.find_command_2("Ｄ"); // \uFF24
    let wpn_lvl_e_command = CalculatorCommand
        ::instantiate_with_class(wpn_lvl_d_command.get_class().clone_for_override())
        .unwrap();

    let wpn_lvl_ss_command = CalculatorCommand
        ::instantiate_with_class(wpn_lvl_d_command.get_class().clone_for_override())
        .unwrap();

    // We replace vtable slots 4 and 9
    if let Some(method) = wpn_lvl_ss_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("get_Name") {
            method.method_ptr = get_name_wpn_lvl_ss_command as _;
        }
    if let Some(method) = wpn_lvl_ss_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_ss_command as _;
        }
    calculator.add_command(wpn_lvl_ss_command);

    if let Some(method) = wpn_lvl_e_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("get_Name") {
            method.method_ptr = get_name_wpn_lvl_e_command as _;
        }
    if let Some(method) = wpn_lvl_e_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_e_command as _;
        }
    calculator.add_command(wpn_lvl_e_command);


    let wpn_lvl_c_command = calculator.find_command_2("Ｃ"); // \uFF23
    let wpn_lvl_b_command = calculator.find_command_2("Ｂ"); // \uFF22
    let wpn_lvl_a_command = calculator.find_command_2("Ａ"); // \uFF21
    let wpn_lvl_s_command = calculator.find_command_2("Ｓ"); // \uFF33

    // We replace the Get vtable method for each command, ensuring the result matches
    // the modified App.WeaponLevel$$GetKind function.
    if let Some(method) = wpn_lvl_d_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_d_command as _;
        }
    if let Some(method) = wpn_lvl_c_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_c_command as _;
        }
    if let Some(method) = wpn_lvl_b_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_b_command as _;
        }
    if let Some(method) = wpn_lvl_a_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_a_command as _;
        }
    if let Some(method) = wpn_lvl_s_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("Get") {
            method.method_ptr = get_wpn_lvl_s_command as _;
        }
}

// WeaponLevel functions
pub extern "C" fn get_name_wpn_lvl_e_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> Il2CppString {
    "Ｅ".into() // \uFF25
}

pub extern "C" fn get_name_wpn_lvl_ss_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> Il2CppString {
    "ＳＳ".into() // \uFF33 \uFF33
}

pub extern "C" fn get_wpn_lvl_e_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 1.0 }

pub extern "C" fn get_wpn_lvl_d_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 2.0 }

pub extern "C" fn get_wpn_lvl_c_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 3.0 }

pub extern "C" fn get_wpn_lvl_b_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 4.0 }

pub extern "C" fn get_wpn_lvl_a_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 5.0 }

pub extern "C" fn get_wpn_lvl_s_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 6.0 }

pub extern "C" fn get_wpn_lvl_ss_command(
    _this: CalculatorCommand,
    _method_info: OptionalMethod,
) -> f32 { 7.0 }


pub extern "C" fn register_wexp_calculator_commands(calculator: CalculatorManager) {
    // We model the new command after UnitCalculator.WeaponWeightCommand:
    // UnitCalculator.WeaponWeightCommand < GameCalculatorCommand < CalculatorCommand
    let weight_command = calculator.find_command_2("武器の重さ");

    // Unit WEXP totals
    let unit_wexp_command = GameCalculatorCommand
        ::instantiate_with_class(weight_command.get_class().clone_for_override())
        .unwrap();

    // We replace vtable slots 4, 30, 31, 33, and 43.
    if let Some(method) = unit_wexp_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("get_Name") {
            method.method_ptr = get_name_unit_wexp_command as _;
        }
    // if let Some(method) = unit_wexp_command
    //     .get_class()
    //     .raw_mut()
    //     .get_vtable_mut()
    //     .get_mut(30) {
    //         method.method_ptr = get_impl_unit_wexp_command as _;
    //     }
    if let Some(method) = unit_wexp_command
        .get_class()
        .raw_mut()
        .get_vtable_mut()
        .get_mut(31) {
            method.method_ptr = get_impl_side_unit_wexp_command as _;
        }
    if let Some(method) = unit_wexp_command
        .get_class()
        .raw_mut()
        .get_vtable_mut()
        .get_mut(33) {
            method.method_ptr = set_impl_side_unit_wexp_command as _;
        }
    if let Some(method) = unit_wexp_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("IsVisible") {
            method.method_ptr = is_visible_unit_wexp_command as _;
        }
    calculator.add_command(unit_wexp_command);
    
    // Earned WEXP
    let earned_wexp_command = GameCalculatorCommand
        ::instantiate_with_class(weight_command.get_class().clone_for_override())
        .unwrap();

    // We replace vtable slots 4, 30, 31, 33, and 43.
    if let Some(method) = earned_wexp_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("get_Name") {
            method.method_ptr = get_name_earned_wexp_command as _;
        }
    if let Some(method) = earned_wexp_command
        .get_class()
        .raw_mut()
        .get_vtable_mut()
        .get_mut(30) {
            method.method_ptr = get_impl_unit_earned_wexp_command as _;
        }
    if let Some(method) = earned_wexp_command
        .get_class()
        .raw_mut()
        .get_vtable_mut()
        .get_mut(31) {
            method.method_ptr = get_impl_side_earned_wexp_command as _;
        }
    if let Some(method) = earned_wexp_command
        .get_class()
        .raw_mut()
        .get_vtable_mut()
        .get_mut(33) {
            method.method_ptr = set_impl_side_earned_wexp_command as _;
        }
    if let Some(method) = earned_wexp_command
        .get_class()
        .raw_mut()
        .get_virtual_method_mut("IsVisible") {
            method.method_ptr = is_visible_earned_wexp_command as _;
        }
    calculator.add_command(earned_wexp_command);
}


// UnitWEXP command
pub extern "C" fn get_name_unit_wexp_command(
    _this: GameCalculatorCommand,
    _method_info: OptionalMethod,
) -> Il2CppString { "UnitWEXP".into() }

// pub extern "C" fn get_impl_unit_wexp_command(
//     _this: GameCalculatorCommand,
//     unit: Unit,
//     _method_info: OptionalMethod,
// ) -> f32 {
//     if unit.is_null() || unit.get_force_type() != Force_Type::player() {
//         return 0.0
//     }

//     let mut unit_item = unit.get_item_selected();
//     if unit_item.is_null() || unit_item.m_item().is_null() {
//         unit_item = unit.get_item_equipped();
//         if unit_item.is_null() || unit_item.m_item().is_null() {
//             return 0.0
//         }
//     }

//     get_static_wexp(unit.m_person().get_name().to_rust_string(), unit_item.get_kind()) as f32
// }

pub extern "C" fn get_impl_side_unit_wexp_command(
    _this: GameCalculatorCommand,
    side: BattleInfoSide,
    _method_info: OptionalMethod,
) -> f32 {
    if side.is_null() {
        return 0.0
    }

    let unit = side.get_unit();
    if unit.is_null() || unit.get_force_type() != Force_Type::player() {
        return 0.0
    }

    let mut unit_item = side.m_specified_item();
    if unit_item.is_null() {
        return 0.0
    } else if unit_item.m_index() == 0 {
        unit_item = side.m_unit_item();
    }

    if unit_item.is_null() || unit_item.m_item().is_null() {
        unit_item = unit.get_item_selected();
        if unit_item.is_null() || unit_item.m_item().is_null() {
            unit_item = unit.get_item_equipped();
            if unit_item.is_null() || unit_item.m_item().is_null() {
                return 0.0
            }
        }
    }

    get_total_wexp(unit) as f32
}

pub extern "C" fn set_impl_side_unit_wexp_command(
    _this: GameCalculatorCommand,
    side: BattleInfoSide,
    value: f32,
    _method_info: OptionalMethod,
) {
    if side.is_null() {
        return
    }

    let unit = side.get_unit();
    if unit.is_null() || unit.get_force_type() != Force_Type::player() {
        return
    }

    if !side.m_reverse().is_null() && side.m_reverse().get_status().test(BattleInfoSide_Status::rod()) {
        set_total_wexp(unit, 0);
        return
    }

    let mut unit_item = side.m_specified_item();
    if unit_item.is_null() {
        set_total_wexp(unit, 0);
        return
    } else if unit_item.m_index() == 0 {
        unit_item = side.m_unit_item();
    }

    if unit_item.is_null() || unit_item.m_item().is_null() {
        unit_item = unit.get_item_selected();
        if unit_item.is_null() || unit_item.m_item().is_null() {
            unit_item = unit.get_item_equipped();
            if unit_item.is_null() || unit_item.m_item().is_null() {
                set_total_wexp(unit, 0);
                return
            }
        }
    }

    set_total_wexp(unit, value as i32);
    set_weapon_kind(unit_item.get_kind().value);
}

pub extern "C" fn is_visible_unit_wexp_command(
    _this: GameCalculatorCommand,
    _method_info: OptionalMethod,
) -> bool { true }


// EarnedWEXP command
pub extern "C" fn get_name_earned_wexp_command(
    _this: GameCalculatorCommand,
    _method_info: OptionalMethod,
) -> Il2CppString { "WEXP".into() }

pub extern "C" fn get_impl_unit_earned_wexp_command(
    _this: GameCalculatorCommand,
    unit: Unit,
    _method_info: OptionalMethod,
) -> f32 {
    calculate_unit_earned_wexp(unit) as f32
}

pub extern "C" fn get_impl_side_earned_wexp_command(
    _this: GameCalculatorCommand,
    side: BattleInfoSide,
    _method_info: OptionalMethod,
) -> f32 {
    if !*crate::misc::statics::OVERWRITE.lock().unwrap() {
        get_once_wexp(side.m_unit()) as f32
    } else {
        side.m_detail().m_base_params().get(*ONCE_WEXP_INDEX.lock().unwrap()) as f32
    }
}

pub extern "C" fn set_impl_side_earned_wexp_command(
    _this: GameCalculatorCommand,
    side: BattleInfoSide,
    value: f32,
    _method_info: OptionalMethod,
) {
    if !*crate::misc::statics::OVERWRITE.lock().unwrap() {
        set_once_wexp(side.m_unit(), value as i32);
    } else {
        side.m_detail().m_base_params().set(*ONCE_WEXP_INDEX.lock().unwrap(), value as i32);
    }
}

pub extern "C" fn is_visible_earned_wexp_command(
    _this: GameCalculatorCommand,
    _method_info: OptionalMethod,
) -> bool { true }