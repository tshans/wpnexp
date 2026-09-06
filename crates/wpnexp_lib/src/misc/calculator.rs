
use engage::app::CalculatorManager;
use unity::OptionalMethod;

#[unity::hook("App", "UnitCalculator", "AddCommand")]
fn add_command_hook(calculator: CalculatorManager, method_info: OptionalMethod) {
    call_original!(calculator, method_info);
    crate::game::wexp::register_weapon_level_calculator_commands(calculator);
    crate::game::wexp::register_wexp_calculator_commands(calculator);
}