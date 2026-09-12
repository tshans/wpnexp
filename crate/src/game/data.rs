
use std::path::Path;
use std::sync::Mutex;
use std::{collections::HashMap, fs::File, io::Read};

use engage::app::{BattleInfoSide, IGodDataMethods, IGodUnit, IJobData};
use engage::app::IPersonDataMethods;
use engage::app::IUnit;
use engage::app::IUnitItemMethods;
use engage::app::ItemData;
use engage::app::ItemData_Kinds;
use engage::app::Force_Type;
use engage::app::GodUnit;
use engage::app::IBattleInfoSideMethods;
use engage::app::IItemDataMethods;
use engage::app::IJobDataMethods;
use engage::app::IUnitItem;
use engage::app::IUnitMethods;
use engage::app::IWeaponMaskMethods;
use engage::app::JobData;
use engage::app::unit::Unit;

use serde_derive::{Deserialize, Serialize};
use serde_json::Error;

use unity::Cast;

use crate::VERSION;
use crate::game::weaponlevel::wexp_to_wlvl_kind;
use crate::game::weaponlevel::wlvl_delta_kind;
use crate::game::weaponlevel::wlvl_to_wexp_kind;
use crate::misc::statics::{BATTLE_WEXP, GOD_APTITUDE, TEMP_UNIT_WEXP_DATA, UNIT_APTITUDE, UNIT_WEXP_DATA, WEXP_DATA};

pub static DEFAULT_FILE_PATH: Mutex<&str> = Mutex::<&str>::new("sd:/engage/mods/WpnExp/default/");
pub static SAVE_FILE_PATH: Mutex<&str> = Mutex::<&str>::new("sd:/engage/mods/WpnExp/save/");
pub static OVERWRITE_FILE_PATH: Mutex<&str> = Mutex::<&str>::new("sd:/engage/config/WpnExp/");

// Storage Read/Write Functions
// (Un)modified plugin configuration data, statically loaded with the plugin's main() function.
pub fn read_config() -> Result<Config, Error> {
    let ow = File::open(OVERWRITE_FILE_PATH.lock().unwrap().to_owned() + "config.json").ok();
    let mut file = match ow {
        Some(path) => path,
        None => {
            println!("Overwrite config file not found, using default. v{}", VERSION);
            File::open(DEFAULT_FILE_PATH.lock().unwrap().to_owned() + "config.json").unwrap()
        },
    };

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();

    let dataset = serde_json::from_str(&data_string)?;

    Ok(dataset)
}

// Weapon and job wexp data, statically loaded with the plugin's main() function.
pub fn read_weapon_data() -> Result<WexpData, Error> {
    let ow = File::open(OVERWRITE_FILE_PATH.lock().unwrap().to_owned() + "wexp_data.json").ok();
    let mut file = match ow {
        Some(path) => path,
        None => {
            println!("Overwrite weapon data file not found, using default. v{}", VERSION);
            File::open(DEFAULT_FILE_PATH.lock().unwrap().to_owned() + "wexp_data.json").unwrap()
        },
    };

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();

    let dataset = serde_json::from_str(&data_string)?;

    Ok(dataset)
}

// Base unit wexp values, loaded with the plugin's main() function.
pub fn read_default_unit_data() -> Result<UnitWexpData, Error> {
    let ow = File::open(OVERWRITE_FILE_PATH.lock().unwrap().to_owned() + "unit_wexp_data.json").ok();
    let mut file = match ow {
        Some(path) => path,
        None => {
            println!("Overwrite unit data file not found, using default. v{}", VERSION);
            File::open(DEFAULT_FILE_PATH.lock().unwrap().to_owned() + "unit_wexp_data.json").unwrap()
        },
    };

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();

    let dataset = serde_json::from_str(&data_string)?;

    Ok(dataset)
}

// Full zero temp unit wexp values, loaded with the plugin's main() function.
pub fn read_default_dyn_unit_data() -> Result<UnitWexpData, Error> {
    let ow = File::open(OVERWRITE_FILE_PATH.lock().unwrap().to_owned() + "dyn_unit_wexp_data.json").ok();
    let mut file = match ow {
        Some(path) => path,
        None => {
            println!("Overwrite dyn unit data file not found, using default. v{}", VERSION);
            File::open(DEFAULT_FILE_PATH.lock().unwrap().to_owned() + "dyn_unit_wexp_data.json").unwrap()
        },
    };

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();

    let dataset = serde_json::from_str(&data_string)?;

    Ok(dataset)
}

