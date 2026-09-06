
use engage::app::ExpSetter;
use engage::app::ExpSetter_ExpWindow;
use engage::app::GameIcon;
use engage::app::GameSkip;
use engage::app::GameUI_Priority;
use engage::app::IExpSequence;
use engage::app::ExpSequence;
use engage::app::IExpSetter;
use engage::app::IExpSetter_ExpWindow;
use engage::app::IExpSetter_ExpWindowMethods;
use engage::app::IGameColor;
use engage::app::IMapHistory_Base_1Methods;
use engage::app::IPad;
use engage::app::IPersonDataMethods;
use engage::app::IProcInstMethods;
use engage::app::ISingletonClass_1Methods;
use engage::app::ISingletonScriptableObject_1Methods;
use engage::app::IUnit;
use engage::app::IUnitMethods;
use engage::app::InfoUtil;
use engage::app::ItemData_Kinds;
use engage::app::MapHistory;
use engage::app::MapHistory_Rewind;
use engage::app::pad::Pad;
use engage::app::ProcInst;
use engage::app::Unit;
use engage::app::UnitStatus;
use engage::app::gamecolor::GameColor;
use engage::app::gametime::GameTime;
use engage::app::gameui::GameUI;
use engage::app::resourcemanager_2::ResourceManager_2;
use engage::app::textmeshmessage::TextMeshMessage;

use engage::tm_pro::ITMP_Text;
use engage::tm_pro::ITMP_TextMethods;
use engage::tm_pro::TextMeshProUGUI;

use engage::unity_engine::Animator;
use engage::unity_engine::GameObject;
use engage::unity_engine::IAnimatorMethods;
use engage::unity_engine::IComponentMethods;
use engage::unity_engine::IGameObjectMethods;
use engage::unity_engine::IObject_2Methods;
use engage::unity_engine::IRectTransformMethods;
use engage::unity_engine::ITransformMethods;
use engage::unity_engine::Object_2;
use engage::unity_engine::RectTransform;
use engage::unity_engine::Vector2;
use engage::unity_engine::Vector3;
use engage::unity_engine::ui::IImageMethods;
use engage::unity_engine::ui::ILayoutElementMethods;
use engage::unity_engine::ui::LayoutElement;
use engage::unity_engine::ui::image::Image;

use unity::{Cast, OptionalMethod};

use crate::game::data::add_dynamic_wexp;
use crate::game::data::get_old_wlvl;
use crate::game::data::get_total_wexp;
use crate::game::data::get_temp_wexp;
use crate::game::data::get_weapon_kind;
use crate::game::data::set_new_wlvl;
use crate::game::data::set_temp_wexp;
use crate::game::data::set_old_wlvl;
use crate::game::data::set_total_wexp;
use crate::game::rewind::write_to_ledger;
use crate::game::weaponlevel::can_gain_wexp;


#[unity::hook("App", "ExpSequence", "CreateBind")] // 0x71024E8C20
pub fn exp_sequence_create_hook(
    sup: ProcInst,
    unit: Unit,
    exp: i32,
    skill_point: i32,
    method_info: OptionalMethod,
) {
    let total_wexp = get_total_wexp(unit);
    if total_wexp != 0 && MapHistory::rewind_is_enable() {
        write_to_ledger(MapHistory_Rewind::get_instance().get_current_index());
    }

    let kind = ItemData_Kinds { value: get_weapon_kind() };
    if GameSkip::is_skip() && kind != ItemData_Kinds::none() && kind.value <= 9 {
        let old = unit.get_weapon_level(kind, true);
        set_old_wlvl(old.value);

        let (can_gain, wexp, offset, delta) = can_gain_wexp(unit, kind, total_wexp);
        if can_gain && wexp != 0 {
            add_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind, wexp);
            if wexp + offset >= delta {
                set_new_wlvl(old.value + 1);
            } else {
                set_new_wlvl(old.value);
            }
            set_total_wexp(unit, 0);
        } else if !can_gain {
            set_total_wexp(unit, 0);
            set_new_wlvl(old.value);
        }
    }

    call_original!(sup, unit, exp, skill_point, method_info);
}


