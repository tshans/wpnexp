

use std::collections::HashSet;
use std::ops::Shr;

use engage::app::ChapterData;
use engage::app::ClassChange;
use engage::app::Force_Type;
use engage::app::GameUserData;
use engage::app::GodData;
use engage::app::GodData_Flags;
use engage::app::GodUnit;
use engage::app::IBitField32Methods;
use engage::app::IBitField64;
use engage::app::IBitField64Methods;
use engage::app::IBitFieldTemplate32_1Methods;
use engage::app::IBitFieldTemplate64_1Methods;
use engage::app::IChapterDataMethods;
use engage::app::IClassChange_ChangeJobDataMethods;
use engage::app::IGameUserData;
use engage::app::IGameVariable;
use engage::app::IGodDataMethods;
use engage::app::IGodUnit;
use engage::app::IGodUnitMethods;
use engage::app::IItemData;
use engage::app::IItemDataMethods;
use engage::app::IJobDataMethods;
use engage::app::IMapImage;
use engage::app::IMapImageCore_1Methods;
use engage::app::IMapImageMethods;
use engage::app::IMapImageTerrain;
use engage::app::IPersonDataMethods;
use engage::app::IRingDataMethods;
use engage::app::ISingletonClass_1Methods;
use engage::app::ISkillArray;
use engage::app::ISkillArrayMethods;
use engage::app::ISkillData;
use engage::app::ISkillDataMethods;
use engage::app::IStructBase;
use engage::app::IStructData_1Methods;
use engage::app::ITerrainData_2Methods;
use engage::app::IUnit;
use engage::app::IUnitEditMethods;
use engage::app::IUnitItem;
use engage::app::IUnitItemListMethods;
use engage::app::IUnitItemMethods;
use engage::app::IUnitMethods;
use engage::app::IUnitRing;
use engage::app::IWeaponLevelsMethods;
use engage::app::IWeaponMaskMethods;
use engage::app::ItemData;
use engage::app::ItemData_Flags;
use engage::app::MapHistory;
use engage::app::SkillArray;
use engage::app::SkillData;
use engage::app::SkillData_Categorys;
use engage::app::SkillData_Flags;
use engage::app::TerrainData_2;
use engage::app::TerrainData_Flags;
use engage::app::Unit;
use engage::app::ItemData_Kinds;
use engage::app::Unit_Status;
use engage::app::UnitItem;
use engage::app::WeaponLevel_Kind;
use engage::app::WeaponMask;
use engage::app::classchange::ClassChange_ChangeJobData;
use engage::app::godpool::GodPool;
use engage::app::interactdata::InteractData;
use engage::app::map::Map;
use engage::app::mapimage::MapImage;
use engage::app::mapitemhelper::MapItemHelper;
use engage::app::unitactors::UnitActors;
use engage::app::unitpool::UnitPool;
use engage::app::versus::Versus;
use engage::system::collections::generic::IDictionary_2Methods;
use engage::system::collections::generic::IList_1;

use engage::system::collections::generic::IList_1Methods;
use unity::Array;
use unity::Cast;
use unity::FromIlInstance;
use unity::Il2CppString;
use unity::IlNull;
use unity::OptionalMethod;

use crate::game::data::calculate_unit_wexp_kind;
use crate::game::data::calculate_unit_wexp_kind_job;
use crate::game::data::get_job_wexp;
use crate::game::weaponlevel::wexp_to_wlvl_kind;
use crate::game::weaponlevel::wexp_to_wlvl_string;


// This is a list of functions which rely on the two App.JobData$$GetMaxWeaponLevel functions.
// Unfortunately, the Job functions are frequently used in places where App.Unit$$GetWeaponLevel would be more
// appropriate since the Job/Unit functions frequently coincide under Engage's somewhat odd weapon level system.



#[unity::hook("App", "ClassChange.ChangeJobData", "GetDispWeaponLevel")] // 0x71019c6590
pub fn class_change_get_disp_wlvl_hook(
    this: ClassChange_ChangeJobData,
    kind: ItemData_Kinds,
    unit: Unit,
    is_up: &mut bool, // This is a C# Out argument
    _method_info: OptionalMethod
) -> Il2CppString {
    // This function grabs the WLVL of the selected reclass option in the ClassChange menu.
    // It has been changed to show the unit's WLVL in the new class (instead of the new class's max weapon level).
    // In addition to WLVL differences, the UI will include an up arrow for a larger job WEXP offset in the new class.
    if this.get_job().is_equipable(kind) {
        let current_wexp = calculate_unit_wexp_kind(unit, kind);
        let reclass_wexp = calculate_unit_wexp_kind_job(unit, kind, this.get_job());
        *is_up = current_wexp < reclass_wexp;
        
        let wlvl = wexp_to_wlvl_string(reclass_wexp);
        Il2CppString::from(wlvl)
    } else {
        *is_up = false;
        "".into()
    }
}

// #[unity::hook("App", "Unit", "SetAptitudeFromDispos")] // 0x7101A3DE50
// This function is used to set Innate Proficiencies for enemy units so they can use higher tier weapons,
// e.g., enemy warriors (C+ bows) with a Silver Bow (B rank) will have their bow rank increased to B.
// Because this should only be used for enemy units, I'll leave it unchanged.

// #[unity::hook("App", "Unit", "CreateEncountPost")] // 0x7101A09520
// This function is used to set enemy Unit data for random encounters in a similar fashion to SetAptitudeFromDispos.
// It should be fine as-is...


#[unity::hook("App", "Unit", "GetWeaponLevel")] // 0x7101A0C050
pub fn unit_get_weapon_level_hook(
    this: Unit,
    kind: ItemData_Kinds,
    calc_enhance: bool,
    method_info: OptionalMethod,
) -> WeaponLevel_Kind {
    // This function needs to be rebuilt for player units, as it uses max Job WLVL and Skill WLVL to determine WLVL.
    if this.get_force_type() == Force_Type::player() && !this.is_summon() {
        if calc_enhance {
            let skill_wlvl = this.get_mask_skill().get_weapon_levels().get_item(kind);
            let wexp = calculate_unit_wexp_kind(this, kind);
            let wexp_wlvl = wexp_to_wlvl_kind(wexp);
            if skill_wlvl.value < wexp_wlvl.value {
                wexp_wlvl
            } else {
                skill_wlvl
            }
        } else {
            // Removes Job WEXP offset, yielding the raw WEXP value.
            let wexp = calculate_unit_wexp_kind(this, kind) - get_job_wexp(this.get_job(), kind);
            wexp_to_wlvl_kind(wexp)
        }
    } else {
        call_original!(this, kind, calc_enhance, method_info)
    }
}