// "Static" unit wexp data, written to file on full game save and moved into memory on full game load.
pub fn read_unit_data(save_slot: i32) -> Result<UnitWexpData, Error> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/unit_wexp_data.json";
    let mut file = match File::open(path).ok() {
        Some(file ) => file,
        None => {
            println!("No unit wexp data for save slot {} could be found, using default data.", save_slot);
            return read_default_unit_data()
        },
    };

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();

    let dataset = serde_json::from_str(&data_string)?;

    Ok(dataset)
}

pub fn write_unit_data(save_slot: i32, data: UnitWexpData) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/unit_wexp_data.json";
    let data_string = serde_json::to_string_pretty(&data)?;
    std::fs::write(path, data_string.as_bytes())?;

    Ok(())
}

pub fn copy_unit_data(dst_save_slot: i32, src_save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &src_save_slot.to_string()
        + "/unit_wexp_data.json";
    let dst_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &dst_save_slot.to_string()
        + "/unit_wexp_data.json";

    let mut src_file = match File::open(src_path).ok() {
        Some(file) => file,
        None => return Ok(()), // There is no saved unit data for this slot, so nothing needs to be done.
    };
    let mut src_contents = String::new();
    src_file.read_to_string(&mut src_contents).unwrap();

    std::fs::write(dst_path, src_contents)?;

    Ok(())
}

pub fn delete_unit_data(save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/unit_wexp_data.json";
    if Path::new(&path).try_exists().unwrap() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

// "Dynamic" unit wexp data, written to file on in-map save and moved into memory on in-map load.
pub fn read_dynamic_unit_data(save_slot: i32) -> Result<UnitWexpData, Error> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/dyn_unit_wexp_data.json";
    let mut dynamic_file = match File::open(path).ok() {
        Some(data) => data,
        None => {
            println!("No temp unit wexp data for save slot {} could be found, using default data.", save_slot);
            return read_default_dyn_unit_data()
        },
    };

    let mut dynamic_string = String::new();
    dynamic_file.read_to_string(&mut dynamic_string).unwrap();
    let dynamic = serde_json::from_str(&dynamic_string)?;

    Ok(dynamic)
}

pub fn write_dynamic_unit_data(save_slot: i32, dynamic: UnitWexpData) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/dyn_unit_wexp_data.json";
    let dynamic_data = serde_json::to_string_pretty(&dynamic)?;
    std::fs::write(path, dynamic_data.as_bytes())?;

    Ok(())
}

pub fn copy_dynamic_unit_data(dst_save_slot: i32, src_save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &src_save_slot.to_string()
        + "/dyn_unit_wexp_data.json";
    let dst_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &dst_save_slot.to_string()
        + "/dyn_unit_wexp_data.json";

    let mut src_file = match File::open(src_path).ok() {
        Some(file) => file,
        None => return Ok(()), // There is no saved dynamic unit data for this slot, so nothing needs to be done.
    };

    let mut src_contents = String::new();
    src_file.read_to_string(&mut src_contents).unwrap();

    std::fs::write(dst_path, src_contents)?;

    Ok(())
}

pub fn delete_dynamic_unit_data(save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/dyn_unit_wexp_data.json";
    if Path::new(&path).try_exists().unwrap() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

// Rewind-labelled "dynamic" unit wexp data, written to file on in-map save and moved into memory on in-map load.
pub fn read_ledger_unit_data(save_slot: i32) -> Result<WexpRewindLedger, Error> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/rewind_ledger.json";
    let mut ledger_file = match File::open(path).ok() {
        Some(data) => data,
        None => {
            println!("No wexp ledger data for save slot {} could be found, using default data.", save_slot);
            return Ok(WexpRewindLedger { ledger: HashMap::new() })
        },
    };

    let mut ledger_string = String::new();
    ledger_file.read_to_string(&mut ledger_string).unwrap();
    let ledger = serde_json::from_str(&ledger_string)?;

    Ok(ledger)
}

