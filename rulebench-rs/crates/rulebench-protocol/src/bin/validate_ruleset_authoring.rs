#![forbid(unsafe_code)]

use rulebench_protocol::{
    validate_ruleset_definition, RuleModuleConfigurationDto, RuleModuleDeclarationDto,
    RulesetDefinitionDto,
};

fn main() {
    let definition = match parse_definition(std::env::args().skip(1).collect()) {
        Ok(definition) => definition,
        Err(message) => {
            println!("error:invalidAuthoringInput:{message}");
            std::process::exit(1);
        }
    };

    match validate_ruleset_definition(&definition) {
        Ok(_) => println!("accepted"),
        Err(error) => {
            println!("error:{}", error.code());
            std::process::exit(1);
        }
    }
}

fn parse_definition(arguments: Vec<String>) -> Result<RulesetDefinitionDto, String> {
    let [id, name, version, summary, modules @ ..] = arguments.as_slice() else {
        return Err("expected id name version summary and module declarations".to_string());
    };
    let modules = modules
        .iter()
        .map(|module| parse_module(module))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RulesetDefinitionDto {
        id: id.clone(),
        name: name.clone(),
        version: version.clone(),
        summary: summary.clone(),
        modules,
    })
}

fn parse_module(value: &str) -> Result<RuleModuleDeclarationDto, String> {
    let mut parts = value.splitn(3, ':');
    let module = parts.next().ok_or_else(|| "missing module".to_string())?;
    let version = parts
        .next()
        .ok_or_else(|| "missing module version".to_string())?;
    let configuration = parts
        .next()
        .ok_or_else(|| "missing module configuration".to_string())?;
    let configuration = match module {
        "actionResolution" => RuleModuleConfigurationDto::ActionResolution {
            targeting_policy: configuration.to_string(),
        },
        "turnControl" => RuleModuleConfigurationDto::TurnControl {
            turn_order_policy: configuration.to_string(),
        },
        _ => RuleModuleConfigurationDto::ActionResolution {
            targeting_policy: configuration.to_string(),
        },
    };
    Ok(RuleModuleDeclarationDto {
        module: module.to_string(),
        version: version.to_string(),
        configuration,
    })
}
