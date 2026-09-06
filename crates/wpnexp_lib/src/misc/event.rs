
use engage::app::GameMessage;
use engage::app::GameSound;
use engage::app::IGameMessageMethods;
use engage::app::IProcInstMethods;
use engage::app::Mess;
use engage::app::WeaponLevel_Kind;
use engage::app::unitstatus::UnitStatus;
use engage::combat::Character;
use engage::unity_engine::GameObject;
use engage::unity_engine::IComponentMethods;
use engage::unity_engine::IGameObjectMethods;
use engage::unity_engine::IObject_2Methods;
use engage::unity_engine::IRectTransformMethods;
use engage::unity_engine::ITransformMethods;
use engage::unity_engine::Object_2;
use engage::unity_engine::RectTransform;
use engage::unity_engine::Vector2;
use engage::unity_engine::Vector3;

use unity::Cast;
use unity::IlNull;

use cobapi::Event;
use cobapi::SystemEvent;

use crate::game::data::get_new_wlvl;
use crate::game::data::get_old_wlvl;
use crate::game::data::get_weapon_kind;
use crate::game::data::read_dynamic_unit_data;
use crate::game::data::read_ledger_unit_data;
use crate::game::data::read_unit_data;
use crate::game::weaponlevel::wlvl_kind_to_string;

use crate::misc::option_null;
use crate::misc::statics::TEMP_UNIT_WEXP_DATA;
use crate::misc::statics::UNIT_WEXP_DATA;
use crate::misc::statics::WEXP_LEDGER;