pub fn write_ledger_unit_data(save_slot: i32, ledger: WexpRewindLedger) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/rewind_ledger.json";
    let ledger_data = serde_json::to_string_pretty(&ledger)?;
    std::fs::write(path, ledger_data.as_bytes())?;

    Ok(())
}

pub fn copy_ledger_unit_data(dst_save_slot: i32, src_save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &src_save_slot.to_string()
        + "/rewind_ledger.json";
    let dst_path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &dst_save_slot.to_string()
        + "/rewind_ledger.json";

    let mut src_file = match File::open(src_path).ok() {
        Some(file) => file,
        None => return Ok(()), // There is no saved ledger for this slot, so nothing needs to be done.
    };
    let mut src_contents = String::new();
    src_file.read_to_string(&mut src_contents).unwrap();

    std::fs::write(dst_path, src_contents)?;

    Ok(())
}

pub fn delete_ledger_unit_data(save_slot: i32) -> Result<(), Box<dyn std::error::Error>> {
    let path = SAVE_FILE_PATH.lock().unwrap().to_owned()
        + &save_slot.to_string()
        + "/rewind_ledger.json";
    if Path::new(&path).try_exists().unwrap() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}


// In-Memory Read/Write Boilerplate Functions
// Unit WEXP total functions
pub fn get_static_wexp(name: String, kind: ItemData_Kinds) -> i32 {
    let ty = match kind.value {
        1 => ItemOptions::Sword,
        2 => ItemOptions::Lance,
        3 => ItemOptions::Axe,
        4 => ItemOptions::Bow,
        5 => ItemOptions::Dagger,
        6 => ItemOptions::Magic,
        7 => ItemOptions::Rod,
        8 => ItemOptions::Fist,
        9 => ItemOptions::Special,
        _ => return 0,
    };

    let mut guard = UNIT_WEXP_DATA.lock().unwrap();
    match guard.as_mut() {
        Some(data) => match data.data.get_mut(&name) {
            Some(wexp) => wexp[ty as usize],
            None => {
                data.data.insert(name, [0;10]);
                0
            },
        },
        None => panic!("Static unit wexp data failed to initialize."),
    }
}

pub fn get_dynamic_wexp(name: String, kind: ItemData_Kinds) -> i32 {
    let ty = match kind.value {
        1 => ItemOptions::Sword,
        2 => ItemOptions::Lance,
        3 => ItemOptions::Axe,
        4 => ItemOptions::Bow,
        5 => ItemOptions::Dagger,
        6 => ItemOptions::Magic,
        7 => ItemOptions::Rod,
        8 => ItemOptions::Fist,
        9 => ItemOptions::Special,
        _ => return 0,
    };

    let mut guard = TEMP_UNIT_WEXP_DATA.lock().unwrap();
    match guard.as_mut() {
        Some(data) => match data.data.get_mut(&name) {
            Some(wexp) => wexp[ty as usize],
            None => {
                data.data.insert(name, [0;10]);
                0
            },
        },
        None => panic!("Dynamic unit wexp data failed to initialize."),
    }
}

pub fn set_dynamic_wexp(name: String, kind: ItemData_Kinds, value: i32) {
    let ty = match kind.value {
        1 => ItemOptions::Sword,
        2 => ItemOptions::Lance,
        3 => ItemOptions::Axe,
        4 => ItemOptions::Bow,
        5 => ItemOptions::Dagger,
        6 => ItemOptions::Magic,
        7 => ItemOptions::Rod,
        8 => ItemOptions::Fist,
        9 => ItemOptions::Special,
        _ => return,
    };

    let mut guard = TEMP_UNIT_WEXP_DATA.lock().unwrap();
    match guard.as_mut() {
        Some(data) => match data.data.get_mut(&name) {
            Some(wexp) => wexp[ty as usize] = value,
            None => {
                let mut new_data = [0;10];
                new_data[ty as usize] = value;
                data.data.insert(name, new_data);
            },
        },
        None => panic!("Dynamic unit wexp data failed to initialize."),
    };
}

