
use engage::app::BasicMenu_Result;
use engage::app::GameSaveData_Types;
use engage::app::GameUserRestartData_Targtes;
use engage::app::IGameSaveDataHeaderMethods;
use engage::app::IGameSaveDataHeaderReader_HandleMethods;
use engage::app::IMapSequenceMethods;
use engage::app::ISaveDataMenu_ConfirmDialog_YesItem;
use engage::app::ISaveDataMenu_CopyConfirmDialog_YesDialogItem;
use engage::app::ProcInst;
use engage::app::SaveDataMenu_CopyConfirmDialog_YesDialogItem;
use engage::app::SaveDataMenu_LoadConfirmDialog_YesDialogItem;
use engage::app::SaveDataMenu_SaveConfirmDialog_YesDialogItem;
use engage::app::SaveDataMenu_SuspendConfirmDialog_YesDialogItem;
use engage::app::mainsequence::MainSequence;
use engage::app::mapsequence::MapSequence;
use engage::app::savedatamenu::SaveDataMenu_DeleteConfirmDialog_YesDialogItem;

use unity::Cast;
use unity::OptionalMethod;

use crate::game::data::WexpRewindLedger;
use crate::game::data::copy_dynamic_unit_data;
use crate::game::data::copy_ledger_unit_data;
use crate::game::data::copy_unit_data;
use crate::game::data::delete_dynamic_unit_data;
use crate::game::data::delete_ledger_unit_data;
use crate::game::data::delete_unit_data;
use crate::game::data::read_default_dyn_unit_data;
use crate::game::data::read_default_unit_data;
use crate::game::data::read_dynamic_unit_data;
use crate::game::data::read_ledger_unit_data;
use crate::game::data::read_unit_data;
use crate::game::data::write_dynamic_unit_data;
use crate::game::data::write_ledger_unit_data;
use crate::game::data::write_unit_data;

use crate::TEMP_UNIT_WEXP_DATA;
use crate::UNIT_WEXP_DATA;
use crate::WEXP_LEDGER;


#[unity::hook("App", "GameUserRestartData", "SetTarget")] // 0x710251E1D0
pub fn game_user_restart_set_target_hook(
    target: GameUserRestartData_Targtes,
    keep_level: bool,
    method_info: OptionalMethod,
) {
    call_original!(target, keep_level, method_info);

    if !keep_level {
        let default_temp_data = read_default_dyn_unit_data().ok();
        TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData on map restart.");
    }
}

#[unity::hook("App", "SaveDataMenu.LoadConfirmDialog.YesDialogItem", "ACall")] // 0x71022ED580
pub fn load_confirm_a_call_hook(
    this: SaveDataMenu_LoadConfirmDialog_YesDialogItem,
    method_info: OptionalMethod,
) -> BasicMenu_Result {
    let ty = this.m_dest_save_data_header_handle().get_type();
    println!("Loading game of type {}.", ty.value);

    if ty == GameSaveData_Types::manual() {
        let index = this.m_dest_save_data_header_handle().get_index();

        let static_data = read_unit_data(index).ok();
        let temp_data = read_dynamic_unit_data(index).ok();
        let ledger_data = read_ledger_unit_data(index).ok();

        UNIT_WEXP_DATA.set(static_data).expect("Failed to load new values into UNIT_WEXP_DATA.");
        TEMP_UNIT_WEXP_DATA.set(temp_data).expect("Failed to load new values into TEMP_UNIT_WEXP_DATA.");
        WEXP_LEDGER.set(ledger_data).expect("Failed to load new values into WEXP_LEDGER.");
    } else if ty == GameSaveData_Types::auto() {
        let static_data = read_unit_data(10).ok();
        let temp_data = read_dynamic_unit_data(10).ok();
        let ledger_data = read_ledger_unit_data(10).ok();

        UNIT_WEXP_DATA.set(static_data).expect("Failed to load new values into UNIT_WEXP_DATA.");
        TEMP_UNIT_WEXP_DATA.set(temp_data).expect("Failed to load new values into TEMP_UNIT_WEXP_DATA.");
        WEXP_LEDGER.set(ledger_data).expect("Failed to load new values into WEXP_LEDGER.");
    }

    call_original!(this, method_info)
}

