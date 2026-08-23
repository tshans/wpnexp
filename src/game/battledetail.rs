
use engage::app::BattleDetail;
use engage::app::BattleInfoSide;
use engage::app::IBattleInfoSide;
// use engage::app::IBattleInfoSideMethods;
// use engage::app::BattleInfoSide_Status;
// use engage::app::IBitFieldTemplate32_1Methods;
// use engage::app::IPersonDataMethods;
// use engage::app::IUnit;

// use unity::Cast;
use unity::OptionalMethod;

use crate::game::data::calculate_side_earned_wexp;
use crate::game::data::set_once_wexp;


#[unity::hook("App", "BattleDetail", "CalcBattle")] // 0x7101E74300
pub fn battle_detail_calc_battle_hook(
    this: BattleDetail,
    current: BattleInfoSide,
    reverse: BattleInfoSide,
    method_info: OptionalMethod,
) {
    call_original!(this, current, reverse, method_info);

    battledetail_calcwexp(this, current);
}

pub extern "C" fn battledetail_calcwexp(_this: BattleDetail, current: BattleInfoSide) {
    // if !reverse.is_null() && reverse.get_status().test(BattleInfoSide_Status::rod()) {
    //     set_once_wexp(current.m_unit(), 0);
    // } else if current.get_status().test(BattleInfoSide_Status::rod()) {
    //     let earned_wexp = calculate_side_earned_wexp(current);
    //     set_once_wexp(current.m_unit(), earned_wexp);
    // } else {
    //     let earned_wexp = calculate_side_earned_wexp(current);
    //     set_once_wexp(current.m_unit(), earned_wexp);
    // }
    set_once_wexp(current.m_unit(), calculate_side_earned_wexp(current));
}