pub fn add_dynamic_wexp(name: String, kind: ItemData_Kinds, value: i32) {
    let initial = get_dynamic_wexp(name.clone(), kind);
    set_dynamic_wexp(name.clone(), kind, initial + value);
}

pub fn get_job_wexp(job: JobData, kind: ItemData_Kinds) -> i32 {
    if job.is_null() {
        return 0
    }

    match job.max_weapon_levels().get(kind.value as usize).to_rust_string().as_str() {
        "E" => crate::game::weaponlevel::WEXP_VALUES[1],    // Guarantees use of E rank weapons
        "E+" => crate::game::weaponlevel::WEXP_VALUES[1],
        "D" => crate::game::weaponlevel::WEXP_VALUES[1],
        "D+" => crate::game::weaponlevel::WEXP_VALUES[1],
        "C" => crate::game::weaponlevel::WEXP_VALUES[1],    // In vanilla, this is the lowest Max WLVL.
        "C+" => crate::game::weaponlevel::WEXP_VALUES[1],
        "B" => crate::game::weaponlevel::WEXP_VALUES[1],
        "B+" => crate::game::weaponlevel::WEXP_VALUES[2],   // Guarantees use of D rank weapons
        "A" => crate::game::weaponlevel::WEXP_VALUES[2],
        "A+" => crate::game::weaponlevel::WEXP_VALUES[3],   // Guarantees use of C rank weapons
        "S" => crate::game::weaponlevel::WEXP_VALUES[3],
        "S+" => crate::game::weaponlevel::WEXP_VALUES[4],   // Guarantees use of B rank weapons
        "SS" => crate::game::weaponlevel::WEXP_VALUES[4],   // Use Z instead?
        _ => crate::game::weaponlevel::WEXP_VALUES[0],      // Includes "N", cannot use provided kind
    }

    // let ty = match kind.value {
    //     1 => ItemOptions::Sword,
    //     2 => ItemOptions::Lance,
    //     3 => ItemOptions::Axe,
    //     4 => ItemOptions::Bow,
    //     5 => ItemOptions::Dagger,
    //     6 => ItemOptions::Magic,
    //     7 => ItemOptions::Rod,
    //     8 => ItemOptions::Fist,
    //     9 => ItemOptions::Special,
    //     _ => return 0,
    // };

    // let jid = job.get_jid().to_rust_string();
    // match WEXP_DATA
    //     .get()
    //     .unwrap()
    //     .job
    //     .get(&jid) {
    //         Some(value) => value[ty as usize],
    //         None => 0,
    //     }
}

pub fn calculate_unit_wexp_kind_job(
    unit: Unit,
    kind: ItemData_Kinds,
    reclass_job: JobData,
) -> i32 {
    if unit.is_null() || unit.is_summon() || unit.get_force_type() != Force_Type::player() {
        return 0
    }

    if reclass_job.is_null() {
        return 0
    }

    // This checks the current selection of modular weapon classes (Griffin, Paladin, Wyvern, Great Knight, etc.)
    // and returns 0 if the class has proficiency in the weapon kind listed.
    if !reclass_job.get_weapon_mask(unit.m_weapon_mask(), unit.m_selected_weapon_mask()).test_2(kind) {
        return 0
    }

    let result = get_static_wexp(unit.m_person().get_name().to_rust_string(), kind)
        + get_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind)
        + get_job_wexp(reclass_job, kind);

    let wlvl = wexp_to_wlvl_kind(result);
    let max_wlvl = reclass_job.get_max_weapon_level_2(kind.value, unit.m_original_aptitude());
    if wlvl.value >= max_wlvl.value {
        wlvl_to_wexp_kind(max_wlvl)
    } else {
        result
    }
}

