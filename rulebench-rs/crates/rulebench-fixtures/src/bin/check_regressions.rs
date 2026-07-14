use rulebench_fixtures::{
    run_scenario_regressions, scenario_package_registry, ScenarioRegressionFilter,
};

fn main() {
    let (filter, list_only) = parse_arguments().unwrap_or_else(|message| fail(&message));
    let report = run_scenario_regressions(&scenario_package_registry(), &filter);
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
    if list_only {
        return;
    }
    if let Some(difference) = report.first_difference {
        fail(&format!(
            "regression mismatch at {}: expected {}; actual {}",
            difference.path, difference.expected, difference.actual,
        ));
    }
    println!(
        "scenario regression check ok ({} cases)",
        report.cases.len()
    );
}

fn parse_arguments() -> Result<(ScenarioRegressionFilter, bool), String> {
    let mut filter = ScenarioRegressionFilter::default();
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
            _ => return Err(format!("unknown regression argument: {argument}")),
        }
    }
    Ok((filter, list_only))
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
