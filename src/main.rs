use crate::loggers::*;
use crate::structs::*;
use crate::traits::HookData;
use crate::validators::*;
use std::process::exit;
use GitPolicyEnforcer::*;

fn main() {
    // Create the logging directory.
    let _ = create_logging_directory();

    // Arguments stuff
    let regex_argument = "regex";
    let regex_argument_value = "regex-value";
    let hooks_argument = "hook";
    let rules_argument = "rules";
    let matches = clap::Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("A Rust CLI tool that helps you enforce Git policies through Git hooks both server and client side.")
        .arg(
            clap::Arg::new(regex_argument)
                .long(regex_argument)
                .requires(regex_argument_value)
                .takes_value(true)
                .help("A regex (for testing)")
        )
        .arg(
            clap::Arg::new(regex_argument_value)
                .long(regex_argument_value)
                .requires(regex_argument)
                .takes_value(true)
                .help("A regex value (for testing)")
        )
        .arg(
            clap::Arg::new(rules_argument)
                .long(rules_argument)
                .help("The json file that contains the rules")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new(hooks_argument)
                .long(hooks_argument)
                .help("The first ($0) argument of the executing hook script")
                .takes_value(true),
        )
        .get_matches();

    // Functionality of on the fly validation.
    let regex_str = matches.value_of(regex_argument);
    if let Some(regex_str) = regex_str {
        let value_to_test = vec![matches.value_of(regex_argument_value).unwrap().to_owned()];
        let regex = create_regex(regex_str);
        if let Err(e) = regex {
            println!("\nValidation failed at regex compilation: {}\n", e);
            exit(0);
        }
        let regex = regex.unwrap();
        let validation_result = validate_title_format(&value_to_test, &regex);
        if validation_result.is_err() {
            println!(
                "\nValidation failed: Value \"{}\" failed to validate against regex \"{}\"\n",
                &value_to_test[0], &regex
            );
        } else {
            println!("Validation succeeded");
        }
        exit(0);
    }
    // Functionality of on the fly validation end.

    // Start executing the actual program.
    let hooks_argument_value = matches.value_of(hooks_argument).unwrap_or("");
    let hook = get_hook(hooks_argument_value);
    let path = get_repo_path(hooks_argument_value);
    let git_repo_directory = std::path::Path::new(&path);
    match std::env::set_current_dir(&git_repo_directory) {
        Ok(_) => {}
        Err(_) => {
            let _ = log_to_file("set_current_dir failed");
        }
    }

    // Start executing based on the hook.
    match hook {
        Hook::Invalid => {
            let _ = log_to_file("Invalid/unsupported hook");
            std::process::exit(1);
        }
        Hook::Update => {
            let update_hook_data = UpdateHookData::get_data(get_stdin_data().as_str());
            let rules = match matches.value_of("rules") {
                Some(value) => match parse_rules(value) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = log_to_file(&format!("!parse_rules: {}", e));
                        exit(1)
                    }
                },
                None => {
                    let _ = log_to_file("No rules argument was provided");
                    exit(1);
                }
            };
            match validate_update_rules(&rules.update, &update_hook_data) {
                Ok(()) => {}
                Err(e) => log_to_ui(&e.to_string()),
            }
        }
    }
}
