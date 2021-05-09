use crate::loggers::*;
use crate::structs::*;
use crate::traits::HookData;
use crate::validators::*;
use std::process::exit;
use GitPolicyEnforcer::*;

fn main() {
    let hooks_argument = "hook";
    let rules_argument = "rules";
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("A Rust CLI tool that helps you enforce Git policies through Git hooks both server and client side.")
        .arg(
            clap::Arg::with_name(rules_argument)
                .long(rules_argument)
                .help("The json file that contains the rules")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(hooks_argument)
                .long(hooks_argument)
                .help("The first ($0) argument of the executing hook script")
                .takes_value(true),
        )
        .get_matches();

    let hooks_argument_value = matches.value_of(hooks_argument).unwrap_or("");
    let hook = get_hook(hooks_argument_value);
    let path = get_repo_path(hooks_argument_value);
    // Change working directory.
    let git_repo_directory = std::path::Path::new(&path);
    std::env::set_current_dir(&git_repo_directory);

    // Create the logging directory.
    match create_logging_directory() {
        Ok(()) => {}
        Err(_e) => {} /* Todo: How to handle this? */
    }

    // Start executing based on the hook.
    match hook {
        Hook::Invalid => {
            log_to_file("Invalid/unsupported hook");
            std::process::exit(1);
        }
        Hook::Update => {
            let update_hook_data = UpdateHookData::get_data(get_stdin_data().as_str());
            let rules = match matches.value_of("rules") {
                Some(value) => match parse_rules(value) {
                    Ok(v) => v,
                    Err(e) => {
                        log_to_file(&format!("!parse_rules: {}", e.to_string()));
                        exit(1)
                    }
                },
                None => {
                    log_to_file("No rules argument was provided");
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