fn modify_wdwexproot_prefab(wdw_exp_root: GameObject) {
    let unit_t = wdw_exp_root.get_transform().find_child("WdwExp/Unit0");
    let unit_p = unit_t.get_position();

    if unit_t.find_child("Icon").is_null() {
        // We copy a weapon icon game object from the UnitStatus prefab, which is perpetually loaded in the background.
        let unit_status_t = UnitStatus::m_game_object().get_transform();
        let weapon_icon_t = unit_status_t
            .find_child("Contents/Parameter/Contents/JobInfo/Content/WeaponLv/WeaponLv0/Icon");

        // The other game objects are copied from the WdwExpRoot prefab.
        let exp_gauge_t = unit_t.find_child("ExpGuage");    // Note the typo;
        let exp_value_t = unit_t.find_child("ExpValue");
        let exp_max_t = unit_t.find_child("Max");
        let increase_t = unit_t.find_child("IncreasedValue");

        // The original name of "Icon" doesn't need to be changed as the WdwExpRoot prefab does not contain
        // a game object with that name. Additionally, the icon does not need to be resized.
        let icon = Object_2::instantiate_3(weapon_icon_t.get_game_object())
            .try_cast::<GameObject>()
            .unwrap();
        icon.set_name("Icon");
        icon.set_active(true);
        let icon_t = icon.get_transform();
        let icon_p = icon_t.get_position();
        let shift_icon = Vector3 {
            x: unit_p.x - icon_p.x + 305.0,
            y: unit_p.y - icon_p.y - 76.0,
            z: unit_p.z - icon_p.z,
        };
        icon_t.translate_2(shift_icon);
        icon_t.set_parent(unit_t);

        let wexp_gauge = Object_2::instantiate_3(exp_gauge_t.get_game_object())
            .try_cast::<GameObject>()
            .unwrap();
        wexp_gauge.set_name("WexpGauge");
        let wexp_gauge_t = wexp_gauge
            .get_transform()
            .try_cast::<RectTransform>()
            .unwrap();
        let wexp_gauge_p = wexp_gauge_t.get_position();
        let shift_gauge = Vector3 {
            x: unit_p.x - wexp_gauge_p.x + 355.0,
            y: unit_p.y - wexp_gauge_p.y - 165.0,
            z: unit_p.z - wexp_gauge_p.z,
        };
        wexp_gauge_t.translate_2(shift_gauge);
        wexp_gauge_t.set_size_delta(Vector2 { x: 100.0, y: 8.0 });
        wexp_gauge_t.set_parent(unit_t);
        
        {
            let wexp_width_t = wexp_gauge_t
                .find_child("Width")
                .try_cast::<RectTransform>()
                .unwrap();
            wexp_width_t.set_size_delta(Vector2 { x: 100.0, y: 0.0 });
            
            // let wexp_shadow_t = wexp_width_t
            //     .find_child("Shadow")
            //     .try_cast::<RectTransform>()
            //     .unwrap();
            
            let wexp_front_t = wexp_width_t
                .find_child("Front")
                .try_cast::<RectTransform>()
                .unwrap();
            wexp_front_t.set_size_delta(Vector2 { x: 100.0, y: 0.0 });
        }

        let wexp_value = Object_2::instantiate_3(exp_value_t.get_game_object())
            .try_cast::<GameObject>()
            .unwrap();
        wexp_value.set_name("WexpValue");
        let wexp_value_t = wexp_value
            .get_transform()
            .try_cast::<RectTransform>()
            .unwrap();
        let wexp_value_p = wexp_value_t.get_position();
        let shift_value = Vector3 {
            x: unit_p.x - wexp_value_p.x + 521.0,
            y: unit_p.y - wexp_value_p.y - 165.0,
            z: unit_p.z - wexp_value_p.z,
        };
        wexp_value_t.translate_2(shift_value);
        wexp_value_t.set_size_delta(Vector2 { x: 113.0, y: 26.0 });
        wexp_value_t.set_parent(unit_t);
        
        {
            let wexp_message_t = wexp_value_t
                .find_child("Message")
                .try_cast::<RectTransform>()
                .unwrap();
            wexp_message_t.set_size_delta(Vector2 { x: 60.0, y: 26.0 });
            wexp_message_t.translate_2(Vector3 { x: 0.0, y: 7.0, z: 0.0 });
            
            {
                let message_text = wexp_message_t.get_game_object().get_component::<TextMeshProUGUI>();
                message_text.set_m_font_size_min(16.0);
                message_text.set_m_font_size_max(16.0);

                let message_layout = wexp_message_t.get_game_object().get_component::<LayoutElement>();
                message_layout.set_preferred_height(24.0_f32);

                let message_tmm = wexp_message_t.get_game_object().get_component::<TextMeshMessage>();
                Object_2::destroy_2(message_tmm);
            }
            
            let wexp_value_act_t = wexp_value_t
                .find_child("Value")
                .try_cast::<RectTransform>()
                .unwrap();
            wexp_value_act_t.set_size_delta(Vector2 { x: 51.0, y: 26.0 });
            wexp_value_act_t.translate_2(Vector3 { x: -53.0, y: 7.0, z: 0.0 });
            
            {
                let value_text = wexp_value_act_t.get_game_object().get_component::<TextMeshProUGUI>();
                value_text.set_m_font_size(26.0);
                value_text.set_m_font_size_base(26.0);

                let value_layout = wexp_value_act_t.get_game_object().get_component::<LayoutElement>();
                value_layout.set_preferred_width(50.0_f32);
                value_layout.set_preferred_height(24.0_f32);
            }
        }

        let wexp_max = Object_2::instantiate_3(exp_max_t.get_game_object())
            .try_cast::<GameObject>()
            .unwrap();
        wexp_max.set_name("WexpMax");
        let wexp_max_t = wexp_max
            .get_transform()
            .try_cast::<RectTransform>()
            .unwrap();
        let wexp_max_p = wexp_max_t.get_position();
        let shift_max = Vector3 {
            x: unit_p.x - wexp_max_p.x + 554.0,
            y: unit_p.y - wexp_max_p.y - 165.0,
            z: unit_p.z - wexp_max_p.z,
        };
        wexp_max_t.translate_2(shift_max);
        wexp_max_t.set_size_delta(Vector2 { x: 89.0, y: 24.0 });
        wexp_max_t.set_parent(unit_t);

        {
            let max_text = wexp_max_t.get_game_object().get_component::<TextMeshProUGUI>();
            max_text.set_m_font_size_min(20.0);
            max_text.set_m_font_size_max(26.0);

            let max_layout = wexp_max_t.get_game_object().get_component::<LayoutElement>();
            max_layout.set_preferred_width(50.0_f32);
            max_layout.set_preferred_height(24.0_f32);

            let max_tmm = wexp_max_t.get_game_object().get_component::<TextMeshMessage>();
            Object_2::destroy_2(max_tmm);
        }

        let wexp_increase = Object_2::instantiate_3(increase_t.get_game_object())
            .try_cast::<GameObject>()
            .unwrap();
        wexp_increase.set_name("WexpIncreasedValue");
        let wexp_increase_t = wexp_increase
            .get_transform()
            .try_cast::<RectTransform>()
            .unwrap();
        let wexp_increase_p = wexp_increase_t.get_position();
        let shift_increase = Vector3 {
            x: unit_p.x - wexp_increase_p.x + 627.0,
            y: unit_p.y - wexp_increase_p.y - 165.0,
            z: unit_p.z - wexp_increase_p.z,
        };
        wexp_increase_t.translate_2(shift_increase);
        wexp_increase_t.set_size_delta(Vector2 { x: 73.0, y: 24.0 });
        wexp_increase_t.set_parent(unit_t);

        {
            let increase_text = wexp_increase_t.get_game_object().get_component::<TextMeshProUGUI>();
            increase_text.set_m_font_size(26.0);
            increase_text.set_m_font_size_base(26.0);

            let increase_layout = wexp_increase_t.get_game_object().get_component::<LayoutElement>();
            increase_layout.set_preferred_width(66.0_f32);
            increase_layout.set_preferred_height(24.0_f32);
        }
    }
}