// Helper functions for Unit$$UpdateStateImpl
pub fn add_skills(this: Unit, to: SkillArray, from: SkillArray) {
    if !from.is_null() {
        let mut update = false;
        for i in from.m_list().to_array().as_mut_slice() {
            let skill = SkillData::get_2((i.value & 4095) as i32);
            if !skill.is_null() {
                let style_skills = skill.m_style_skills();
                let style_skill = style_skills.get(this.m_job().get_style().value as usize);
                if (style_skill.get_flag().value + 5i64).shr(5) & 1i64 == 0i64
                    && (IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_whole_skill())
                        || (IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_equip_skill())
                            && i.get_category() == SkillData_Categorys::equip())) {
                                continue
                            }
                
                let added = to.add_without_update(style_skill, i.get_category(), i.get_age());
                update |= added;
            }
        }

        if update {
            to.update();
        }
    }
}

pub fn add_skills_god(this: Unit, to: SkillArray, from: SkillArray) {
    if !from.is_null() {
        let mut update = false;
        for i in from.m_list().to_array().as_mut_slice() {
            let skill = SkillData::get_2((i.value & 4095) as i32);
            if !skill.is_null() {
                let style_skills = skill.m_style_skills();
                let style_skill = style_skills.get(this.m_job().get_style().value as usize);
                if (style_skill.get_flag().value + 5i64).shr(5) & 1i64 == 0i64
                    && IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_whole_skill()) {
                        continue
                    }
                
                let added = to.add_without_update(style_skill, SkillData_Categorys::god(), i.get_age());
                update |= added;
            }
        }

        if update {
            to.update();
        }
    }
}

pub fn add_skill(this: Unit, to: SkillArray, skill: SkillData, category: SkillData_Categorys, age: i32) {
    let mut update = false;
    if !skill.is_null() {
        let style_skills = skill.m_style_skills();
        let style_skill = style_skills.get(this.m_job().get_style().value as usize);
        if (style_skill.get_flag().value + 5i64).shr(5) & 1i64 == 0i64
            && IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_whole_skill()) {
                return
            }
        
        let added = to.add_without_update(style_skill, category, age);
        update |= added;
    }

    if update {
        to.update();
    }
}

#[unity::hook("App" "Unit", "UpdateStateImpl")] // 0x7101A12020
pub fn unit_update_state_impl_hook(
    this: Unit,
    is_auto_equip: bool,
    equipped: UnitItem,
    _method_info: OptionalMethod,
) {
    if this.m_person().is_null() || this.m_job().is_null() {
        return
    }

    if IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::locked_update()) {
        let mut status = this.m_status().get_value();
        status &= !Unit_Status::locked_update().value;
        this.m_status().set_m_value(status);
        return
    } else {
        let mut status = this.m_status().get_value();
        status |= Unit_Status::locked_update().value;
        this.m_status().set_m_value(status);
    }

    // Do I need to threadlock here?
    // let skill_lock = this.m_mask_skill_lock();
    
    let mask_skill = this.m_mask_skill();
    mask_skill.clear();

    let person_mask = this.m_person().get_mask_skill();
    add_skills(this, mask_skill, person_mask);

    if !IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::vision()) {
        let job_mask = this.m_job().get_mask_skill();
        add_skills(this, mask_skill, job_mask);

        let learned_job_skill = this.m_learned_job_skill();
        add_skill(this, mask_skill, learned_job_skill, SkillData_Categorys::job(), 0);
    }

    let equip_skill = this.m_equip_skill();
    add_skills(this, mask_skill, equip_skill);

    let private_skill = this.m_private_skill();
    add_skills(this, mask_skill, private_skill);

    if !IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_supported_skill()) {
        let supported_skill = this.m_supported_skill();
        add_skills(this, mask_skill, supported_skill);
    }

    let mut god = this.m_god_link();
    if god.is_null() {
        god = this.m_god_unit();
    }

    if god.is_null() || IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_god_unit()) {
        let ring = this.m_ring();
        if !ring.is_null() && !ring.m_data().is_null() && !ring.m_data().get_equip_skills().is_null() {
            let ring_skills = ring.m_data().get_equip_skills();
            add_skills(this, mask_skill, ring_skills);
        }
        this.m_item_list().put_engage_item(GodUnit::null(), false);
    } else {
        let mut engaged = true;
        if !IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::engaging()) {
            engaged = god.get_engage_limit() == 0;
        }

        let synchro_skills = god.get_syncro_skills_2();
        add_skills_god(this, mask_skill, synchro_skills);

        if engaged && !god.is_null() {
            let engage_skills = god.get_engage_skills(this);
            add_skills_god(this, mask_skill, engage_skills);
        }

        this.m_item_list().put_engage_item(god, engaged);
    }

    if IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::engage_attack()) {
        let engage_attack = this.get_engage_attack();
        add_skill(this, mask_skill, engage_attack, SkillData_Categorys::god(), 0);
    }

    for i in 0..=7 {
        let item = this.m_item_list().get_item(i);
        if !item.is_null()
            && item.m_index() != 0
            && !item.m_item().is_null()
            && !item.m_item().get_passive_skills().is_null() {
                let item_passive_skills = item.m_item().get_passive_skills();
                add_skills(this, mask_skill, item_passive_skills);
            }
    }

    this.update_weapon_mask();

    let mut current_item = equipped;
    if equipped.is_null() {
        current_item = this.get_item_equipped();
    }

    if is_auto_equip {
        if current_item.is_null() 
            || !this.can_item_equip(current_item, current_item.get_kind() == ItemData_Kinds::rod(), true) {
                this.item_equip();
            }

        current_item = this.get_item_equipped();
    }

    if !current_item.is_null() {
        let mut equipped_item_skills = SkillArray::null();
        let item_god = current_item.m_god_unit();
        if item_god.is_null() {
            if UnitItem::s_enchant_hash() == 0 {
                equipped_item_skills = current_item.m_item().get_equip_skills();
            } else {
                let item_hash = current_item.m_item().m_enchant_hash();
                let static_hash = UnitItem::s_enchant_hash();
                if item_hash == static_hash {
                    equipped_item_skills = current_item.m_item().get_enchant_skills(0);
                }
            }
        } else {
            let iid = current_item.get_iid();
            let mut enchanted = false;
            if UnitItem::s_enchant_hash() != 0 {
                let item_hash = current_item.m_item().m_enchant_hash();
                let static_hash = UnitItem::s_enchant_hash();
                if item_hash == static_hash {
                    enchanted = true;
                }
            }
            
            equipped_item_skills = item_god.try_get_god_weapon_refine_result_equip_skills(iid, enchanted);
        }

        add_skills(this, mask_skill, equipped_item_skills);
    }

    mask_skill.commit(this);

    mask_skill.sort();

    if IBitFieldTemplate64_1Methods::test(this.m_status(), Unit_Status::ignore_equip_enhance()) {
        this.commit_enhance(UnitItem::null());
    } else {
        this.commit_enhance(current_item);
    }

    let engage_count = this.m_engage_count();
    let mut max_count = 0;
    if god.is_null() {
        let god_data = this.m_person().get_link_god();
        if !god_data.is_null() && !GodPool::try_get_2(god_data, false).is_null() {
            max_count = this.m_person().get_link_god().get_engage_count();
        }
    } else {
        let limit = god.get_engage_limit();
        let mut sub_god = this.m_god_link();
        if sub_god.is_null() {
            sub_god = this.m_god_unit();
        }
        let sub_limit = sub_god.can_sub_engage_count_limit_2() as i32;
        let skill_adjustment = (mask_skill.m_flags().value.shr(42) & 1i64) as i32;
        max_count = (limit - sub_limit - skill_adjustment).max(0) as u8;
    }
    this.set_engage_count(engage_count.clamp(0, max_count));

    let engage_turn = this.m_engage_turn();
    this.set_engage_turn(engage_turn.clamp(0, this.get_engage_turn_limit() as u8));

    this.set_item_selected(current_item);

    // Unlock thread here?

    let mut status = this.m_status().get_value();
    status &= !Unit_Status::locked_update().value;
    this.m_status().set_m_value(status);
}