#[unity::hook("App", "SaveDataMenu.SaveConfirmDialog.YesDialogItem", "ACall")] // 0x71022ED760
pub fn save_confirm_a_call_hook(
    this: SaveDataMenu_SaveConfirmDialog_YesDialogItem,
    method_info: OptionalMethod,
) -> BasicMenu_Result {
    let ty = this.m_dest_save_data_header_handle().get_type();
    println!("Saving game of type {}.", ty.value);

    let mut is_temp = false;
    if !this.m_dest_save_data_header_handle().get_header().is_null() {
        is_temp = this.m_dest_save_data_header_handle().get_header().is_temporary();
    }

    if ty == GameSaveData_Types::manual() && !is_temp {
        let index = this.m_dest_save_data_header_handle().get_index();

        let mut unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();

        unit_data.add(temp_data);

        let new_unit_data = unit_data.clone();

        write_unit_data(index, unit_data).expect("Failed to write UnitWexpData to file.");

        let default_temp_data = read_default_dyn_unit_data().ok();
        let default_ledger = Some(WexpRewindLedger::new());

        UNIT_WEXP_DATA.set(Some(new_unit_data)).expect("Failed to set UnitWexpData.");
        TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
        WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    } else if ty == GameSaveData_Types::manual() && is_temp {
        let index = this.m_dest_save_data_header_handle().get_index();

        let unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let ledger = WEXP_LEDGER.get_cloned().unwrap().unwrap();

        write_unit_data(index, unit_data).expect("Failed to write UnitWexpData to file.");
        write_dynamic_unit_data(index, temp_data).expect("Failed to write TempUnitWexpData to file.");
        write_ledger_unit_data(index, ledger).expect("Failed to write WexpLedger to file.");

        let default_unit_data = read_default_unit_data().ok();
        let default_temp_data = read_default_dyn_unit_data().ok();
        let default_ledger = Some(WexpRewindLedger::new());

        UNIT_WEXP_DATA.set(default_unit_data).expect("Failed to set default UnitWexpData.");
        TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
        WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    }
    
    call_original!(this, method_info)
}

#[unity::hook("App", "SaveDataMenu.SuspendConfirmDialog.YesDialogItem", "ACall")] // 0x71022EDA10
pub fn suspend_confirm_a_call_hook(
    this: SaveDataMenu_SuspendConfirmDialog_YesDialogItem,
    method_info: OptionalMethod,
) -> BasicMenu_Result {
    let ty = this.m_dest_save_data_header_handle().get_type();
    println!("Suspending game of type {}.", ty.value);

    if ty == GameSaveData_Types::manual() {
        let index = this.m_dest_save_data_header_handle().get_index();

        let unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let ledger = WEXP_LEDGER.get_cloned().unwrap().unwrap();

        write_unit_data(index, unit_data).expect("Failed to write UnitWexpData to file.");
        write_dynamic_unit_data(index, temp_data).expect("Failed to write TempUnitWexpData to file.");
        write_ledger_unit_data(index, ledger).expect("Failed to write WexpLedger to file.");

        let default_unit_data = read_default_unit_data().ok();
        let default_temp_data = read_default_dyn_unit_data().ok();
        let default_ledger = Some(WexpRewindLedger::new());

        UNIT_WEXP_DATA.set(default_unit_data).expect("Failed to set default UnitWexpData.");
        TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
        WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    }

    call_original!(this, method_info)
}

#[unity::hook("App", "SaveDataMenu.DeleteConfirmDialog.YesDialogItem", "ACall")] // 0x71022ED330
pub fn delete_confirm_a_call_hook(
    this: SaveDataMenu_DeleteConfirmDialog_YesDialogItem,
    method_info: OptionalMethod,
) -> BasicMenu_Result {
    let index = this.m_dest_save_data_header_handle().get_index();

    delete_unit_data(index).expect("Failed to delete UnitWexpData file.");
    delete_dynamic_unit_data(index).expect("Failed to delete TempUnitWexpData file.");
    delete_ledger_unit_data(index).expect("Failed to delete WexpLedger file.");

    call_original!(this, method_info)
}