#[unity::hook("App", "ExpSequence", "Open")] // 0x71024E8870
pub fn exp_sequence_open_hook(
    this: ExpSequence,
    _method_info: OptionalMethod,
) {
    let parent = GameUI::get_canvas(GameUI_Priority::default());
    let wdw_exp_root = ResourceManager_2::instantiate("UI/Battle/WdwExp/Prefabs/WdwExpRoot", parent);

    if !wdw_exp_root.is_null() {
        // We add WEXP information to the WdwExpRoot prefab.
        modify_wdwexproot_prefab(wdw_exp_root);

        // Then, we complete the rest of the original function.
        let exp_setter = wdw_exp_root.get_component::<ExpSetter>();
        this.set_m_window(exp_setter);

        if !this.m_window().is_null() && !this.m_window().m_unit_window().is_null() {
            this.m_window().m_unit_window().setup_unit(this.m_unit(), this.m_exp());
            this.m_window().m_unit_window().update_add_exp(this.m_exp());
        }
    }
}

#[unity::hook("App", "ExpSequence", "Tick")] // 0x71024E8370
pub fn exp_sequence_tick_hook(
    this: ExpSequence,
    _method_info: OptionalMethod,
) {
    if GameSkip::is_skip() {
        let unit = this.m_unit();
        unit.add_exp(this.m_exp());
        unit.add_skill_point(this.m_skill_point());

        let kind = ItemData_Kinds { value: get_weapon_kind() };

        if kind == ItemData_Kinds::none() || kind.value > 9 {
            set_old_wlvl(0);
            set_new_wlvl(0);
            set_temp_wexp(0);
            set_total_wexp(unit, 0);
            this.next_imm();
            return
        }

        let old = unit.get_weapon_level(kind, true);
        set_old_wlvl(old.value);

        let temp_wexp = get_temp_wexp();

        let (can_gain, wexp, offset, delta) = can_gain_wexp(unit, kind, temp_wexp);
        if can_gain && wexp != 0 {
            add_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind, wexp);
            if wexp + offset >= delta {
                set_new_wlvl(old.value + 1);
            } else {
                set_new_wlvl(old.value);
            }
            set_total_wexp(unit, 0);
        } else if !can_gain {
            set_new_wlvl(old.value);
            set_total_wexp(unit, 0);
        }

        this.next_imm();
        return
    }

    let mut vsync = GameTime::get_vsync_delta_count().min(1);
    let temp_wexp = get_temp_wexp();
    if vsync > 0 {
        while vsync != 0 {
            if this.m_exp() < 1 && this.m_skill_point() < 1 && temp_wexp < 1 {
                this.next_imm();
                let wdw_anim = this.m_window().get_component_2::<Animator>();
                wdw_anim.play_2("Out");
                return
            }

            let a = Pad::a();
            let b = Pad::b();
            let pad = Pad::get_instance();

            let skip = if pad.is_null() || pad.m_npad_state().buttons.value & (a.value | b.value) == 0 {
                false
            } else {
                pad.m_old_buttons().value & (a.value | b.value) == 0
            };

            let unit = this.m_unit();
            let exp = this.m_exp();
            if exp > 0 {
                if skip {
                    unit.add_exp(exp);
                    this.set_m_exp(0);
                } else {
                    unit.add_exp(1);
                    this.set_m_exp(exp - 1);
                }
            }
            
            let sp = this.m_skill_point();
            if sp > 0 {
                if skip {
                    unit.add_skill_point(sp);
                    this.set_m_skill_point(0);
                } else {
                    unit.add_skill_point(1);
                    this.set_m_skill_point(sp - 1);
                }
            }

            let kind = ItemData_Kinds { value: get_weapon_kind() };
            if kind == ItemData_Kinds::none() || kind.value > 9 {
                set_temp_wexp(0);
                set_total_wexp(unit, 0);
            } else {
                if temp_wexp > 0 {
                    if skip {
                        let (can_gain, wexp, _offset, _delta) = can_gain_wexp(
                            unit,
                            kind,
                            temp_wexp
                        );
                        
                        if can_gain && wexp != 0 {
                            add_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind, wexp);
                        }

                        set_temp_wexp(0);
                        set_total_wexp(unit, 0);
                    } else {
                        let (can_gain, wexp, _offset, _delta) = can_gain_wexp(
                            unit,
                            kind,
                            1
                        );
                        
                        if can_gain && wexp != 0 {
                            add_dynamic_wexp(unit.m_person().get_name().to_rust_string(), kind, wexp);
                            set_temp_wexp(temp_wexp - 1);
                        } else {
                            set_temp_wexp(0);
                            set_total_wexp(unit, 0);
                        }
                    }
                }
            }            

            this.m_window().m_unit_window().update_unit(unit);
            this.m_window().m_unit_window().update_add_exp(this.m_exp());

            vsync -= 1;
        }
    }
}

