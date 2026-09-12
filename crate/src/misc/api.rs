use engage::Mess;

use engage::app::IBattleDetail; 
use engage::app::IBattleInfo;
use engage::app::IBattleInfoSideMethods;
use engage::app::IBattleSide_ContainerArray_1;
use engage::app::IHelpParamSetter;
use engage::app::IHelpParamSetterMethods;
use engage::app::IItemDataMethods;
use engage::app::IItemMenuDetailSetter;
use engage::app::IItemMenuDetailSetterMethods;
use engage::app::IUnitItem;
use engage::app::IUnitItemMethods;
use engage::app::InfoUtil;
use engage::app::ItemData_Kinds;
use engage::app::TextMeshMessage;

use engage::tm_pro::TextMeshProUGUI;

use engage::unity_engine::ui::image::Image;
use engage::unity_engine::component::IComponentMethods;
use engage::unity_engine::gameobject::IGameObjectMethods;
use engage::unity_engine::object_2::IObject_2Methods;
use engage::unity_engine::transform::ITransformMethods;
use engage::unity_engine::Object_2;

use unity::Cast;

use crate::misc::statics::ONCE_WEXP_INDEX;
use crate::{calculate_side_earned_wexp, get_item_wexp};
use crate::register_weapon_level_calculator_commands;
use crate::register_wexp_calculator_commands;


extern "C" fn build_wdwitemhelp_wexp(args: &crtdapi::CreateDetails3Args) {
    let wexp_t = args.crit_copy_t;
    wexp_t.get_game_object().set_name("Wexp");
    let wexp_title_t = wexp_t.find_child("Title");
    Object_2::destroy_2(wexp_title_t
        .get_game_object()
        .get_component_3("TextMeshMessage")
        .try_cast::<TextMeshMessage>()
        .unwrap());
    let wexp_title_text = wexp_title_t
        .get_game_object()
        .get_component_3("TextMeshProUGUI")
        .try_cast::<TextMeshProUGUI>()
        .unwrap();
    let wexp_title = Mess::get("MID_SYS_Wexp");
    InfoUtil::try_set_text(wexp_title_text, wexp_title);
}

extern "C" fn build_helpparamsetter_wexp(args: &crtdapi::CreateDetails3Args) {
    let wexp_t = args.crit_copy_t;
    wexp_t.get_game_object().set_name("Wexp");
    let wexp_title_t = wexp_t.find_child("Title");
    Object_2::destroy_2(wexp_title_t
        .get_game_object()
        .get_component_3("TextMeshMessage")
        .try_cast::<TextMeshMessage>()
        .unwrap());
    let wexp_title_text = wexp_title_t
        .get_game_object()
        .get_component_3("TextMeshProUGUI")
        .try_cast::<TextMeshProUGUI>()
        .unwrap();
    let wexp_title = Mess::get("MID_SYS_Wexp");
    InfoUtil::try_set_text(wexp_title_text, wexp_title);
}

extern "C" fn battledetail_calcwexp(args: &crtdapi::DetailArgs) {
    let wexp = calculate_side_earned_wexp(args.current);
    args.this.m_base_params().set(*ONCE_WEXP_INDEX.lock().unwrap(), wexp);
}

extern "C" fn calculator_add(args: &crtdapi::CalculatorArgs) {
    register_weapon_level_calculator_commands(args.calculator);
    register_wexp_calculator_commands(args.calculator);
}

extern "C" fn setdata_soloitem_wexp(args: &crtdapi::SoloItemArgs) {
    if args.item.is_null() {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else if args.item.is_weapon() || args.item.is_rod() {
        let wpn_wexp_value = get_item_wexp(args.item.m_item());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, wpn_wexp_value.to_string());
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    }
}

pub extern "C" fn setdata_unititem_wexp(args: &crtdapi::UnitItemArgs) {
    if args.item.is_null() {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else if args.item.is_weapon() || args.item.is_rod() {
        // Old information
        let current = args.setter.m_battle_info().m_sides().m_array().get(0);
        let details = current.get_detail();

        // Updated information
        let tmp_current = args.setter.m_tmp_battle_info().m_sides().m_array().get(0);
        let tmp_details = tmp_current.get_detail();
        
        let wexp = details.m_base_params().get(*ONCE_WEXP_INDEX.lock().unwrap());
        let tmp_wexp = tmp_details.m_base_params().get(*ONCE_WEXP_INDEX.lock().unwrap());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, tmp_wexp.to_string());
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, wexp, tmp_wexp);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    }
}