#[unity::hook("App", "SaveDataMenu.CopyConfirmDialog.YesDialogItem", "ACall")] // 0x71022ECFF0
pub fn copy_confirm_a_call_hook(
    this: SaveDataMenu_CopyConfirmDialog_YesDialogItem,
    method_info: OptionalMethod,
) -> BasicMenu_Result {
    let from_ty = this.m_src_save_data_header_handle().get_type();

    if from_ty == GameSaveData_Types::manual() {
        let index = this.m_dest_save_data_header_handle().get_index();
        let from_index = this.m_src_save_data_header_handle().get_index();

        copy_unit_data(index, from_index).expect("Failed to copy UnitWexpData.");
        copy_dynamic_unit_data(index, from_index).expect("Failed to copy TempUnitWexpData.");
        copy_ledger_unit_data(index, from_index).expect("Failed to copy WexpLedger.");
    } else if from_ty == GameSaveData_Types::auto() {
        let index = this.m_dest_save_data_header_handle().get_index();

        copy_unit_data(index, 10).expect("Failed to copy UnitWexpData.");
        copy_dynamic_unit_data(index, 10).expect("Failed to copy TempUnitWexpData.");
        copy_ledger_unit_data(index, 10).expect("Failed to copy WexpLedger.");
    }

    call_original!(this, method_info)
}


#[unity::hook("App", "GameSaveDataUtil", "DeleteAll")] // 0x7102285C50
pub fn game_save_data_util_delete_all_hook(
    sup: ProcInst,
    method_info: OptionalMethod,
) {
    for i in 0..=10 { // 1 Autosave and 10 manual save slots
        delete_unit_data(i).expect("Failed to delete UnitWexpData file.");
        delete_dynamic_unit_data(i).expect("Failed to delete TempUnitWexpData file.");
        delete_ledger_unit_data(i).expect("Failed to delete WexpLedger file.");
    }

    call_original!(sup, method_info)
}

#[unity::hook("App", "MainSequence", "AutoSave")] // 0x7101EDE820
pub fn main_auto_save_hook(
    this: MainSequence,
    method_info: OptionalMethod,
) {
    let mut unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
    let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();

    unit_data.add(temp_data);
    let new_unit_data = unit_data.clone();

    write_unit_data(10, unit_data).expect("Failed to write UnitWexpData to file.");

    let default_temp_data = read_default_dyn_unit_data().ok();
    let default_ledger = Some(WexpRewindLedger::new());

    UNIT_WEXP_DATA.set(Some(new_unit_data)).expect("Failed to set UnitWexpData.");
    TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
    WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");

    call_original!(this, method_info)
}

#[unity::hook("App", "MapSequence", "AutoSave")] // 0x710236C9A0
pub fn map_auto_save_hook(
    this: MapSequence,
    method_info: OptionalMethod,
) {
    if this.can_auto_save() {
        let mut unit_data = UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();
        let temp_data = TEMP_UNIT_WEXP_DATA.get_cloned().unwrap().unwrap();

        unit_data.add(temp_data);
        let new_unit_data = unit_data.clone();

        write_unit_data(10, unit_data).expect("Failed to write UnitWexpData to file.");

        let default_temp_data = read_default_dyn_unit_data().ok();
        let default_ledger = Some(WexpRewindLedger::new());

        UNIT_WEXP_DATA.set(Some(new_unit_data)).expect("Failed to set UnitWexpData.");
        TEMP_UNIT_WEXP_DATA.set(default_temp_data).expect("Failed to set default TempUnitWexpData.");
        WEXP_LEDGER.set(default_ledger).expect("Failed to set default WexpLedger.");
    }

    call_original!(this, method_info)
}