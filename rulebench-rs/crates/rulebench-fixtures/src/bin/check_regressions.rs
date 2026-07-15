use rulebench_fixtures::{
    run_capability_conformance, run_scenario_regressions, scenario_package_registry,
    CapabilityConformanceFilter, ScenarioRegressionFilter,
};

fn main() {
    let (filter, capability_id, list_only) =
        parse_arguments().unwrap_or_else(|message| fail(&message));
    let registry = scenario_package_registry();
    let report = run_scenario_regressions(&registry, &filter);
    for case in &report.cases {
        println!(
            "{}@{} {}@{} {} outcome={} events={} trace={} fingerprint={}",
            case.package_id,
            case.package_version,
            case.ruleset_id,
            case.ruleset_version,
            case.scenario_id,
            case.outcome_class
                .map(|outcome| outcome.code())
                .unwrap_or("unclassified"),
            case.event_count,
            case.trace_count,
            case.final_state_fingerprint,
        );
    }
    let conformance = run_capability_conformance(
        &registry,
        &CapabilityConformanceFilter {
            capability_id,
            package_id: filter.package_id.clone(),
            ruleset_id: filter.ruleset_id.clone(),
            scenario_id: filter.scenario_id.clone(),
        },
    );
    for case in &conformance.cases {
        println!(
            "conformance {} capabilities={} replay={} mismatch={} rejections={} fingerprint={}",
            case.case_id,
            case.capabilities
                .iter()
                .map(|capability| format!("{}@{}", capability.id, capability.version))
                .collect::<Vec<_>>()
                .join(","),
            case.replay_verified,
            case.replay_mismatch_classified,
            case.rejection_codes.join(","),
            case.final_state_fingerprint,
        );
    }
    if list_only {
        return;
    }
    if let Some(difference) = report.first_difference {
        fail(&format!(
            "regression mismatch at {}: expected {}; actual {}",
            difference.path, difference.expected, difference.actual,
        ));
    }
    if let Some(failure) = conformance.failures.first() {
        fail(&format!(
            "capability conformance {} for {}@{} in {}: {}",
            failure.kind.code(),
            failure.capability.id,
            failure.capability.version,
            failure.case_id.as_deref().unwrap_or("registry"),
            failure.detail,
        ));
    }
    if !conformance.accepted {
        fail("capability conformance selected no executable cases");
    }
    println!(
        "scenario regression and capability conformance check ok ({} scenarios; {} conformance cases; {} capabilities)",
        report.cases.len(),
        conformance.cases.len(),
        conformance.covered_capabilities.len(),
    );
}

fn parse_arguments() -> Result<(ScenarioRegressionFilter, Option<String>, bool), String> {
    let mut filter = ScenarioRegressionFilter::default();
    let mut capability_id = None;
    let mut list_only = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--list" => list_only = true,
            "--package" => filter.package_id = Some(next_value(&mut arguments, "--package")?),
            "--package-version" => {
                filter.package_version = Some(next_value(&mut arguments, "--package-version")?)
            }
            "--ruleset" => filter.ruleset_id = Some(next_value(&mut arguments, "--ruleset")?),
            "--ruleset-version" => {
                filter.ruleset_version = Some(next_value(&mut arguments, "--ruleset-version")?)
            }
            "--scenario" => filter.scenario_id = Some(next_value(&mut arguments, "--scenario")?),
            "--capability" => capability_id = Some(next_value(&mut arguments, "--capability")?),
            _ => return Err(format!("unknown regression argument: {argument}")),
        }
    }
    Ok((filter, capability_id, list_only))
}

fn next_value(arguments: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}
