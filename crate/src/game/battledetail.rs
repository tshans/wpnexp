
use engage::app::BattleDetail;
use engage::app::BattleInfoSide;
use engage::app::IBattleInfoSide;

use unity::OptionalMethod;

use crate::game::data::calculate_side_earned_wexp;
use crate::game::data::set_once_wexp;


#[unity::hook("App", "BattleDetail", "CalcBattle")] // 0x7101E74300
fn battle_detail_calc_battle_hook(
    this: BattleDetail,
    current: BattleInfoSide,
    reverse: BattleInfoSide,
    method_info: OptionalMethod,
) {
    call_original!(this, current, reverse, method_info);

    battledetail_calcwexp(this, current);
}

extern "C" fn battledetail_calcwexp(_this: BattleDetail, current: BattleInfoSide) {
    set_once_wexp(current.m_unit(), calculate_side_earned_wexp(current));
}

pub fn install_detail_hooks() {
    skyline::install_hook!(crate::game::battledetail::battle_detail_calc_battle_hook);
}