pub fn calculate_unit_wexp_kind(
    unit: Unit,
    kind: ItemData_Kinds,
) -> i32 {
    if unit.is_null() || unit.is_summon() || unit.get_force_type() != Force_Type::player() {
        return 0
    }

    let job = unit.get_job();
    if job.is_null() {
        return 0
    }

    // This checks the current selection of modular weapon classes (Griffin, Paladin, Wyvern, Great Knight, etc.)
    // and returns 0 if the class has proficiency in the weapon kind listed.
    if !job.get_weapon_mask(unit.m_weapon_mask(), unit.m_selected_weapon_mask()).test_2(kind) {
        return 0
    }

    let result = get_static_wexp(unit.m_person().get_name().to_rust_string(), kind)
        + get_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind)
        + get_job_wexp(job, kind);

    let wlvl = wexp_to_wlvl_kind(result);
    let max_wlvl = job.get_max_weapon_level_2(kind.value, unit.m_original_aptitude());
    if wlvl.value >= max_wlvl.value {
        wlvl_to_wexp_kind(max_wlvl)
    } else {
        result
    }
}

#[allow(dead_code)] // This function will mostly be used in external plugins.
pub fn calculate_side_wexp_kind(
    side: BattleInfoSide,
    kind: ItemData_Kinds,
) -> i32 {
    if side.is_null() {
        return 0
    }

    calculate_unit_wexp_kind(side.get_unit(), kind)
}

// Earned WEXP functions
pub fn get_item_wexp(item_data: ItemData) -> i32 {
    let kind = item_data.get_kind();
    let ty = match kind.value {
        1 => ItemOptions::Sword,
        2 => ItemOptions::Lance,
        3 => ItemOptions::Axe,
        4 => ItemOptions::Bow,
        5 => ItemOptions::Dagger,
        6 => ItemOptions::Magic,
        7 => ItemOptions::Rod,
        8 => ItemOptions::Fist,
        9 => ItemOptions::Special,
        _ => return 0,
    };

    let miid = item_data.get_name().to_rust_string();
    match WEXP_DATA
        .get()
        .unwrap()
        .item
        .get(&ty)
        .unwrap()
        .get(&miid) {
            Some(value) => *value,
            None => 0,
        }
}

pub fn get_unit_wexp(unit: Unit, kind: ItemData_Kinds) -> i32 {
    if !*UNIT_APTITUDE.lock().unwrap() ||
        unit.is_null() ||
        unit.m_aptitude().is_null() ||
        unit.m_original_aptitude().is_null() {
            return 0
        }

    if unit.m_original_aptitude().test_2(kind) {
        1
    } else {
        0
    }

    // let ty = match kind.value {
    //     1 => ItemOptions::Sword,
    //     2 => ItemOptions::Lance,
    //     3 => ItemOptions::Axe,
    //     4 => ItemOptions::Bow,
    //     5 => ItemOptions::Dagger,
    //     6 => ItemOptions::Magic,
    //     7 => ItemOptions::Rod,
    //     8 => ItemOptions::Fist,
    //     9 => ItemOptions::Special,
    //     _ => return 0,
    // };

    // let mpid = unit.m_person().get_name().to_rust_string();
    // match WEXP_DATA
    //     .get()
    //     .unwrap()
    //     .unit
    //     .get(&mpid) {
    //         Some(value) => value[ty as usize],
    //         None => 0,
    //     }
}

pub fn get_god_wexp(god: GodUnit, kind: ItemData_Kinds) -> i32 {
    if !*GOD_APTITUDE.lock().unwrap() || god.is_null() || god.m_data().is_null() {
        return 0
    }

    if god.m_data().get_good_weapon() == kind {
        1
    } else {
        0
    }

    // let ty = match kind.value {
    //     1 => ItemOptions::Sword,
    //     2 => ItemOptions::Lance,
    //     3 => ItemOptions::Axe,
    //     4 => ItemOptions::Bow,
    //     5 => ItemOptions::Dagger,
    //     6 => ItemOptions::Magic,
    //     7 => ItemOptions::Rod,
    //     8 => ItemOptions::Fist,
    //     9 => ItemOptions::Special,
    //     _ => return 0,
    // };

    // let gid = god.get_gid().to_rust_string();
    // match WEXP_DATA
    //     .get()
    //     .unwrap()
    //     .god
    //     .get(&gid) {
    //         Some(value) => value[ty as usize],
    //         None => 0,
    //     }
}

