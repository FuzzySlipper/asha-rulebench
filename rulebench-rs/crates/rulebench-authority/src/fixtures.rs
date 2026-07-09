use crate::model::{RulebenchReceipt, RulebenchScenario};
use crate::scenarios::hexing_bolt::fixture;

pub fn hexing_bolt_fixture_scenario() -> RulebenchScenario {
    fixture::hexing_bolt_fixture_scenario()
}

pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    fixture::accepted_hexing_bolt_fixture_receipt()
}

pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    fixture::rejected_target_fixture_receipt()
}
