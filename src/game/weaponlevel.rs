
use engage::app::Force_Type;
use engage::app::IJobDataMethods;
use engage::app::IUnit;
use engage::app::IUnitMethods;
use engage::app::ItemData_Kinds;
use engage::app::Unit;
use engage::app::WeaponLevel_Kind;

use unity::{Il2CppString, OptionalMethod};

use crate::game::data::calculate_unit_wexp_kind;
use crate::game::data::set_new_wlvl;


pub static WEXP_KINDS: [&str; 8] = ["N", "E", "D", "C", "B", "A", "S", "SS"];
pub static WEXP_VALUES: [i32; 8] = [0, 1, 31, 71, 121, 181, 251, 331];
pub static WEXP_DELTAS: [i32; 8] = [0, 30, 40, 50, 60, 70, 80, -1];

#[unity::hook("App", "WeaponLevel", "GetKind")] // 0x71021C93F0
pub fn weapon_level_get_kind_hook(
    level: Il2CppString,
    _method_info: OptionalMethod,
) -> WeaponLevel_Kind {                         // Vanilla Engage values
    match level.to_rust_string().as_str() {
        "N" => WeaponLevel_Kind { value: 0 },   // None
        "E" => WeaponLevel_Kind { value: 1 },   // D
        "D" => WeaponLevel_Kind { value: 2 },   // C
        "C" => WeaponLevel_Kind { value: 3 },   // B
        "B" => WeaponLevel_Kind { value: 4 },   // A
        "A" => WeaponLevel_Kind { value: 5 },   // S
        "S" => WeaponLevel_Kind { value: 6 },   // Unused
        "SS" => WeaponLevel_Kind { value: 7 },  // Unused, should I use Z instead?
        _ => panic!("The provided string is not a WeaponLevel (N,E,D,C,B,A,S,SS).")
    }
}

pub fn wlvl_kind_to_string(wlvl: WeaponLevel_Kind) -> Il2CppString {
    WEXP_KINDS[wlvl.value as usize].into()
}

// Currently, this uses RD values...
pub fn wexp_to_wlvl_kind(value: i32) -> WeaponLevel_Kind {
    if value < WEXP_VALUES[1] {
        WeaponLevel_Kind { value: 0 }   // None
    } else if value < WEXP_VALUES[2] {
        WeaponLevel_Kind { value: 1 }   // E
    } else if value < WEXP_VALUES[3] {
        WeaponLevel_Kind { value: 2 }   // D
    } else if value < WEXP_VALUES[4] {
        WeaponLevel_Kind { value: 3 }   // C
    } else if value < WEXP_VALUES[5] {
        WeaponLevel_Kind { value: 4 }   // B
    } else if value < WEXP_VALUES[6] {
        WeaponLevel_Kind { value: 5 }   // A
    } else if value < WEXP_VALUES[7] {
        WeaponLevel_Kind { value: 6 }   // S
    } else {
        WeaponLevel_Kind { value: 7 }   // SS
    }
}

pub fn wexp_to_wlvl_value(value: i32) -> i32 {
    if value < WEXP_VALUES[1] {
        0
    } else if value < WEXP_VALUES[2] {
        1
    } else if value < WEXP_VALUES[3] {
        2
    } else if value < WEXP_VALUES[4] {
        3
    } else if value < WEXP_VALUES[5] {
        4
    } else if value < WEXP_VALUES[6] {
        5
    } else if value < WEXP_VALUES[7] {
        6
    } else {
        7
    }
}

pub fn wexp_to_wlvl_string(value: i32) -> String {
    if value < WEXP_VALUES[1] {
        WEXP_KINDS[0].to_string()
    } else if value < WEXP_VALUES[2] {
        WEXP_KINDS[1].to_string()
    } else if value < WEXP_VALUES[3] {
        WEXP_KINDS[2].to_string()
    } else if value < WEXP_VALUES[4] {
        WEXP_KINDS[3].to_string()
    } else if value < WEXP_VALUES[5] {
        WEXP_KINDS[4].to_string()
    } else if value < WEXP_VALUES[6] {
        WEXP_KINDS[5].to_string()
    } else if value < WEXP_VALUES[7] {
        WEXP_KINDS[6].to_string()
    } else {
        WEXP_KINDS[7].to_string()
    }
}