#[no_mangle]
pub extern "C" fn listener(event: &Event<SystemEvent>) {
    if let Event::Args(SystemEvent::ProcInstJump { proc, label }) = event {
        if proc.get_hash_code() == -339912801 && *label == 1 {
            // We modify the UnitStatus prefab after the game has loaded it into memory.
            // MainSequence.Label == Startup (1)
            let unit_status_t = UnitStatus::m_game_object().get_transform();

            let exp_t = unit_status_t
                .find_child("Contents/Parameter/Contents/JobInfo/Content/Status/Lv/ExpGauge");
            let exp_bar = exp_t.get_game_object();

            let wlvl_t = unit_status_t
                .find_child("Contents/Parameter/Contents/JobInfo/Content/WeaponLv");
            for i in 0..4 {
                let root_t = wlvl_t.get_child(i);

                match option_null(root_t.find_child("WexpGauge")) {
                    Some(_wexp_gauge) => continue,
                    None => {
                        // We copy the game's default EXP bar from the UnitStatus prefab.
                        let wexp_gauge = Object_2::instantiate_3(exp_bar)
                            .try_cast::<GameObject>()
                            .unwrap();
                        wexp_gauge.set_name("WexpGauge");

                        let wexp_gauge_t = wexp_gauge
                            .get_transform()
                            .try_cast::<RectTransform>()
                            .unwrap();

                        let root_p = root_t.get_position();
                        let wexp_gauge_p = wexp_gauge_t.get_position();
                        let shift = Vector3 {
                            x: root_p.x - wexp_gauge_p.x + 6.0,     // Shift the gauge 6 pixels right of the root
                            y: root_p.y - wexp_gauge_p.y - 18.0,    // Shift the gauge 18 pixels down from the root
                            z: root_p.z - wexp_gauge_p.z,           // The z position is unchanged from the root
                        };
                        wexp_gauge_t.translate_2(shift);
                        wexp_gauge_t.set_size_delta(Vector2 { x: 40.0, y: 4.0 }); // Set the max size of the Gauge

                        let width_t = wexp_gauge_t
                            .find_child("Width")
                            .try_cast::<RectTransform>()
                            .unwrap();
                        width_t.set_size_delta(Vector2 { x: 40.0, y: 0.0 });    // Set the default size of the Gauge

                        let front_t = width_t
                            .find_child("Mask/Front")
                            .try_cast::<RectTransform>()
                            .unwrap();
                        front_t.set_size_delta(Vector2 { x: 40.0, y: 0.0 });    // Set the default size of the Mask

                        wexp_gauge_t.set_parent(root_t);
                    },
                }
            }
        } else if proc.get_hash_code() == -813168385 && *label == 1 {
            // We add a splash screen after a wlvl increase.
            // UnitGrowSequence.Label == LevelUpEnd (1)
            let kind = get_weapon_kind();
            let old = get_old_wlvl();
            let new = get_new_wlvl();
            if old < new && kind != 0 && kind <= 9 {
                let kind_string = match kind {
                    1 => Mess::get("MID_H_INFO_WLV_Sword"),
                    2 => Mess::get("MID_H_INFO_WLV_Lance"),
                    3 => Mess::get("MID_H_INFO_WLV_Axe"),
                    4 => Mess::get("MID_H_INFO_WLV_Bow"),
                    5 => Mess::get("MID_H_INFO_WLV_Dagger"),
                    6 => Mess::get("MID_H_INFO_WLV_Magic"),
                    7 => Mess::get("MID_H_INFO_WLV_Rod"),
                    8 => Mess::get("MID_H_INFO_WLV_Fist"),
                    9 => Mess::get("MID_H_INFO_WLV_Special"),
                    _ => "".into(),
                };

                let new_wlvl = wlvl_kind_to_string(WeaponLevel_Kind { value: new });

                let message = Mess::get_3("MID_MSG_WLVL_Increase", kind_string, new_wlvl);
                // ParamUp's sound doesn't feel impactful enough...
                // Other options include
                //      - SE_Result_SSS (SSS somniel minigame result sound)
                //      - SE_Muscle_Good/Perfect/Assist (Good/Perfect/Assist result during bodybuilding rhythm games)
                //      - ItemGet/ItemGet_Important
                //      - Status_Up
                //      - LevelUp/LevelUp_Short <--- Leaning towards the short version atm
                //      - RelianceLevelUp
                //      - WeaponUpgrade
                //      - Notification
                GameSound::post_event("LevelUp_Short", Character::null());
                let mess = GameMessage::create_system(*proc, message);
                mess.set_shadow_off();
            }
        }
    } else if let Event::Args(SystemEvent::SaveLoaded { ty, slot_id }) = event {
        if *ty == 5 {
            println!("Autosave loaded.");
            let static_data = read_unit_data(10).ok();
            let temp_data = read_dynamic_unit_data(10).ok();
            let ledger_data = read_ledger_unit_data(10).ok();

            UNIT_WEXP_DATA.set(static_data).expect("Failed to load autosave values into UNIT_WEXP_DATA.");
            TEMP_UNIT_WEXP_DATA.set(temp_data).expect("Failed to load autosave values into TEMP_UNIT_WEXP_DATA.");
            WEXP_LEDGER.set(ledger_data).expect("Failed to load autosave values into WEXP_LEDGER.");
        } else if *ty == 1 {
            println!("Global save loaded, so we do nothing.");
        } else {
            println!("Other save type loaded.");
            let static_data = read_unit_data(*slot_id).ok();
            let temp_data = read_dynamic_unit_data(*slot_id).ok();
            let ledger_data = read_ledger_unit_data(*slot_id).ok();

            UNIT_WEXP_DATA.set(static_data).expect("Failed to load save values into UNIT_WEXP_DATA.");
            TEMP_UNIT_WEXP_DATA.set(temp_data).expect("Failed to load save values into TEMP_UNIT_WEXP_DATA.");
            WEXP_LEDGER.set(ledger_data).expect("Failed to load save values into WEXP_LEDGER.");
        }
    }
    // else if let Event::Args(SystemEvent::SaveSaved { ty, slot_id }) = event {
    //     if *ty == 5 {
    //         let mut unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();

    //         unit_data.add(temp_data);

    //         let new_unit_data = unit_data.clone();

    //         write_unit_data(10, unit_data).expect("Failed to write UnitWexpData to file.");

    //         let default_temp_data = read_default_dyn_unit_data().ok();
    //         let default_ledger = Some(WexpRewindLedger::new());

    //         UNIT_WEXP_DATA.set(Some(new_unit_data)).expect("Failed to set UnitWexpData.");
    //         TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
    //         WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    //     } else if *ty == 1 {
    //         println!("Global save saved, so we do nothing.");
    //     } else {
    //         let mut unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();

    //         unit_data.add(temp_data);

    //         let new_unit_data = unit_data.clone();

    //         write_unit_data(*slot_id, unit_data).expect("Failed to write UnitWexpData to file.");

    //         let default_temp_data = read_default_dyn_unit_data().ok();
    //         let default_ledger = Some(WexpRewindLedger::new());

    //         UNIT_WEXP_DATA.set(Some(new_unit_data)).expect("Failed to set UnitWexpData.");
    //         TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
    //         WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    //     }
    // } else if let Event::Args(SystemEvent::SaveSuspended { ty, slot_id }) = event {
    //     if *ty == 5 {
    //         let unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let ledger = WEXP_LEDGER.get_cloned().unwrap().unwrap();

    //         write_unit_data(10, unit_data).expect("Failed to write UnitWexpData to file.");
    //         write_dynamic_unit_data(10, temp_data).expect("Failed to write TempUnitWexpData to file.");
    //         write_ledger_unit_data(10, ledger).expect("Failed to write WexpLedger to file.");

    //         let default_unit_data = read_default_unit_data().ok();
    //         let default_temp_data = read_default_dyn_unit_data().ok();
    //         let default_ledger = Some(WexpRewindLedger::new());

    //         UNIT_WEXP_DATA.set(default_unit_data).expect("Failed to set default UnitWexpData.");
    //         TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
    //         WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    //     } else {
    //         let unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    //         let ledger = WEXP_LEDGER.get_cloned().unwrap().unwrap();

    //         write_unit_data(*slot_id, unit_data).expect("Failed to write UnitWexpData to file.");
    //         write_dynamic_unit_data(*slot_id, temp_data).expect("Failed to write TempUnitWexpData to file.");
    //         write_ledger_unit_data(*slot_id, ledger).expect("Failed to write WexpLedger to file.");

    //         let default_unit_data = read_default_unit_data().ok();
    //         let default_temp_data = read_default_dyn_unit_data().ok();
    //         let default_ledger = Some(WexpRewindLedger::new());

    //         UNIT_WEXP_DATA.set(default_unit_data).expect("Failed to set default UnitWexpData.");
    //         TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
    //         WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    //     }
    // } else if let Event::Args(SystemEvent::SaveDeleted { ty, slot_id }) = event {
    //     if *ty == 5 {
    //         delete_unit_data(10).expect("Failed to delete UnitWexpData file.");
    //         delete_dynamic_unit_data(10).expect("Failed to delete TempUnitWexpData file.");
    //         delete_ledger_unit_data(10).expect("Failed to delete WexpLedger file.");
    //     } else {
    //         delete_unit_data(*slot_id).expect("Failed to delete UnitWexpData file.");
    //         delete_dynamic_unit_data(*slot_id).expect("Failed to delete TempUnitWexpData file.");
    //         delete_ledger_unit_data(*slot_id).expect("Failed to delete WexpLedger file.");
    //     }
    // } else if let Event::Args(SystemEvent::SaveCopied { from_ty, from_id, _to_ty, to_id }) = event {
    //     if *from_ty == 5 {
    //         copy_unit_data(*to_id, 10).expect("Failed to copy UnitWexpData.");
    //         copy_dynamic_unit_data(*to_id, 10).expect("Failed to copy TempUnitWexpData.");
    //         copy_ledger_unit_data(*to_id, 10).expect("Failed to copy WexpLedger.");
    //     } else {
    //         copy_unit_data(*to_id, *from_id).expect("Failed to copy UnitWexpData.");
    //         copy_dynamic_unit_data(*to_id, *from_id).expect("Failed to copy TempUnitWexpData.");
    //         copy_ledger_unit_data(*to_id, *from_id).expect("Failed to copy WexpLedger.");
    //     }
    // }
}


pub fn register_listener() {
    cobapi::register_system_event_handler(listener);
}