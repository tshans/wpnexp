

use engage::BasicMenuItemAttribute;
use engage::BasicMenuResult;
use engage::ProcVoidMethodExt;

use engage::app::BasicDialogItemNo;
use engage::app::BasicDialogItemYes;
use engage::app::GameMessage;
use engage::app::GameSound;
use engage::app::IBasicMenu;
use engage::app::IBasicMenuMethods;
use engage::app::IGameMessageMethods;
use engage::app::IItemDataMethods;
use engage::app::IPersonDataMethods;
use engage::app::ISingletonClass_1Methods;
use engage::app::ISortieInventoryManager;
use engage::app::ISortieInventoryManager_SelectionInfoMethods;
use engage::app::IUnit;
use engage::app::IUnitItem;
use engage::app::IUnitMethods;
use engage::app::ItemData_Kinds;
use engage::app::ItemData_UseTypes;
use engage::app::Pad;
use engage::app::Proc;
use engage::app::ProcInst;
use engage::app::ProcVoidMethod;
use engage::app::SortieInventoryManager;
use engage::app::BasicMenu;
use engage::app::basicmenuitem::*;
use engage::app::BasicMenuItem;
use engage::app::Mess;
use engage::app::IProcInstMethods;
use engage::app::Unit;

use engage::app::sortieutil::SortieUtil;
use engage::app::basicdialogitemyes::*;
use engage::app::yesnodialog::YesNoDialog;

use engage::combat::Character;
use engage::nn::hid::NpadButton;

use engage::system::collections::generic::IList_1Methods;
use engage::system::collections::generic::List_1;

use unity::Il2CppString;
use unity::FromIlInstance;
use unity::prelude::*;

use crate::game::data::add_dynamic_wexp;
use crate::game::data::calculate_arms_scroll_delta;


#[unity::inject(
    namespace = "Wexp",
    name = "ArmsScrollYesDialog",
    parent = BasicDialogItemYes,
)]
pub struct ArmsScrollYesDialog {
    pub index: i32,
    pub unit: Unit,
    pub delta: i32,
    pub kind: ItemData_Kinds,
}

#[unity::injected_methods]
impl ArmsScrollYesDialog {
    #[override_virtual(name = "GetName")]
    pub fn get_name(self) -> Il2CppString {
        Mess::get("MID_MENU_YES")
    }

    #[override_virtual(name = "ACall")]
    pub fn a_call(self) -> BasicMenuResult {
        let name = match self.kind().value {
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
        };

        let unit = self.unit();
        SortieUtil::expend_item(unit, self.index());

        // For the Sortie menu, WEXP is increased immediately if the Yes option of the YesNoDialog is selected.
        add_dynamic_wexp(unit.m_person().get_name().to_rust_string(), self.kind(), self.delta());
        let new_wlvl = unit.get_weapon_level(self.kind(), true).value.to_string();

        let message = Mess::get_3("MID_MSG_WLVL_Increase", name, new_wlvl);
        GameSound::post_event("LevelUp_Short", Character::null());
        let mess = GameMessage::create_system(self.m_menu(), message);
        mess.set_shadow_off();
        
        BasicMenuResult::se_decide().with_close_all(true)
    }

    #[override_virtual(name = "BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub fn register_arms_scroll_yes_dialog() -> Class {
    let result = cobapi::injection::register::<ArmsScrollYesDialog>();
    match result {
        Ok(klass) => klass,
        Err(e) => panic!("Failed to register ArmsScrollYesDialog: {}.", e),
    }
}

#[no_mangle]
pub extern "C" fn yes_dialog_callback(index: i32, unit: Unit, delta: i32, kind: ItemData_Kinds) -> BasicDialogItemYes {
    let instance = ArmsScrollYesDialog::instantiate().unwrap();
    instance.set_index(index);
    instance.set_unit(unit);
    instance.set_delta(delta);
    instance.set_kind(kind);
    instance.try_cast::<BasicDialogItemYes>().unwrap()
}

#[unity::inject(
    namespace = "Wexp",
    name = "ArmsScrollMenuItem",
    parent = BasicMenuItem,
)]
pub struct ArmsScrollMenuItem {
    pub index: i32,
    pub unit: Unit,
    pub delta: i32,
    pub kind: ItemData_Kinds,
}

#[unity::injected_methods]
impl ArmsScrollMenuItem {
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

        let message = Mess::get_3(
            "MID_MSG_ArmsScroll_Used",
            self.get_name(),
            self.delta().to_string()
        );

        let yes = yes_dialog_callback(self.index(), unit, self.delta(), self.kind());

        YesNoDialog::create_bind(
            self.get_menu(),
            message,
            yes,
            BasicDialogItemNo::null(),
        );

