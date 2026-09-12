extern crate wpnexp_lib as wpnexp_lib;

// I like the current WEXP setup:
//     Total WEXP = Static + Dynamic + Job
//     Earned WEXP = Weapon + Unit's Innate Proficiency + Emblem Favored Weapon

// Add WEXP_DATA values to the Fer/Grt line of WdwItemHelp/HelpParamSetter? Shouldn't be too hard...
// though this will have to wait till the plugin is merged with crtdmg. Maybe I can use a cobapi Service?

// BattleGrow -> GainExp (C) + End
//  - UnitGrowSequence -> Prepare (C) + GainExp (C) + Label (0) + CheckLevelUp (C) + LevelUp (C)
//                          + Label (1) + Label (2) + CheckClassChange (C) + ClassChange (C)
//                          + SetWeapon (C) + Label (C) + Label (5) + End
//      - ExpSequence -> WaitLoad (T) + Open (C) + Yield + WaitAnime (T) + SoundStart (C)
//                          + Tick (T) + SoundStop (C) + Yield + WaitAnime (T) + Release (C) + End
//      - LevelUpSequence -> Prepare (C) + Effect (C) + IsLoadingRes (W) + Open (C) + Label (0)
//                              + Yield + WaitTime + CheckParamChange (C) + CalcTalkMid (C) + Reflect (C)
//                              + Talk (C) + KeyWait (T) + Yield + LearnJobSkill (C) + WaitAnime (C)
//                              + Release (C) + ReloadActor (C) + WaitReloadActor (T) + End


// I've successfully shifted the WdwExpRoot prefab changes from a function hook to an Event Listener...
// What else can be moved over? (Obviously, I'll want to repeat this process for the prefab changes in crtdmg.)

// MainSequence.Label => ChapterSave/SaveDataLoad 15/18
// MainMenuSequence.Label => SaveDataCopy/Delete/ToStartGame/ToContinueGame 20/21/28/29

// RewindSequence.Label => ExecuteRewind 2


// Investigate these to replace the ACall hooks?
// SaveDataMenuSequence.Label => 
//          LoadMenu/SaveMenuFromMenu/SaveMenuFromPeriod/SaveMenuFromEnding/SuspendMenu/CopyMenu/DeleteMenu


// Finish up armsscroll.rs
//  - It works!


#[skyline::main(name = "wpnexp")]
pub fn main() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };
        let err_msg = format!(
            "wpnexp has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        skyline::error::show_error(
            420,
            "wpnexp has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));

    let config = wpnexp_lib::read_config().unwrap();
    if !config.enabled {
        println!("The wpnexp plugin has been disabled by another plugin. v{}", wpnexp_lib::VERSION);
    } else {
        wpnexp_lib::misc_init(config.overwrite, config.index, config.unit_aptitude, config.god_aptitude);
        wpnexp_lib::game_init();
        wpnexp_lib::ui_init();
    }
}