#[unity::hook("App", "ExpSetter.ExpWindow", "SetupUnit")] // 0x7101E59770
pub fn expset_expwin_setup_unit_hook(
    this: ExpSetter_ExpWindow,
    unit: Unit,
    gain_exp: i32,
    method_info: OptionalMethod,
) {
    call_original!(this, unit, gain_exp, method_info);

    let root = this.m_root();
    if !root.is_null() {
        let wexp_icon = root.get_transform().find_child("Icon").get_game_object();
        let wexp_gauge = root.get_transform().find_child("WexpGauge/Width/Front").get_game_object();
        let wexp_value = root.get_transform().find_child("WexpValue").get_game_object();
        let wexp_max = root.get_transform().find_child("WexpMax").get_game_object();
        let wexp_inc = root.get_transform().find_child("WexpIncreasedValue").get_game_object();

        let kind = ItemData_Kinds { value: get_weapon_kind() };
        if kind == ItemData_Kinds::none() || kind.value > 9 {
            wexp_icon.set_active(false);
            wexp_gauge.get_transform().get_parent().get_parent().get_game_object().set_active(false);
            wexp_value.set_active(false);
            wexp_max.set_active(false);
            wexp_inc.set_active(false);

            return
        }

        let icon = wexp_icon.get_component::<Image>();
        let weapon_sprite = GameIcon::try_get_item_kind_4(kind, unit.get_job(), false);
        InfoUtil::try_set_sprite_2(icon, weapon_sprite);

        let start_wlvl = unit.get_weapon_level(kind, true).value;
        set_old_wlvl(start_wlvl);

        let gain_wexp = get_total_wexp(unit);
        set_temp_wexp(gain_wexp);

        let (can_gain, wexp, offset, delta) = can_gain_wexp(unit, kind, gain_wexp);
        if can_gain {
            InfoUtil::try_update_gauge(wexp_gauge, offset, delta);

            let wexp_text = wexp_value
                .get_transform()
                .find_child("Value")
                .get_game_object()
                .get_component::<TextMeshProUGUI>();
            InfoUtil::try_set_text_2(wexp_text, delta - offset);
        } else if wexp == -2 {
            set_temp_wexp(0);
            set_total_wexp(unit, 0);
            wexp_icon.set_active(false);
            wexp_gauge.get_transform().get_parent().get_parent().get_game_object().set_active(false);
            wexp_value.set_active(false);
            wexp_max.set_active(false);
            wexp_inc.set_active(false);
        } else if wexp == -3 {
            if delta > 0 {
                wexp_inc.set_active(false);
                InfoUtil::try_update_gauge(wexp_gauge, offset, delta);

                let wexp_text = wexp_value
                    .get_transform()
                    .find_child("Value")
                    .get_game_object()
                    .get_component::<TextMeshProUGUI>();
                InfoUtil::try_set_text_2(wexp_text, delta - offset);
            } else {
                wexp_inc.set_active(false);
                wexp_value.set_active(false);
                wexp_max.set_active(true);
                InfoUtil::try_update_gauge(wexp_gauge, 100, 100);

                let gauge_image = wexp_gauge.get_component::<Image>();
                gauge_image.set_material(this.m_max_color());

                set_temp_wexp(0);
                set_total_wexp(unit, 0);
            }
        } else if wexp == -4 {
            wexp_inc.set_active(false);
            wexp_value.set_active(false);
            wexp_max.set_active(true);
            InfoUtil::try_update_gauge(wexp_gauge, 100, 100);

            let gauge_image = wexp_gauge.get_component::<Image>();
            gauge_image.set_material(this.m_max_color());

            set_temp_wexp(0);
            set_total_wexp(unit, 0);
        }
    }
}