pub fn calculate_unit_earned_wexp(
    unit: Unit,
) -> i32 {
    if unit.is_null() || unit.is_summon() || unit.get_force_type() != Force_Type::player() {
        return 0
    }
    
    let mut unit_item = unit.get_item_selected();
    if unit_item.is_null() || unit_item.m_item().is_null() {
        unit_item = unit.get_item_equipped();
        if unit_item.is_null() || unit_item.m_item().is_null() {
            return 0
        }
    }

    if !unit
        .get_job()
        .get_weapon_mask(unit.m_weapon_mask(), unit.m_selected_weapon_mask())
        .test_2(unit_item.get_kind()) {
            return 0
        }

    get_item_wexp(unit_item.m_item())
        + get_unit_wexp(unit, unit_item.get_kind())
        + get_god_wexp(unit.get_god_unit(), unit_item.get_kind())
}

pub fn calculate_side_earned_wexp(
    side: BattleInfoSide,
) -> i32 {
    if side.is_null() {
        return 0
    }

    calculate_unit_earned_wexp(side.get_unit())
}

// Arms Scroll related functions
pub fn calculate_arms_scroll_delta(
    unit: Unit,
    kind: ItemData_Kinds,
) -> Option<i32> {
    if unit.is_null() || kind.value == 0 || kind.value > 9 {
        return None
    }

    let base_wexp = calculate_unit_wexp_kind(unit, kind);
    let current_wlvl = wexp_to_wlvl_kind(base_wexp);
    let wexp_delta = wlvl_delta_kind(current_wlvl);

    let max_wlvl = unit
        .get_job()
        .get_max_weapon_level_2(kind.value, unit.m_original_aptitude());

    if wexp_delta < 1 || current_wlvl == max_wlvl {
        // The unit's WEXP is at the class maximum for the equipped item type (including adjustments from innate proficiency)
        // or the unit cannot gain WEXP for the equipped item type, so the unit should not be allowed to use an Arms Scroll.
        None
    } else if current_wlvl.value == max_wlvl.value - 1 {
        // If the unit's WLVL is one less than the max WLVL, we increase WEXP by exactly enough to reach max WLVL.
        Some(wlvl_to_wexp_kind(max_wlvl) - base_wexp)
    } else {
        // Otherwise, we increase WEXP by the difference between the current and next WLVL.
        Some(wexp_delta)
    }
}

pub fn arms_scroll_can_grow(unit: Unit) -> bool {
    for i in 1..=9 {
        let delta = calculate_arms_scroll_delta(unit, ItemData_Kinds { value: i });
        match delta {
            Some(_value) => return true,
            None => continue
        }
    }

    false
}

// WEXP Data Structures
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Config {
    pub enabled: bool,
    pub unit_aptitude: bool,
    pub god_aptitude: bool,
    pub overwrite: bool,
    pub index: usize,
}

// Item WEXP data structure
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct WexpData {
    // Values are mostly vibes-based atm; this should probably be properly tested...
    pub item: HashMap<ItemOptions, HashMap<String, i32>>, // (ItemOptions, (MIID, WEXP value))

    // The below data is now derived from the job/god/person xml files and has been removed from the json file.
    // // MaxWlvl = WEXP bonus: C/C+/B/B+ = 1 (E), A/A+ = 31 (D),  S = 71 (C).
    // pub job: HashMap<String, [i32; 10]>, // (JID, [WEXP])
    // // Emblems grant extra WEXP with their favored weapon type.
    // pub god: HashMap<String, [i32; 10]>, // (GID, [WEXP])
    // // Units gain extra WEXP with Primary/Secondary weapons.
    // pub unit: HashMap<String, [i32; 10]>, // (MPID, [WEXP])
}

// Static/Temporary WEXP data structure
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct UnitWexpData {
    pub data: HashMap<String, [i32; 10]>, // (MPID, [WEXP])
}

pub fn add_array(first: [i32; 10], second: [i32; 10]) -> [i32; 10] {
    let mut count = -1;
    first.map(|e| { count += 1; e + second[count as usize] })
}

impl UnitWexpData {
    pub fn new() -> Self {
        UnitWexpData { data: HashMap::new() }
    }
    pub fn add(&mut self, consumed: Self) {
        for i in consumed.data {
            let entry = match self.data.remove(&i.0) {
                Some(data) => add_array(data, i.1),
                None => i.1,
            };
            self.data.insert(i.0, entry);
        }
    }
}

