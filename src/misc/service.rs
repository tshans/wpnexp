
use std::sync::RwLock;


#[repr(C)]
pub enum WexpEvent {
    Initialize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ValueArgs {
    value: i32
}

#[cobapi::service]
pub trait WexpService {
    fn subscribe_initialize(&self, handler: extern "C" fn());
    fn unsubscribe_initialize(&self, handler: extern "C" fn());
}


struct Slot<F: 'static> {
    handlers: RwLock<Vec<F>>,
}

impl<F: 'static> Slot<F> {
    const fn new() -> Self {
        Self { handlers: RwLock::new(Vec::new()) }
    }
}

impl<F: PartialEq + Copy + 'static> Slot<F> {
    fn subscribe(&self, h: F) {
        self.handlers.write().unwrap().push(h);
    }

    fn unsubscribe(&self, h: F) {
        self.handlers.write().unwrap().retain(|x| *x != h);
    }
}

impl Slot<extern "C" fn()> {
    fn fire(&self) {
        // Copy them to avoid a deadlock
        let snapshot = self.handlers.read().unwrap().clone();

        for h in snapshot {
            h();
        }
    }
}

// impl<A> Slot<extern "C" fn(&A)> {
//     fn fire(&self, args: &A) {
//         // Copy them to avoid a deadlock
//         let snapshot = self.handlers.read().unwrap().clone();

//         for h in snapshot {
//             h(args);
//         }
//     }
// }

struct WexpBridge {
    initialize: Slot<extern "C" fn()>,
}

impl WexpBridge {
    const fn new() -> Self {
        Self {
            initialize: Slot::new(),
        }
    }
}

impl WexpService for &'static WexpBridge {
    fn subscribe_initialize(&self, handler: extern "C" fn()) {
        self.initialize.subscribe(handler);
    }
    fn unsubscribe_initialize(&self, handler: extern "C" fn()) {
        self.initialize.unsubscribe(handler);
    }
}


static WEXP_BRIDGE: WexpBridge = WexpBridge::new();

pub fn install_wexp_services() {
    let _ = install(&WEXP_BRIDGE);
}

pub fn publish_wexp_event(event: WexpEvent) {
    // Call the callbacks for the service
    match &event {
        WexpEvent::Initialize => {
            WEXP_BRIDGE.initialize.fire();
        },
    }
}

// Services I would like to provide to other mods:
//  - Get base item wexp values via data::get_item_wexp (or let them replace the function)
//  - Get base unit wexp values via data::get_unit_wexp (or let them replace the function)
//  - Get base god wexp values via data::get_god_wexp (or let them replace the function)
//  - Get the total earned wexp value via data::calculate_unit_earned_wexp and data::calculate_side_earned_wexp
//    (or let them replace the function?)
//  - Get the current wexp of a BattleInfoSide/Unit given an ItemData.Kind value (with or without the Class offset)
//    via data::calculate_side_wexp_kind, data::calculate_unit_wexp_kind, or data::calculate_unit_wexp_kind_job
//    (or let them replace the function?)

//  - Expose the weaponlevel.rs functions?

//  - Expose the BATTLE_WEXP associated functions for use with custom items/scripted events?
//  - I probably won't expose the Static/Dynamic/Ledger functions...