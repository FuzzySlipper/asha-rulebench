use crate::model::{RulebenchReceipt, RulebenchScenario};

pub fn hexing_bolt_fixture_scenario() -> RulebenchScenario {
    rulebench_fixtures::hexing_bolt_fixture_scenario()
}

pub fn turn_control_fixture_scenario() -> RulebenchScenario {
    rulebench_fixtures::turn_control_fixture_scenario()
}

pub fn hexing_bolt_scenario_package() -> rulebench_fixtures::ScenarioPackage {
    rulebench_fixtures::hexing_bolt_scenario_package()
}

pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    rulebench_fixtures::accepted_hexing_bolt_fixture_receipt()
}

pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    rulebench_fixtures::rejected_target_fixture_receipt()
}