impl Default for UnitWexpData {
    fn default() -> Self {
        UnitWexpData::new()
    }
}

// Rewind dynamic WEXP ledger
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct WexpRewindLedger {
    pub ledger: HashMap<i32, UnitWexpData> // (CommandNumber, TempUnitWexpData)
}

impl WexpRewindLedger {
    pub fn new() -> Self {
        WexpRewindLedger { ledger: HashMap::new() }
    }
}

impl Default for WexpRewindLedger {
    fn default() -> Self {
        WexpRewindLedger::new()
    }
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone, Copy)]
#[repr(C)]
pub enum ItemOptions {
    None,
    Sword,
    Lance,
    Axe,
    Bow,
    Dagger,
    Magic,
    Rod,
    Fist,
    Special,
}


// In-battle WEXP backend
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct BattleWEXP {
    pub once_wexp: HashMap<String, i32>,    // (MPID, WEXP)
    pub total_wexp: HashMap<String, i32>,   // (MPID, WEXP)
    pub lvl_and_exp: [i32; 4],              // (Old WLVL, New WLVL, TempWEXP, Item_Kinds)
}

impl BattleWEXP {
    pub fn new() -> Self {
        BattleWEXP { once_wexp: HashMap::new(), total_wexp: HashMap::new(), lvl_and_exp: [0,0,0,0] }
    }
}

impl Default for BattleWEXP {
    fn default() -> Self {
        BattleWEXP::new()
    }
}

pub fn get_once_wexp(unit: Unit) -> i32 {
    // Handle null results on unit/person when calling this function?
    let mpid = unit.get_person().get_name().to_rust_string();
    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => match data.once_wexp.get_mut(&mpid) {
            Some(wexp) => *wexp,
            None => {
                data.once_wexp.insert(mpid, 0);
                0
            },
        },
        None => panic!("Once WEXP data failed to initialize."),
    }
}

pub fn set_once_wexp(unit: Unit, value: i32) {
    // Handle null results on unit/person when calling this function?
    let mpid = unit.get_person().get_name().to_rust_string();
    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.once_wexp.insert(mpid, value),
        None => panic!("Once WEXP data failed to initialize."),
    };
}

pub fn get_total_wexp(unit: Unit) -> i32 {
    // Handle null results on unit/person when calling this function?
    let mpid = unit.get_person().get_name().to_rust_string();
    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => match data.total_wexp.get_mut(&mpid) {
            Some(wexp) => *wexp,
            None => {
                data.total_wexp.insert(mpid, 0);
                0
            },
        },
        None => panic!("Once WEXP data failed to initialize."),
    }
}

pub fn set_total_wexp(unit: Unit, value: i32) {
    // Handle null results on unit/person when calling this function?
    let mpid = unit.get_person().get_name().to_rust_string();
    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.total_wexp.insert(mpid, value),
        None => panic!("Once WEXP data failed to initialize."),
    };
}

pub fn get_old_wlvl() -> i32 {    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[0],
        None => panic!("BattleWexp data failed to initialize."),
    }
}

pub fn set_old_wlvl(value: i32) {
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[0] = value,
        None => panic!("BattleWexp data failed to initialize."),
    };
}

pub fn get_new_wlvl() -> i32 {    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[1],
        None => panic!("BattleWexp data failed to initialize."),
    }
}

pub fn set_new_wlvl(value: i32) {
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[1] = value,
        None => panic!("BattleWexp data failed to initialize."),
    };
}

pub fn get_temp_wexp() -> i32 {    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[2],
        None => panic!("BattleWexp data failed to initialize."),
    }
}

pub fn set_temp_wexp(value: i32) {
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[2] = value,
        None => panic!("BattleWexp data failed to initialize."),
    };
}

pub fn get_weapon_kind() -> i32 {    
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[3],
        None => panic!("BattleWexp data failed to initialize."),
    }
}

pub fn set_weapon_kind(value: i32) {
    let mut guard = BATTLE_WEXP.lock().unwrap();
    match guard.as_mut() {
        Some(data) => data.lvl_and_exp[3] = value,
        None => panic!("BattleWexp data failed to initialize."),
    };
}