#[skyline::hook(offset = 0x1A21530)] // 0x7101A21530
pub fn unit_item_equip_hook(
    this: Unit,
    _method_info: OptionalMethod,
) -> bool {
    for i in 0..=7 {
        match this.item_equip_2(i, true) {
            true => return true,
            false => continue,
        }
    }

    false
}

#[skyline::hook(offset = 0x1A282D0)] // 0x7101A282D0
pub fn unit_item_equip_options_hook(
    this: Unit,
    index: i32,
    reoder: bool,
    _method_info: OptionalMethod,
) -> bool {
    if index == -1 {
        return this.item_equip()
    }

    let unit_item = this.m_item_list().get_item(index);
    
    if unit_item.is_null() {
        return false
    }

    let item = unit_item.m_item();
    if item.is_null() {
        return false
    }

    let item_kind = item.get_kind();

    if item_kind.value > 9          // Non-equippable items
        || item_kind.value == 0     // None
        || item_kind.value == 7     // Staves
        || !IWeaponMaskMethods::test_2(this.m_weapon_mask(), item_kind) {
            return false
        }

    let item_flag = item.get_flag();
    if item_flag.get_value() & 384 == 0 { // ItemData.Flags{value: 384} = IgnoreWeaponLevel + Engage
        let item_wlvl = item.m_weapon_level();
        let unit_wlvl = this.get_weapon_level(item_kind, true);
        if item_wlvl.value > unit_wlvl.value {
            return false
        }
    }

    let equip_cond = item.get_equip_condition();
    if !equip_cond.is_null() && equip_cond.to_rust_string() != "" {
        let equip_skill = SkillData::try_get(equip_cond);
        if equip_skill.is_null() {
            return false
        }

        let equip_index = equip_skill.index();
        let test_skill = this.get_mask_skill().m_mask().get_item(equip_index);
        if !test_skill {
            return false
        }
    }

    if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_male()) {
        loop {
            if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                break
            }

            let mut force_ty = this.get_force_type();
            if this.get_force().is_null() {
                force_ty = Force_Type::empty();
            }

            let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
            if test_unit.is_null()
                || test_unit.get_force().is_null()
                || test_unit.get_force_type().value > 2
                || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                    break
                }
        }

        let edit_unit = this.get_edit();
        let person_unit = this.get_person();
        let mut unit_gender = person_unit.get_gender();
        if edit_unit.is_enable() {
            unit_gender = edit_unit.get_gender();
        }

        if unit_gender.value == 2 {
            return false
        }
    }

    if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_female()) {
        loop {
            if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                break
            }

            let mut force_ty = this.get_force_type();
            if this.get_force().is_null() {
                force_ty = Force_Type::empty();
            }

            let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
            if test_unit.is_null()
                || test_unit.get_force().is_null()
                || test_unit.get_force_type().value > 2
                || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                    break
                }
        }

        let edit_unit = this.get_edit();
        let person_unit = this.get_person();
        let mut unit_gender = person_unit.get_gender();
        if edit_unit.is_enable() {
            unit_gender = edit_unit.get_gender();
        }

        if unit_gender.value != 2 {
            return false
        }
    }

    if IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::engaging()) {
        let mut god = this.get_god_unit();
        let mut god_data: GodData;
        let parent: Unit;

        if !this.m_god_link().is_null() {
            god = this.m_god_link();
            parent = god.get_parent();
        } else if !god.is_null() {
            parent = god.get_parent();
        } else {
            parent = Unit::null();
        }

        if parent.is_null() {
            god_data = god.get_data();
        } else {
            god_data = god.get_data();

            if IBitFieldTemplate64_1Methods::test(parent.get_status(), Unit_Status::engaging()) {
                god_data = god_data.get_main_data();
            }
        }

        if IBitFieldTemplate32_1Methods::test(god_data.get_flag(), GodData_Flags::only_engage_weapon())
            && !IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::engage()) {
                return false
            }
    }

    this.m_item_list().equip(index);
    if reoder {
        this.m_item_list().r#move(index, 0);
    }
    this.update_state_impl(true, UnitItem::null());
    true
}

