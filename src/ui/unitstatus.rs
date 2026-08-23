
use engage::app::Force_Type;
use engage::app::HelpParamSetter;
use engage::app::IHelpParamSetter;
use engage::app::IJobDataMethods;
use engage::app::IUnit;
use engage::app::IUnitMethods;
use engage::app::IUnitStatusSetter;
use engage::app::IUnitStatusSetter_WeaponLevelSetter;
use engage::app::IUnitStatusSetter_WeaponLevelSetterMethods;
use engage::app::IWeaponMaskMethods;
use engage::app::ItemData_Kinds;
use engage::app::JobData;
use engage::app::Mess;
use engage::app::Unit;
use engage::app::UnitInfo;
use engage::app::UnitInfo_Side;
use engage::app::UnitStatusSetter;
use engage::app::WeaponLevel_Kind;
use engage::app::infoutil::InfoUtil;
use engage::app::gameicon::GameIcon;
use engage::system::collections::generic::IList_1Methods;
use engage::tm_pro::ITMP_Text;
use engage::unity_engine::GameObject;
use engage::unity_engine::IComponentMethods;
use engage::unity_engine::IGameObjectMethods;
use engage::unity_engine::ITransformMethods;
use unity::Cast;
use unity::IlNull;
use unity::OptionalMethod;

use crate::game::data::calculate_unit_wexp_kind;
use crate::game::weaponlevel::wlvl_delta_kind;
use crate::game::weaponlevel::wlvl_kind_to_string;
use crate::game::weaponlevel::wlvl_to_wexp_kind;


#[unity::hook("App", "UnitStatusSetter", "SetWeaponLevel")] // 0x7101C68360
pub fn unit_status_set_wlvl_hook(
    this: UnitStatusSetter,
    base_unit: Unit,
    temp_unit: Unit,
    _method_info: OptionalMethod,
) {
    let weapon_lvl = this.m_weapon_level();
    let mut count = 0;
    for i in 1..=9 {
        if count >= weapon_lvl.get_count() {
            break
        }

        let setter = weapon_lvl.get_item(count);
        let kind = ItemData_Kinds { value: i };

        let base_wlvl = base_unit.get_weapon_level(kind, true);
        if base_unit.m_weapon_mask().test_2(kind) && base_wlvl != WeaponLevel_Kind::none() {
            InfoUtil::try_set_active(setter.m_root(), true);
            let weapon_sprite = GameIcon::try_get_item_kind_4(kind, temp_unit.m_job(), false);
            InfoUtil::try_set_sprite_2(setter.m_icon(), weapon_sprite);

            InfoUtil::try_set_text(setter.m_level(), wlvl_kind_to_string(base_wlvl));

            let job_max = base_unit.m_job().get_max_weapon_level_2(i, base_unit.m_original_aptitude());
            let is_limit = base_wlvl == job_max;
            UnitStatusSetter::set_text_color(setter.m_level(), 0, is_limit);

            // Set Gauge values for player units
            if base_unit.get_force_type() == Force_Type::player() {
                InfoUtil::try_set_active(setter.m_root().get_transform().find_child("WexpGauge").get_game_object(), true);
                if is_limit {
                    InfoUtil::try_update_child_gauge(
                        setter.m_root().get_transform().find_child("WexpGauge").get_game_object(),
                        0,
                        1,
                        true
                    );
                } else {
                    let unit_wexp = calculate_unit_wexp_kind(base_unit, kind);
                    let wexp_offset = wlvl_to_wexp_kind(base_wlvl);
                    let progress = unit_wexp - wexp_offset;
                    let to_next_wlvl = wlvl_delta_kind(base_wlvl);
                    InfoUtil::try_update_child_gauge(
                        setter.m_root().get_transform().find_child("WexpGauge").get_game_object(),
                        progress,
                        to_next_wlvl,
                        false
                    );
                }
            } else {
                InfoUtil::try_set_active(setter.m_root().get_transform().find_child("WexpGauge").get_game_object(), false);
            }

            count += 1;
        }
    }

    while count < weapon_lvl.get_count() {
        let setter = weapon_lvl.get_item(count);
        setter.set(ItemData_Kinds::none(), WeaponLevel_Kind::none(), 0, JobData::null());
        count += 1;
    }
}

#[unity::hook("App", "HelpParamSetter", "SetWeaponLevel")] // 0x7102162370
pub fn help_param_set_wlvl_hook(
    this: HelpParamSetter,
    frame: GameObject,
    item_kind: ItemData_Kinds,
    job_data: JobData,
    method_info: OptionalMethod,
) {
    call_original!(this, frame, item_kind, job_data, method_info);

    let mut unit = this.m_tmp_calc_unit();
    if unit.is_null() {
        unit = UnitInfo::get_unit(UnitInfo_Side::left());
    }

    if unit.get_force_type() == Force_Type::player() && item_kind.value < 10 {
        let base_text = this.m_contents_text().m_text().to_rust_string();

        let unit_wlvl = unit.get_weapon_level(item_kind, true);
        let max_wlvl = job_data.get_max_weapon_level_2(item_kind.value, unit.m_original_aptitude());
        let mut wexp_text = Mess::get("MID_H_INFO_WEXP_MAX").to_rust_string();
        if unit_wlvl != max_wlvl {
            let wexp = calculate_unit_wexp_kind(unit, item_kind);
            let progress = wexp - wlvl_to_wexp_kind(unit_wlvl);
            let delta = wlvl_delta_kind(unit_wlvl);
            Mess::set_argument(0, delta - progress);
            let wlvl_text = Mess::get("MID_H_INFO_WEXP");
            wexp_text = wlvl_text.to_rust_string();
        }

        let final_text = base_text + "\n" + wexp_text.as_str();

        InfoUtil::try_set_text(this.m_contents_text(), final_text);
    }
}