        BasicMenuResult::se_decide()
    }

    #[override_virtual(name = "BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub fn register_arms_scroll_menu_item() -> Class {
    let result = cobapi::injection::register::<ArmsScrollMenuItem>();
    match result {
        Ok(klass) => klass,
        Err(e) => panic!("Failed to register ArmsScrollMenuItem: {}.", e),
    }
}

#[no_mangle]
pub extern "C" fn menu_item_callback(index: i32, unit: Unit, delta: i32, kind: ItemData_Kinds) -> BasicMenuItem {
    let instance = ArmsScrollMenuItem::instantiate().unwrap();
    instance.set_index(index);
    instance.set_unit(unit);
    instance.set_delta(delta);
    instance.set_kind(kind);
    instance.try_cast::<BasicMenuItem>().unwrap()
}

#[unity::inject(
    namespace = "Wexp",
    name = "ArmsScrollSortieSubmenu",
    parent = BasicMenuItem,
)]
pub struct ArmsScrollSortieSubmenu {}

#[unity::injected_methods]
impl ArmsScrollSortieSubmenu {
    #[override_virtual(name = "GetName")]
    pub fn get_name(self) -> Il2CppString {
        Mess::get("MID_SORTIE_INVENTORY_USE")
    }

    #[override_virtual(name = "CustomCall")]
    pub fn custom_call(self) -> BasicMenuResult {
        if !Pad::is_trigger(NpadButton::a()) {
            return BasicMenuResult::new()
        }

        let menu = self.get_menu();
        open_sortie_submenu(menu);

        let child = menu.get_child();
        let submenu: BasicMenu;
        if child.is_null() {
            return BasicMenuResult::new()
        } else if let Some(sub) = child.try_cast::<BasicMenu>() {
            submenu = sub;
        } else if child.get_child().is_null() {
            return BasicMenuResult::new()
        } else if let Some(sub2) = child.get_child().try_cast::<BasicMenu>() {
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

extern "C" fn sortie_submenu_start(seq: ProcInst, _method_info: OptionalMethod) {
    let menu_item_list = List_1::<BasicMenuItem>::new();

    let unit = SortieInventoryManager::get_instance().m_selection().get_unit();
    let index = unit.get_item_index("IID_ArmsScroll");
    for i in 1..=9 {
        let kind = ItemData_Kinds { value: i };
        let delta = calculate_arms_scroll_delta(unit, kind);
        match delta {
            Some(value) => {
                let menu_item = menu_item_callback(index, unit, value, kind);
                menu_item_list.add(menu_item);
            },
            None => continue,
        }
    }

    BasicMenu::create_basic_menu_bind(menu_item_list, seq);
}

pub fn open_sortie_submenu(menu: BasicMenu) {
    let seq = <ProcInst as FromIlInstance>::instantiate().unwrap();

    let steps = [
        Proc::label(0),
        Proc::call_2(
            ProcVoidMethod::from_fn(seq, sortie_submenu_start).unwrap()
            ),
        Proc::end(),
    ];

    let descs = Array::from_slice(&steps).unwrap();

    seq.create_bind(menu, descs, "ArmsScrollSortieSubmenu");
}

pub fn register_sortie_submenu() -> Class {
    let result = cobapi::injection::register::<ArmsScrollSortieSubmenu>();
    match result {
        Ok(klass) => klass,
        Err(e) => panic!("Failed to register ArmsScrollSortieSubmenu: {}.", e),
    }
}

#[no_mangle]
pub extern "C" fn sortie_submenu_callback() -> BasicMenuItem {
    let item = ArmsScrollSortieSubmenu::instantiate().unwrap();
    item.try_cast::<BasicMenuItem>().unwrap()
}

fn arms_scroll_can_grow(unit: Unit) -> bool {
    for i in 1..=9 {
        let delta = calculate_arms_scroll_delta(unit, ItemData_Kinds { value: i });
        match delta {
            Some(_value) => return true,
            None => continue
        }
    }

    false
}

#[unity::hook("App", "InventorySubMenu", "CreateBind")] // 0x710279C840
pub fn sortie_sub_menu_create_bind_hook(
    sup: ProcInst,
    parent_menu: BasicMenu,
    parent_menu_item: BasicMenuItem,
    method_info: OptionalMethod,
) {
    call_original!(sup, parent_menu, parent_menu_item, method_info);

    let child = sup.get_child();
    if child.is_null() {
        return;
    }

    let manager = SortieInventoryManager::get_instance();
    let unit = manager.m_selection().get_unit();
    let item = manager.m_selection().get_unit_item();
    if item.m_item().get_use_type() == ItemData_UseTypes::weapon_level_up()
        && arms_scroll_can_grow(unit) {
            // We replace the game's default "UseItem" menu option.
            let menu = child.try_cast::<BasicMenu>().unwrap();
            let map_item = sortie_submenu_callback();
            menu.m_full_menu_item_list().set_item(5, map_item); // "UseItem" index is "5".
        }
}