#[unity::hook("App", "Unit", "CanBreakable")] // 0x7101A241F0
pub fn unit_can_breakable_hook(
    this: Unit,
    target: Unit,
    _method_info: OptionalMethod,
) -> bool {
    if target.is_null() {
        return false
    }

    let mut selected_item = this.m_item_selected();
    if selected_item.is_null() {
        selected_item = this.m_item_list().get_equipped();
    }

    let mut target_item = target.m_item_list().get_equipped();
    if target.m_mask_skill().m_flags().value & SkillData_Flags::revenge_auto_equip().value != 0 {
        target_item = target.get_revenge_weapon(this, selected_item, -1);
    }

    if !selected_item.is_null() && !target_item.is_null() {
        let interact = InteractData::get_interact(selected_item, target_item);

        if interact.value != 1 {
            return false
        } else if (target.m_mask_skill().m_bad_ignore().value).shr(10) == 0 {
            let map_image = MapImage::get_instance();
            let x = target.m_x() as i32;
            let z = target.m_z() as i32;
            if -1 < ((map_image.get_play_area_z2() - z) * (z - map_image.get_play_area_z1()))
                | ((map_image.get_play_area_x2() - x) * (x - map_image.get_play_area_x1()))
                && Map::test_terrain_flag(x, z, TerrainData_Flags::not_stun()) {
                        return false
                    }
            
            let game_user_data = GameUserData::get_instance();
            let (exist, variable) = game_user_data
                .m_variable()
                .m_dictionary()
                .try_get_value("禁止_ブレイク".into()); // Break Prohibited
            
            return (variable.number == 0) | exist
        }
    }

    false
}

#[unity::hook("App", "Unit", "CanEnemyEngageAttack")] // 0x7101A289E0
pub fn unit_can_enemy_engage_attack_hook(
    this: Unit, 
    _method_info: OptionalMethod,
) -> bool {
    let mut god = this.m_god_link();
    let mut parent = Unit::null();
    if !god.is_null() {
        parent = god.m_parent();
    } else {
        god = this.m_god_unit();
        if !god.is_null() {
            parent = god.m_parent();
        }
    }

    let mut force_type = god.m_data().get_force_type();
    if !parent.is_null() {
        let status = parent.m_status();
        let mut god_data = god.m_data();
        if IBitFieldTemplate64_1Methods::test(status, Unit_Status::engaging()) {
            god_data = god.m_data().get_main_data();
        }

        force_type = god_data.get_force_type();
    }

    if force_type.value == 1 && !Versus::is_valid() {
        let engage_skill = this.get_engage_attack();
        if engage_skill.is_null() {
            return false
        }

        let equipped = this.get_skill_equip(engage_skill, -1);
        if !equipped.is_null() && !equipped.m_item().is_null() {
            match equipped.m_item().get_use_type().value {
                1 => return true,
                _ => return false,
            }
        }

        for i in 0..=7 {
            match this.can_item_equip_3(i, false, true) {
                true => {
                    let item = this.m_item_list().get_item(i);
                    match item.m_item().get_use_type().value {
                        1 => return true,
                        _ => continue,
                    }
                },
                false => continue,
            }
        }
    }

    false
}

#[unity::hook("App", "Unit", "SetOptimalWeaponForClassChange")] // 0x7101A3CDF0
pub fn unit_set_optimal_weapon_hook(
    this: Unit,
    weapon_mask: WeaponMask,
    is_bullet: bool,
    _method_info: OptionalMethod,
) -> UnitItem {
    let equipped = this.m_item_list().get_equipped();

    if equipped.is_null() || equipped.m_item().is_null() {
        let can_equip = this.item_equip();
        if can_equip {
            return this.m_item_list().get_equipped()
        } else {
            let job_weapons = this
                .m_job()
                .get_max_level_weapons(weapon_mask, this.m_original_aptitude());
            let mut weapon_list = HashSet::new();
            for i in job_weapons {
                if IWeaponMaskMethods::test_2(weapon_mask, i) {
                    weapon_list.insert(i.value);
                }
            }

            if this.m_job().is_gunner() {
                weapon_list.insert(ItemData_Kinds::special().value);
            }

            for i in weapon_list.iter() {
                let item = ItemData::create_simple_weapon(ItemData_Kinds { value: *i}, is_bullet);
                this.m_item_list().clear();
                this.m_item_list().add(item);
                match this.item_equip() {
                    true => return this.m_item_list().get_equipped(),
                    false => continue,
                }
            }
        }
    } else {
        if IWeaponMaskMethods::test_2(weapon_mask, equipped.m_item().get_kind()) {
            return equipped
        } else {
            if this.item_equip() {
                return this.m_item_list().get_equipped()
            } else {
                let job_weapons = this
                    .m_job()
                    .get_max_level_weapons(weapon_mask, this.m_original_aptitude());
                let mut weapon_list = HashSet::new();
                for i in job_weapons {
                    if IWeaponMaskMethods::test_2(weapon_mask, i) {
                        weapon_list.insert(i.value);
                    }
                }

                if this.m_job().is_gunner() {
                    weapon_list.insert(ItemData_Kinds::special().value);
                }

                for i in weapon_list.iter() {
                    let item = ItemData::create_simple_weapon(ItemData_Kinds {value: *i}, is_bullet);
                    this.m_item_list().clear();
                    this.m_item_list().add(item);
                    match this.item_equip() {
                        true => return this.m_item_list().get_equipped(),
                        false => continue,
                    }
                }
            }
        }
    }

    UnitItem::null()
}