#[unity::hook("App", "ExpSetter.ExpWindow", "UpdateAddExp")] // 0x7101E59C20
pub fn expset_expwin_update_add_exp_hook(
    this: ExpSetter_ExpWindow,
    gain_exp: i32,
    method_info: OptionalMethod,
) {
    call_original!(this, gain_exp, method_info);

    let wexp_inc = this.m_root().get_transform().find_child("WexpIncreasedValue").get_game_object();
    let wexp_inc_text = wexp_inc.get_component::<TextMeshProUGUI>();

    let gain_wexp = get_temp_wexp();
    if gain_wexp != 0 {
        let gain_text = "+".to_string() + gain_wexp.to_string().as_str();
        wexp_inc_text.set_text_2(gain_text, true);

        let mut color = GameColor::get_instance().数値上昇();
        if gain_wexp < 1 {
            color = GameColor::get_instance().数値下降();
        }

        InfoUtil::try_set_color(wexp_inc_text, color);
    } else {
        InfoUtil::try_set_text_2(wexp_inc_text, 0);
    }
}

#[unity::hook("App", "ExpSetter.ExpWindow", "UpdateUnit")] // 0x7101E59A70
pub fn expset_expwin_update_unit_hook(
    this: ExpSetter_ExpWindow,
    unit: Unit,
    method_info: OptionalMethod,
) {
    call_original!(this, unit, method_info);

    if !unit.is_null() {
        let wexp_gauge = this.m_root().get_transform().find_child("WexpGauge/Width/Front").get_game_object();
        let wexp_value = this.m_root().get_transform().find_child("WexpValue").get_game_object();
        let wexp_max = this.m_root().get_transform().find_child("WexpMax").get_game_object();
        let wexp_inc = this.m_root().get_transform().find_child("WexpIncreasedValue").get_game_object();

        let kind = ItemData_Kinds { value : get_weapon_kind() };
        if kind == ItemData_Kinds::none() || kind.value > 9 {
            return
        }

        let gain_wexp = get_temp_wexp();
        let (can_gain, wexp, offset, delta) = can_gain_wexp(unit, kind, gain_wexp);
        if can_gain {
            InfoUtil::try_update_gauge(wexp_gauge, offset, delta);

            let wexp_value_text = wexp_value
                .get_transform()
                .find_child("Value")
                .get_game_object()
                .get_component::<TextMeshProUGUI>();
            InfoUtil::try_set_text_2(wexp_value_text, delta - offset);

            let old = get_old_wlvl();
            let new = unit.get_weapon_level(kind, true).value;

            if old < new {
                let gauge_image = wexp_gauge.get_component::<Image>();
                gauge_image.set_material(this.m_max_color());
                set_new_wlvl(new);
            }
        } else if wexp == -3 {
            let old = get_old_wlvl();
            let new = unit.get_weapon_level(kind, true).value;

            if old < new && delta > 0 {
                InfoUtil::try_update_gauge(wexp_gauge, offset, delta);

                let wexp_value_text = wexp_value
                    .get_transform()
                    .find_child("Value")
                    .get_game_object()
                    .get_component::<TextMeshProUGUI>();
                InfoUtil::try_set_text_2(wexp_value_text, delta - offset);
                
                let gauge_image = wexp_gauge.get_component::<Image>();
                gauge_image.set_material(this.m_max_color());
                set_new_wlvl(new);
            } else if old < new && delta == -1 {
                wexp_inc.set_active(false);
                wexp_value.set_active(false);
                wexp_max.set_active(true);
                InfoUtil::try_update_gauge(wexp_gauge, 100, 100);
                
                let gauge_image = wexp_gauge.get_component::<Image>();
                gauge_image.set_material(this.m_max_color());
                set_new_wlvl(new);
            } else {
                InfoUtil::try_update_gauge(wexp_gauge, offset, delta);
                
                let wexp_value_text = wexp_value
                    .get_transform()
                    .find_child("Value")
                    .get_game_object()
                    .get_component::<TextMeshProUGUI>();
                InfoUtil::try_set_text_2(wexp_value_text, delta - offset);

                set_new_wlvl(new);
            }

            set_temp_wexp(0);
            set_total_wexp(unit, 0);
        } else if wexp == -4 {
            wexp_inc.set_active(false);
            wexp_value.set_active(false);
            wexp_max.set_active(true);
            InfoUtil::try_update_gauge(wexp_gauge, 100, 100);

            let gauge_image = wexp_gauge.get_component::<Image>();
            gauge_image.set_material(this.m_max_color());

            set_temp_wexp(0);
            set_total_wexp(unit, 0);
        }
    }
}