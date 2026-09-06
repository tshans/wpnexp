
use std::sync::{Mutex, OnceLock};

use crate::game::data::{
    BattleWEXP,
    UnitWexpData,
    WexpData,
    WexpRewindLedger,
    read_default_dyn_unit_data,
    read_default_unit_data,
    read_weapon_data,
};

pub static VERSION: &str = "1.0.0";
pub static ENABLED: Mutex<bool> = Mutex::<bool>::new(true);

// This is set statically during plugin launch, loading WEXP values for every weapon from a preset config JSON.
pub static WEXP_DATA: OnceLock<WexpData> = OnceLock::<WexpData>::new();

// These are set to a default value on plugin launch and their contents are replaced with (written as) data from (in)
// a modular config JSON on game load (save). Their values change dynamically throughout gameplay.
pub static UNIT_WEXP_DATA: Mutex<Option<UnitWexpData>> = Mutex::<Option<UnitWexpData>>::new(None);
pub static TEMP_UNIT_WEXP_DATA: Mutex<Option<UnitWexpData>> = Mutex::<Option<UnitWexpData>>::new(None);
pub static WEXP_LEDGER: Mutex<Option<WexpRewindLedger>> = Mutex::<Option<WexpRewindLedger>>::new(None);

pub static BATTLE_WEXP: Mutex<Option<BattleWEXP>> = Mutex::<Option<BattleWEXP>>::new(None);

pub fn init_statics() {
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