#[unity::hook("App", "Unit", "ItemAddOnDlcEvil")] // 0x7101A3F520
pub fn unit_item_add_on_dlc_evil_hook(
    this: Unit, 
    iids: Array<Il2CppString>,
    chapter: ChapterData,
    level: i32,
    _method_info: OptionalMethod,
) {
    if !iids.is_null() {
        if (chapter.get_flag().value + 1).shr(6) & 1 == 0 {
            this.equipable_item_add(iids);
        } else {
            for name in iids {
                let mut new_item = ItemData::try_get(name);
                if !new_item.is_null() {
                    if new_item.get_high_rank_item().is_null() 
                        || new_item.get_high_rank_item().to_rust_string() == "" {
                            let new_item = ItemData::try_get(name);
                            if !new_item.is_null() && this.item_equip_4(new_item) {
                                let new_ui = UnitItem::instantiate().unwrap();
                                new_ui.ctor_2(new_item);
                            }
                        } else {
                            new_item = ItemData::try_get(new_item.get_high_rank_item());
                            if !new_item.is_null()
                                && this.can_item_equip_2(new_item, new_item.get_kind() == ItemData_Kinds::rod(), true) {
                                    let new_ui = UnitItem::instantiate().unwrap();
                                    new_ui.ctor_2(new_item);
                                }
                        }
                }
            }
        }
    }

    let mut count = 0;
    for i in iids {
        count += 1;
        match i.to_rust_string().as_str() {
            "ライブ" => break,
            "リライブ" => break,
            "リブロー" => break,
            "リカバー" => break,
            "リザーブ" => break,
            "レスト" => break,
            _ => continue,
        }
    }

    if !iids.is_null() && count == iids.len() && level < 21 {
        this.item_add("傷薬");
    } else if !iids.is_null() && count == iids.len() && level >= 21 {
        this.item_add("特効薬");
    }
}

#[unity::hook("App", "Unit", "EquipableItemAdd")] // 0x7101A408E0
pub fn unit_equipable_item_add(
    this: Unit,
    iids: Array<Il2CppString>,
    _method_info: OptionalMethod,
) -> bool {
    if iids.is_null() || iids.is_empty() {
        return false
    }

    let mut equipped_slots = 0;
    for name in iids {
        let new_item = ItemData::try_get(name);
        if !new_item.is_null() && this.can_item_equip_2(new_item, new_item.get_kind() == ItemData_Kinds::rod(), true) {
            let new_ui = UnitItem::instantiate().unwrap();
            new_ui.ctor_2(new_item);
            let index = this.m_item_list().add_2(new_ui);
            equipped_slots |= (!index).shr(31);
        }
    }

    equipped_slots != 0
}

#[skyline::hook(offset = 0x1A417F0)] // 0x7101A417F0, App.Unit$$HasEquipableItem
pub fn unit_has_equipable_item_hook(
    this: Unit,
    _method_info: OptionalMethod,
) -> bool {
    for i in 0..=7 {
        if this.can_item_equip_3(i, false, true) {
            return true
        }
    }

    false
}

#[skyline::hook(offset = 0x1A41E30)] // 0x7101A41E30, App.Unit$$HasEquipableItem
pub fn unit_has_equipable_item_range_hook(
    this: Unit,
    range: i32,
    _method_info: OptionalMethod,
) -> bool {
    for i in 0..=7 {
        if this.can_item_equip_3(i, false, true) {
            let item = this.m_item_list().get_item(i);
            let (_in_range, inner, outer)= this.calc_item_range(item.m_item(), SkillData::null());
            if range <= inner || range <= outer {
                return true
            }
        }
    }

    false
}

#[skyline::hook(offset = 0x1A42510)] // 0x7101A42510, App.Unit$$HasEquipableItem
pub fn unit_has_equipable_item_kind_hook(
    this: Unit,
    kind: ItemData_Kinds,
    _method_info: OptionalMethod
) -> bool {
    for i in 0..=7 {
        if this.can_item_equip_3(i, kind == ItemData_Kinds::rod(), true)
            && !this.m_item_list().get_item(i).is_null()
            && this.m_item_list().get_item(i).get_kind() == kind {
                return true
            }
    }

    false
}

#[skyline::hook(offset = 0x1A42B50)] // 0x7101A42B50, App.Unit$$CanItemEquip
pub fn unit_can_item_equip_uire_hook(
    this: Unit,
    unit_item: UnitItem,
    rod: bool,
    exp: bool,
    _method_info: OptionalMethod,
) -> bool {
    if unit_item.is_null() {
        false
    } else {
        this.can_item_equip_2(unit_item.m_item(), rod, exp)
    }
}

#[skyline::hook(offset = 0x1A43120)] // 0x7101A43120, App.Unit$$CanItemEquip
pub fn unit_can_item_equip_idrw_hook(
    this: Unit,
    item: ItemData,
    rod: bool,
    weapon_level: bool,
    _method_info: OptionalMethod,
) -> bool {
    if item.is_null() {
        return false
    }

    let item_kind = item.get_kind();

    if item_kind.value > 9                  // Non-equippable items
        || item_kind.value == 0             // None
        || (item_kind.value == 7 && !rod)   // Staves
        || !IWeaponMaskMethods::test_2(this.m_weapon_mask(), item_kind) {
            return false
        }

    let item_flag = item.get_flag();
    if item_flag.get_value() & 384 == 0 && weapon_level { // ItemData.Flags{value: 384} = IgnoreWeaponLevel + Engage
        let item_wlvl = item.m_weapon_level();
        let unit_wlvl = this.get_weapon_level(item_kind, true);
        if item_wlvl.value > unit_wlvl.value {
            return false
        }
    }

    let equip_cond = item.get_equip_condition();
    if !equip_cond.is_null() && equip_cond.to_rust_string() != "" {
        let equip_skill = SkillData::try_get(equip_cond);
        if equip_skill.is_null() {
            return false
        }

        let equip_index = equip_skill.index();
        let test_skill = this.get_mask_skill().m_mask().get_item(equip_index);
        if !test_skill {
            return false
        }
    }

    if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_male()) {
        loop {
            if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                break
            }

            let mut force_ty = this.get_force_type();
            if this.get_force().is_null() {
                force_ty = Force_Type::empty();
            }

            let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
            if test_unit.is_null()
                || test_unit.get_force().is_null()
                || test_unit.get_force_type().value > 2
                || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                    break
                }
        }

        let edit_unit = this.get_edit();
        let person_unit = this.get_person();
        let mut unit_gender = person_unit.get_gender();
        if edit_unit.is_enable() {
            unit_gender = edit_unit.get_gender();
        }

        if unit_gender.value == 2 {
            return false
        }
    }

    if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_female()) {
        loop {
            if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                break
            }

            let mut force_ty = this.get_force_type();
            if this.get_force().is_null() {
                force_ty = Force_Type::empty();
            }

            let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
            if test_unit.is_null()
                || test_unit.get_force().is_null()
                || test_unit.get_force_type().value > 2
                || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                    break
                }
        }

        let edit_unit = this.get_edit();
        let person_unit = this.get_person();
        let mut unit_gender = person_unit.get_gender();
        if edit_unit.is_enable() {
            unit_gender = edit_unit.get_gender();
        }

        if unit_gender.value != 2 {
            return false
        }
    }

    if IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::engaging()) {
        let mut god = this.get_god_unit();
        let mut god_data: GodData;
        let parent: Unit;

        if !this.m_god_link().is_null() {
            god = this.m_god_link();
            parent = god.get_parent();
        } else if !god.is_null() {
            parent = god.get_parent();
        } else {
            parent = Unit::null();
        }

        if parent.is_null() {
            god_data = god.get_data();
        } else {
            god_data = god.get_data();

            if IBitFieldTemplate64_1Methods::test(parent.get_status(), Unit_Status::engaging()) {
                god_data = god_data.get_main_data();
            }
        }

        if IBitFieldTemplate32_1Methods::test(god_data.get_flag(), GodData_Flags::only_engage_weapon())
            && !IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::engage()) {
                return false
            }
    }

    true
}

