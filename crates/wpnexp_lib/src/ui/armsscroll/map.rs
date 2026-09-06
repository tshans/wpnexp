
use engage::BasicMenuItemAttribute;
use engage::BasicMenuResult;
use engage::ProcVoidMethodExt;

use engage::app::BasicDialog;
use engage::app::BasicDialogItem;
use engage::app::IBasicDialogMethods;
use engage::app::IBasicMenu;
use engage::app::IBasicMenuMethods;
use engage::app::IItemDataMethods;
use engage::app::IMapMindMethods;
use engage::app::ISingletonClass_1Methods;
use engage::app::IUnit;
use engage::app::IUnitGrowSequenceMethods;
use engage::app::IUnitItem;
use engage::app::IUnitMethods;
use engage::app::ItemData;
use engage::app::ItemData_Kinds;
use engage::app::ItemData_UseTypes;
use engage::app::MapMind;
use engage::app::MapMind_Type;
use engage::app::Pad;
use engage::app::Proc;
use engage::app::ProcInst;
use engage::app::ProcVoidMethod;
use engage::app::BasicMenu;
use engage::app::SkillArray;
use engage::app::UnitInfo;
use engage::app::UnitInfo_Side;
use engage::app::BasicMenuItem;
use engage::app::Mess;
use engage::app::IProcInstMethods;
use engage::app::ISingletonProcInst_1Methods;
use engage::app::Unit;

use engage::app::basicdialogitem::*;
use engage::app::basicmenuitem::*;
use engage::app::mapsequencehuman::MapSequenceHuman;
use engage::app::unitgrowsequence::UnitGrowSequence;

use engage::nn::hid::NpadButton;

use engage::system::collections::generic::IList_1Methods;
use engage::system::collections::generic::List_1;

use unity::Il2CppString;
use unity::FromIlInstance;
use unity::prelude::*;

use crate::game::data::calculate_arms_scroll_delta;
use crate::game::data::set_total_wexp;
use crate::game::data::set_weapon_kind;


#[unity::inject(
    namespace = "Wexp",
    name = "ArmsScrollMapDialogItem",
    parent = BasicDialogItem,
)]
pub struct ArmsScrollMapDialogItem {
    pub index: i32,
    pub unit: Unit,
    pub delta: i32,
    pub kind: ItemData_Kinds,
}

#[unity::injected_methods]
impl ArmsScrollMapDialogItem {
    #[override_virtual(name = "GetName")]
    pub fn get_name(self) -> Il2CppString {
        match self.kind().value {
            1 => Mess::get("MID_H_INFO_WLV_Sword"),
            2 => Mess::get("MID_H_INFO_WLV_Lance"),
            3 => Mess::get("MID_H_INFO_WLV_Axe"),
            4 => Mess::get("MID_H_INFO_WLV_Bow"),
            5 => Mess::get("MID_H_INFO_WLV_Dagger"),
            6 => Mess::get("MID_H_INFO_WLV_Magic"),
            7 => Mess::get("MID_H_INFO_WLV_Rod"),
            8 => Mess::get("MID_H_INFO_WLV_Fist"),
            9 => Mess::get("MID_H_INFO_WLV_Special"),
            _ => panic!("ItemData.Kinds value is invalid."),
        }
    }

    #[override_virtual(name = "ACall")]
    pub fn a_call(self) -> BasicMenuResult {
        let unit = self.unit();

        set_weapon_kind(self.kind().value);
        set_total_wexp(unit, self.delta());

        let grow_seq = UnitGrowSequence::create_bind(self.m_menu());
        grow_seq.set_unit_grow_data_3(unit, 0, 0, false);
    
        // We reproduce the MapItemMenu.SubItemUseMenuItem$$ACall function.
        let mind = MapMind::get_instance();
        mind.set_x(unit.m_x());
        mind.set_z(unit.m_z());
        mind.set_item_index(self.index());
        mind.set_mind(MapMind_Type::item_use());

        let map_seq = MapSequenceHuman::get_instance();
        map_seq.jump(46);

        UnitInfo::update_current_unit(UnitInfo_Side::left());

        BasicMenuResult::se_decide().with_close_all(true)
    }

    #[override_virtual(name = "BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub fn register_arms_scroll_map_menu_item() -> Class {
    let result = cobapi::injection::register::<ArmsScrollMapDialogItem>();
    match result {
        Ok(klass) => klass,
        Err(e) => panic!("Failed to register ArmsScrollMenuItem: {}.", e),
    }
}

#[no_mangle]
pub extern "C" fn map_menu_dialog_item_callback(index: i32, unit: Unit, delta: i32, kind: ItemData_Kinds) -> BasicMenuItem {
    let instance = ArmsScrollMapDialogItem::instantiate().unwrap();
    instance.set_index(index);
    instance.set_unit(unit);
    instance.set_delta(delta);
    instance.set_kind(kind);
    instance.try_cast::<BasicMenuItem>().unwrap()
}