pub fn wlvl_to_wexp_kind(wlvl: WeaponLevel_Kind) -> i32 {
    wlvl_to_wexp_value(wlvl.value)
}

pub fn wlvl_to_wexp_value(wlvl: i32) -> i32 {
    WEXP_VALUES[wlvl as usize]
}

pub fn wlvl_to_wexp_string(wlvl: String) -> i32 {
    if wlvl.as_str() == WEXP_KINDS[0] {
        WEXP_VALUES[0]
    } else if wlvl.as_str() == WEXP_KINDS[1] {
        WEXP_VALUES[1]
    } else if wlvl.as_str() == WEXP_KINDS[2] {
        WEXP_VALUES[2]
    } else if wlvl.as_str() == WEXP_KINDS[3] {
        WEXP_VALUES[3]
    } else if wlvl.as_str() == WEXP_KINDS[4] {
        WEXP_VALUES[4]
    } else if wlvl.as_str() == WEXP_KINDS[5] {
        WEXP_VALUES[5]
    } else if wlvl.as_str() == WEXP_KINDS[6] {
        WEXP_VALUES[6]
    } else if wlvl.as_str() == WEXP_KINDS[7] {
        WEXP_VALUES[7]
    } else {
        -1
    }
}

pub fn wlvl_delta_kind(wlvl: WeaponLevel_Kind) -> i32 {
    wlvl_delta_value(wlvl.value)
}

pub fn wlvl_delta_value(wlvl: i32) -> i32 {
    WEXP_DELTAS[wlvl as usize]
}

pub fn wlvl_delta_string(wlvl: String) -> i32 {

    if wlvl.as_str() == WEXP_KINDS[0] {
        WEXP_DELTAS[0]
    } else if wlvl.as_str() == WEXP_KINDS[1] {
        WEXP_DELTAS[1]
    } else if wlvl.as_str() == WEXP_KINDS[2] {
        WEXP_DELTAS[2]
    } else if wlvl.as_str() == WEXP_KINDS[3] {
        WEXP_DELTAS[3]
    } else if wlvl.as_str() == WEXP_KINDS[4] {
        WEXP_DELTAS[4]
    } else if wlvl.as_str() == WEXP_KINDS[5] {
        WEXP_DELTAS[5]
    } else if wlvl.as_str() == WEXP_KINDS[6] {
        WEXP_DELTAS[6]
    } else if wlvl.as_str() == WEXP_KINDS[7] {
        WEXP_DELTAS[7]
    } else {
        -1
    }
}

pub fn can_gain_wexp(unit: Unit, kind: ItemData_Kinds, value: i32) -> (bool, i32, i32, i32) {
    if unit.get_force_type() != Force_Type::player() {
        return (false, -1, 0, 0)
    }

    let base_wexp = calculate_unit_wexp_kind(unit, kind);
    let max_job_wlvl = unit.get_job().get_max_weapon_level_2(kind.value, unit.m_original_aptitude());
    let max_job_wexp = wlvl_to_wexp_kind(max_job_wlvl);
    
    let base_wlvl = wexp_to_wlvl_kind(base_wexp);
    set_new_wlvl(base_wlvl.value);
    let starting_offset = base_wexp - wlvl_to_wexp_kind(base_wlvl);
    let mut delta = wlvl_delta_kind(base_wlvl);
    if base_wlvl == max_job_wlvl {
        delta = -1;
    }

    if base_wexp == 0 || max_job_wexp == 0 {
        (false, -2, 0, 0)
    } else if base_wexp >= max_job_wexp {
        (false, -4, 0, 0)
    } else if value == 0 {
        (false, -3, starting_offset, delta)
    } else {
        if value < max_job_wexp - base_wexp {
            (true, value, starting_offset, delta)
        } else {
            (true, max_job_wexp - base_wexp, starting_offset, delta)
        }
    }
}