#[skyline::hook(offset = 0x1A436B0)] // 0x7101A436B0, App.Unit$$CanItemEquip
pub fn unit_can_item_equip_ire_hook(
    this: Unit,
    index: i32,
    rod: bool,
    exp: bool,
    _method_info: OptionalMethod,
) -> bool {
    let item = this.m_item_list().get_item(index);
    this.can_item_equip(item, rod, exp)
}

#[skyline::hook(offset = 0x1A43CE0)] // 0x7101A43CE0, App.Unit$$CanUseCannon
pub fn unit_can_use_cannon_xz_hook(
    this: Unit,
    x: i32,
    z: i32,
    _method_info: OptionalMethod,
) -> bool {
    let map_image = MapImage::get_instance();
    let map_terrain = map_image.m_terrain();
    let index = map_terrain.m_result().base().get_2(x, z) as i32;
    let terrain_data = TerrainData_2::try_get_2(index);
    
    this.can_use_cannon_2(terrain_data)
}

#[skyline::hook(offset = 0x1A44A40)] // 0x7101A44A40, App.Unit$$CanUseCannon
pub fn unit_can_use_cannon_terrain_hook(
    this: Unit,
    terrain: TerrainData_2,
    _method_info: OptionalMethod,
) -> bool {
    if terrain.is_bow_cannon() {
        for i in 0..=7 {
            let item = this.m_item_list().get_item(i);
            if !item.is_null() && item.get_kind() == ItemData_Kinds::bow() {
                return this.can_item_equip(item, false, true)
            }
        }
    } else if terrain.is_magic_cannon() {
        for i in 0..=7 {
            let item = this.m_item_list().get_item(i);
            if !item.is_null() && item.get_kind() == ItemData_Kinds::magic() {
                return this.can_item_equip(item, false, true)
            }
        }
    } else {
        return terrain.is_fire_cannon()
    }

    false
}

#[unity::hook("App", "Unit", "NextItemEquip")] // 0x7101A456A0
pub fn unit_next_item_equip_hook(
    this: Unit,
    reverse: bool,
    _method_info: OptionalMethod,
) -> bool {
    let index = this.m_item_list().get_index_equipped();
    if index == 0 {
        for i in 1..=7 {
            let next_index = match reverse {
                true => 8 - i,
                false => i,
            };

            if this.can_item_equip_3(next_index, false, true) {
                MapHistory::preequip_item(this);
                this.item_equip_2(next_index, true);
                MapHistory::postequip_item(this);

                if !reverse {
                    for _j in 1..=next_index {
                        this.m_item_list().r#move(1, 7);
                    }

                    this.m_item_list().close_up();
                }

                if this.m_actor().is_null() {
                    UnitActors::create_actor(this);
                }

                return true
            }
        }
    }

    false
}

#[unity::hook("App", "Unit", "GetEngageEquip")] // 0x7101A478C0
pub fn unit_get_engage_equip_hook(
    this: Unit,
    skill: SkillData,
    target: Unit,
    _method_info: OptionalMethod,
) -> UnitItem {
    if skill.is_null() {
        return UnitItem::null()
    }

    if (this.m_mask_skill().m_flags().value & 20i64) != 0i64 {
        return this.m_item_list().get_equipped()
    }

    let mut winning_index = -1;
    let mut winning_range = 0;
    let mut winning_power = 0;
    if this.get_skill_equip(skill, -1).is_null() {
        for i in 0..=7 {
            let item = this.m_item_list().get_item(i);
            if item.is_null() || item.m_item().is_null() {
                continue
            }
            let range = item.m_item().get_range_o();
            let mut power = item.m_item().get_power();
            if this.can_item_equip(item, item.get_kind() == ItemData_Kinds::rod(), true)
                && !IWeaponMaskMethods::test_2(skill.get_weapon_prohibit(), item.get_kind())
                && !target.is_null()
                && !target.m_person().is_null() {
                    let attrs = target.m_job().get_attrs().value | target.m_person().get_attrs().value;

                    let mut skills_data = this.m_mask_skill();
                    if item.m_god_unit().is_null() {
                        if UnitItem::s_enchant_hash() == 0 {
                            skills_data = item.m_item().get_equip_skills();
                        } else if item.m_item().m_enchant_hash() == UnitItem::s_enchant_hash() {
                            skills_data = item.m_item().get_enchant_skills(0);
                        }
                    } else {
                        let mut enchanted = false;
                        if UnitItem::s_enchant_hash() == 0 {
                            enchanted = false;
                        } else if item.m_item().m_enchant_hash() == UnitItem::s_enchant_hash() {
                            enchanted = true;
                        }
                        skills_data = item
                            .m_god_unit()
                            .try_get_god_weapon_refine_result_equip_skills(item.get_iid(), enchanted);
                    }
                    
                    if skills_data.m_efficacys().value
                        & attrs
                        & !target.m_mask_skill().m_efficacy_ignores().value != 0 {
                            power *= 3;
                        }
                    
                    if (winning_range <= range) && (winning_power <= power) {
                        winning_index = i;
                        winning_range = range;
                        winning_power = power;
                    }
                }
        }
    }

    if -1 < winning_index {
        return this.m_item_list().get_item(winning_index)
    }
    
    UnitItem::null()
}