#[unity::inject(
    namespace = "Wexp",
    name = "ArmsScrollMapSubmenu",
    parent = BasicMenuItem,
)]
pub struct ArmsScrollMapSubmenu {}

#[unity::injected_methods]
impl ArmsScrollMapSubmenu {
    #[override_virtual(name = "GetName")]
    pub fn get_name(self) -> Il2CppString {
        Mess::get("MID_MENU_ITEM_USE")
    }

    #[override_virtual(name = "CustomCall")]
    pub fn custom_call(self) -> BasicMenuResult {
        if !Pad::is_trigger(NpadButton::a()) {
            return BasicMenuResult::new()
        }

        let menu = self.get_menu();
        open_map_submenu(menu);

        let child = menu.get_child();
        let submenu: BasicDialog;
        if child.is_null() {
            return BasicMenuResult::new()
        } else if let Some(sub) = child.try_cast::<BasicDialog>() {
            submenu = sub;
        } else if child.get_child().is_null() {
            return BasicMenuResult::new()
        } else if let Some(sub2) = child.get_child().try_cast::<BasicDialog>() {
            submenu = sub2;
        } else {
            return BasicMenuResult::new()
        }

        submenu.set_transform_as_sub_menu(menu, self);

        BasicMenuResult::se_decide()
    }

    #[override_virtual(name = "ACall")]
    pub fn a_call(self) -> BasicMenuResult {
        BasicMenuResult::new()
    }

    #[override_virtual(name = "BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub fn open_map_submenu(menu: BasicMenu) {
    let seq = <ProcInst as FromIlInstance>::instantiate().unwrap();

    let steps = [
        Proc::label(0),
        Proc::call_2(
            ProcVoidMethod::from_fn(seq, map_submenu_start).unwrap()
            ),
        Proc::end(),
    ];

    let descs = Array::from_slice(&steps).unwrap();

    seq.create_bind(menu, descs, "ArmsScrollMapSubmenu");
}

extern "C" fn map_submenu_start(seq: ProcInst, _method_info: OptionalMethod) {
    let menu_item_list = List_1::<BasicMenuItem>::new();

    let unit = MapMind::get_instance().get_unit();
    let index = unit.get_item_index("IID_ArmsScroll");
    for i in 1..=9 {
        let kind = ItemData_Kinds { value: i };
        let delta = calculate_arms_scroll_delta(unit, kind);
        match delta {
            Some(value) => {
                let menu_item = map_menu_dialog_item_callback(index, unit, value, kind);
                menu_item_list.add(menu_item);
            },
            None => continue,
        }
    }

    let dialog = BasicDialog::create_basic_dialog_bind(seq, menu_item_list);
    dialog.set_text(Mess::get("MID_MSG_ArmsScroll_Use"));
}

pub fn register_map_submenu() -> Class {
    let result = cobapi::injection::register::<ArmsScrollMapSubmenu>();
    match result {
        Ok(klass) => klass,
        Err(e) => panic!("Failed to register ArmsScrollMapSubmenu: {}.", e),
    }
}

#[no_mangle]
pub extern "C" fn map_submenu_callback() -> BasicMenuItem {
    let item = ArmsScrollMapSubmenu::instantiate().unwrap();
    item.try_cast::<BasicMenuItem>().unwrap()
}

#[unity::hook("App", "MapItemHelper", "CanUseImpl")] // 0x7101DEA700
pub fn map_item_helper_can_use_hook(
    unit: Unit,
    item: ItemData,
    target: Unit,
    use_type: ItemData_UseTypes,
    give_skills: SkillArray,
    method_info: OptionalMethod,
) -> bool {
    if use_type == ItemData_UseTypes::weapon_level_up() {
        crate::game::data::arms_scroll_can_grow(unit)
    } else {
        call_original!(unit, item, target, use_type, give_skills, method_info)
    }
}

#[unity::hook("App", "MapItemMenu.SubItemMenu", "CreateBind")] // 0x710217FFC0
pub fn map_sub_menu_create_bind_hook(
    sup: BasicMenu,
    unit_item_index: i32,
    parent_menu_item: BasicMenuItem,
    method_info: OptionalMethod
) {
    call_original!(sup, unit_item_index, parent_menu_item, method_info);

    let child = sup.get_child();
    if child.is_null() {
        return;
    }

    let unit = MapMind::get_instance().get_unit();
    let item = unit.get_item(unit_item_index);
    if item.m_item().get_use_type() == ItemData_UseTypes::weapon_level_up()
        && crate::game::data::arms_scroll_can_grow(unit) {
            // We replace the game's default "UseItem" menu option.
            let menu = child.try_cast::<BasicMenu>().unwrap();
            let map_item = map_submenu_callback();
            menu.m_full_menu_item_list().set_item(2, map_item); // "UseItem" index is "2".
        }
}