pub extern "C" fn setdata_duoitem_wexp(args: &crtdapi::DuoItemArgs) {
    if args.base.is_null() || args.target.is_null() {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else if (args.base.is_weapon() || args.base.is_rod()) && (args.target.is_weapon() || args.target.is_rod()) {
        let wpn_wexp = get_item_wexp(args.base.m_item());
        let target_wpn_wexp = get_item_wexp(args.target.m_item());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, target_wpn_wexp.to_string());
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, wpn_wexp, target_wpn_wexp);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else if !(args.base.is_weapon() || args.base.is_rod()) && (args.target.is_weapon() || args.target.is_rod()) {
        let target_wpn_wexp = get_item_wexp(args.target.m_item());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, target_wpn_wexp.to_string());
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);

        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    }
}

pub extern "C" fn setdata_unitkinditem_wexp(args: &crtdapi::UnitKindItemArgs) {
    if args.item.is_null() {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);
        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else if args.item.is_weapon() || args.item.is_rod() {
        let wpn_wexp = get_item_wexp(args.item.m_item());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, wpn_wexp.to_string());
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);
        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    } else {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        let wexp_color = args.setter.try_set_up_down_icon(wexp_icon, 0, 0);
        InfoUtil::try_set_color(wexp_value_text, wexp_color);
    }
}

pub extern "C" fn helpparam_setitemdata_wexp(args: &crtdapi::UnitItemHelpParamArgs) {
    if args.item.is_null() {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        args.setter.try_set_up_down_icon(wexp_icon, wexp_value_text, 0, 0);
    } else if args.item.is_weapon() || args.item.get_kind() == ItemData_Kinds::rod() {
        // Old information
        let current = args.setter.m_battle_info().m_sides().m_array().get(0);
        let details = current.get_detail();

        // Updated information
        let tmp_current = args.setter.m_tmp_battle_info().m_sides().m_array().get(0);
        let tmp_details = tmp_current.get_detail();
    
        let wexp = details.m_base_params().get(*ONCE_WEXP_INDEX.lock().unwrap());
        let tmp_wexp = tmp_details.m_base_params().get(*ONCE_WEXP_INDEX.lock().unwrap());

        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, tmp_wexp.to_string());
        args.setter.try_set_up_down_icon(wexp_icon, wexp_value_text, wexp, tmp_wexp);
    } else {
        let wexp_icon = args.details3_t
            .find_child("Wexp/Value/Arrow")
            .get_game_object()
            .get_component_3("Image")
            .try_cast::<Image>()
            .unwrap();
        let wexp_value_text = args.details3_t
            .find_child("Wexp/Value")
            .get_game_object()
            .get_component_3("TextMeshProUGUI")
            .try_cast::<TextMeshProUGUI>()
            .unwrap();
        InfoUtil::try_set_text(wexp_value_text, "--");
        args.setter.try_set_up_down_icon(wexp_icon, wexp_value_text, 0, 0);
    }
}


pub fn init_api() {
    let bridge = crtdapi::lookup().unwrap();

    bridge.subscribe_wdwitemhelp(build_wdwitemhelp_wexp);
    bridge.subscribe_helpparamsetter(build_helpparamsetter_wexp);

    bridge.subscribe_battledetail_battleparam(battledetail_calcwexp);
    bridge.subscribe_calculator_addcommand(calculator_add);

    bridge.subscribe_setdata_soloitem(setdata_soloitem_wexp);
    bridge.subscribe_setdata_unititem(setdata_unititem_wexp);
    bridge.subscribe_setdata_duoitem(setdata_duoitem_wexp);
    bridge.subscribe_setdata_unitkinditem(setdata_unitkinditem_wexp);

    bridge.subscribe_helpparam_setitemdata(helpparam_setitemdata_wexp);
}