#[skyline::hook(offset = 0x1A48280)] // 0x7101A48280, App.Unit$$GetAttackRange
pub fn unit_get_attack_range_hook(
    this: Unit,
    min_range: &mut i32,
    max_range: &mut i32,
    skill: SkillData,
    _method_info: OptionalMethod,
) -> bool {
    *min_range = 255;
    *max_range = 0;

    for i in 0..=7 {
        let item = this.m_item_list().get_item(i);
        if this.can_item_equip(item, item.get_kind() == ItemData_Kinds::rod(), true) {
            let (in_range, inner, outer) = this.calc_item_range(item.m_item(), skill);
            if in_range {
                *min_range = (*min_range).min(inner);
                *max_range = (*max_range).max(outer);
            }
        }
    }

    if !skill.is_null() && *min_range <= *max_range && skill.get_range_i() > 0 {
        *min_range = (*min_range).max(skill.get_range_i());
        *max_range = (*max_range).min(skill.get_range_o());
    }
    
    *min_range <= *max_range
}

#[skyline::hook(offset = 0x1A49260)] // 0x7101A49260, App.Unit$$GetAttackRange
pub fn unit_get_attack_range_item_hook(
    this: Unit,
    min_range: &mut i32,
    max_range: &mut i32,
    equip_item: UnitItem,
    skill: SkillData,
    _method_info: OptionalMethod,
) -> bool {
    *min_range = 255;
    *max_range = 0;

    if equip_item.is_null() || equip_item.m_item().is_null() {
        let (result, inner, outer) = this.get_attack_range(skill);
        *min_range = inner;
        *max_range = outer;
        return result
    } else if equip_item.m_item().get_kind() == ItemData_Kinds::rod() {
        return false
    }

    let (_in_range, inner, outer) = this.calc_item_range(equip_item.m_item(), skill);
    *min_range = (*min_range).min(inner);
    *max_range = (*max_range).max(outer);

    if !skill.is_null() && *min_range <= *max_range && skill.get_range_i() > 0 {
        *min_range = (*min_range).max(skill.get_range_i());
        *max_range = (*max_range).min(skill.get_range_o());
    }
    
    *min_range <= *max_range
}

#[skyline::hook(offset = 0x1A48A70)] // 0x7101A48A70, App.Unit$$GetRodRange
pub fn unit_get_rod_range_hook(
    this: Unit,
    min_range: &mut i32,
    max_range: &mut i32,
    skill: SkillData,
    _method_info: OptionalMethod,
) -> bool {
    *min_range = 255;
    *max_range = 0;

    for i in 0..=7 {
        let unit_item = this.m_item_list().get_item(i);
    
        if unit_item.is_null() {
            continue
        }

        let item = unit_item.m_item();
        if item.is_null() {
            continue
        }

        let item_kind = item.get_kind();

        if item_kind.value > 9          // Non-equippable items
            || item_kind.value == 0     // None
            || !IWeaponMaskMethods::test_2(this.m_weapon_mask(), item_kind) {
                continue
            }

        let item_flag = item.get_flag();
        if item_flag.get_value() & 384 == 0 { // ItemData.Flags{value: 384} = IgnoreWeaponLevel + Engage
            let item_wlvl = item.m_weapon_level();
            let unit_wlvl = this.get_weapon_level(item_kind, true);
            if item_wlvl.value > unit_wlvl.value {
                continue
            }
        }

        let equip_cond = item.get_equip_condition();
        if !equip_cond.is_null() && equip_cond.to_rust_string() != "" {
            let equip_skill = SkillData::try_get(equip_cond);
            if equip_skill.is_null() {
                continue
            }

            let equip_index = equip_skill.index();
            let test_skill = this.get_mask_skill().m_mask().get_item(equip_index);
            if !test_skill {
                continue
            }
        }

        if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_male()) {
            loop {
                if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                    break
                }

                let mut force_ty = this.get_force_type();
                if this.get_force().is_null() {
                    force_ty = Force_Type::empty();
                }

                let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
                if test_unit.is_null()
                    || test_unit.get_force().is_null()
                    || test_unit.get_force_type().value > 2
                    || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                        break
                    }
            }

            let edit_unit = this.get_edit();
            let person_unit = this.get_person();
            let mut unit_gender = person_unit.get_gender();
            if edit_unit.is_enable() {
                unit_gender = edit_unit.get_gender();
            }

            if unit_gender.value == 2 {
                continue
            }
        }

        if IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::only_female()) {
            loop {
                if !IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::vision()) {
                    break
                }

                let mut force_ty = this.get_force_type();
                if this.get_force().is_null() {
                    force_ty = Force_Type::empty();
                }

                let test_unit = UnitPool::get_from_ident(force_ty, this.m_owner_unit());
                if test_unit.is_null()
                    || test_unit.get_force().is_null()
                    || test_unit.get_force_type().value > 2
                    || (test_unit.get_mask_skill().get_flags().value + 7).shr(3) & 1i64 == 0i64  {
                        break
                    }
            }

            let edit_unit = this.get_edit();
            let person_unit = this.get_person();
            let mut unit_gender = person_unit.get_gender();
            if edit_unit.is_enable() {
                unit_gender = edit_unit.get_gender();
            }

            if unit_gender.value != 2 {
                continue
            }
        }

        if IBitFieldTemplate64_1Methods::test(this.get_status(), Unit_Status::engaging()) {
            let mut god = this.get_god_unit();
            let mut god_data: GodData;
            let parent: Unit;

            if !this.m_god_link().is_null() {
                god = this.m_god_link();
                parent = god.get_parent();
            } else if !god.is_null() {
                parent = god.get_parent();
            } else {
                parent = Unit::null();
            }

            if parent.is_null() {
                god_data = god.get_data();
            } else {
                god_data = god.get_data();

                if IBitFieldTemplate64_1Methods::test(parent.get_status(), Unit_Status::engaging()) {
                    god_data = god_data.get_main_data();
                }
            }

            if IBitFieldTemplate32_1Methods::test(god_data.get_flag(), GodData_Flags::only_engage_weapon())
                && !IBitFieldTemplate32_1Methods::test(item_flag, ItemData_Flags::engage()) {
                    continue
                }
        }

        if item.get_kind() == ItemData_Kinds::rod() {
            if IWeaponMaskMethods::test_2(skill.get_weapon_prohibit(), ItemData_Kinds::rod()) {
                continue
            }

            let (in_range, inner, outer) = this.calc_item_range(item, skill);
            if in_range {
                *min_range = (*min_range).min(inner);
                *max_range = (*max_range).max(outer);
            }
        }
    }

    if !skill.is_null() && *min_range <= *max_range && skill.get_range_i() > 0 {
        *min_range = (*min_range).max(skill.get_range_i());
        *max_range = (*max_range).min(skill.get_range_o());
    }

    *min_range <= *max_range
}

#[skyline::hook(offset = 0x1A49B90)] // 0x7101A49B90, App.Unit$$GetRodRange
pub fn unit_get_rod_range_item_hook(
    this: Unit,
    min_range: &mut i32,
    max_range: &mut i32,
    equip_item: UnitItem,
    skill: SkillData,
    _method_info: OptionalMethod,
) -> bool {
    *min_range = 255;
    *max_range = 0;

    if equip_item.is_null() || equip_item.m_item().is_null() {
        let (result, inner, outer) = this.get_rod_range(skill);
        *min_range = inner;
        *max_range = outer;
        return result
    }

    if equip_item.m_item().get_kind() != ItemData_Kinds::rod() {
        return false
    }

    let (_in_range, inner, outer) = this.calc_item_range(equip_item.m_item(), skill);
    *min_range = inner;
    *max_range = outer;

    if !skill.is_null() && *min_range <= *max_range && skill.get_range_i() > 0 {
        *min_range = (*min_range).max(skill.get_range_i());
        *max_range = (*max_range).min(skill.get_range_o());
    }

    *min_range <= *max_range
}

#[unity::hook("App", "Unit", "GetRevengeWeapon")] // 0x7101A4A4C0
pub fn unit_get_revenge_weapon_hook(
    this: Unit,
    target: Unit,
    target_item: UnitItem,
    range: i32,
    _method_info: OptionalMethod,
) -> UnitItem {
    let mut equipped_item = this.m_item_list().get_equipped();
    if target.m_mask_skill().m_flags().value & SkillData_Flags::revenge_auto_equip().value != 0 {
        let mut distance = range;
        if range < 0 {
            distance = Map::get_range_2(this, target);
        }
        if !target_item.is_null() && target_item.m_item().m_is_weapon() {
            let mut winning_score = -1;
            let mut winning_index = -1;
            for i in 0..=7 {
                if this.can_item_equip_3(i, false, true) {
                    let current_item = this.m_item_list().get_item(i);
                    if current_item.is_null() || !current_item.is_weapon() {
                        continue
                    }

                    let (_in_range, inner, outer) = 
                        this.calc_item_range(current_item.m_item(), SkillData::null());
                    if inner <= outer {
                        let mut range_test = false;
                        if distance <= outer {
                            range_test = inner - distance < 0;
                        }
                        let mut flags = 0;
                        if distance <= outer && inner == distance ||
                            range_test != (distance <= outer && inner.overflowing_sub(distance).1) {
                                flags = 0x1000000;
                            }
                        
                        if target_item.is_null() {
                            flags |= 0x40000;
                        } else {
                            let interact = InteractData::get_interact(current_item, target_item);

                            match interact.value {
                                0 => flags |= 0x40000,  // Neutral
                                1 => flags |= 0x100000, // Advantage
                                _ => (),                // Disadvantage
                            }
                        }

                        if current_item.is_efficacy(target) {
                            flags |= 0x10000;
                        }

                        let power = current_item.get_power();
                        let hit = current_item.get_hit();
                        let crit = current_item.get_critical();
                        let avoid = current_item.get_avoid();
                        let dodge = current_item.get_secure();

                        let new_score = flags + power + hit + crit + avoid + dodge;

                        if new_score > winning_score {
                            winning_score = new_score;
                            winning_index = i;
                        }
                    }
                }
            }

            if winning_index > -1 {
                equipped_item = this.m_item_list().get_item(winning_index);
            }
        }
    }

    equipped_item
}

#[skyline::hook(offset = 0x1A4BA20)] // 0x7101A4BA20, Unit$$CanItemUse
pub fn unit_can_item_use_hook(
    this: Unit,
    item: ItemData,
    _method_info: OptionalMethod,
) -> bool {
    if item.is_null() || !IBitFieldTemplate32_1Methods::test(item.get_flag(), ItemData_Flags::can_use()) {
        return false
    }
    
    if !item.is_weapon() {
        if item.get_kind() != ItemData_Kinds::tool() || item.get_use_type().value > 24 {
            return MapItemHelper::can_use_target(this, item, this)
        }

        let job_list = ClassChange::get_job_list(this, item);
        return 0 < job_list.size()
    }
        
    this.can_item_equip_2(item, item.get_kind() == ItemData_Kinds::rod(), true)
}

#[skyline::hook(offset = 0x1A4C060)] // 0x7101A4C060, Unit$$CanItemUse
pub fn unit_can_item_use_target_hook(
    this: Unit,
    item: ItemData,
    target_unit: Unit,
    _method_info: OptionalMethod,
) -> bool {
    if item.is_null() || !IBitFieldTemplate32_1Methods::test(item.get_flag(), ItemData_Flags::can_use()) {
        return false
    }
    
    if !item.is_weapon() {
        if item.get_kind() != ItemData_Kinds::tool() || item.get_use_type().value > 24 {
            return MapItemHelper::can_use_target(this, item, target_unit)
        }

        let job_list = ClassChange::get_job_list(target_unit, item);
        return 0 < job_list.size()
    }
        
    target_unit.can_item_equip_2(item, item.get_kind() == ItemData_Kinds::rod(), true)
}

#[unity::hook("App", "Unit", "IsDrawActiveColor")] // 0x7101A4DA20
pub fn unit_is_draw_active_color(
    this: Unit,
    unit_item: UnitItem,
    _method_info: OptionalMethod,
) -> bool {
    if unit_item.is_null() || unit_item.m_item().is_null() {
        return false
    }

    if !unit_item.m_item().m_is_weapon() {
        return unit_item.m_item().get_kind() != ItemData_Kinds::precious()
    }
    
    this.can_item_equip(unit_item, unit_item.get_kind() == ItemData_Kinds::rod(), true)
}