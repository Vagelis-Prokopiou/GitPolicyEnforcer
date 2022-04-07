use crate::loggers::log_to_file;
use crate::structs::{UpdateHookData, UpdateRules, ValidationError};
use regex::Regex;
use std::process::{exit, Command};

// Public functions
pub fn validate_update_rules(
    hook_rules: &UpdateRules,
    hook_data: &UpdateHookData,
) -> Result<(), ValidationError> {
    if hook_rules.branches.is_some() {
        // Do not run any validation if the current branch is not in the list of provided branches.
        if !hook_rules
            .branches
            .as_ref()
            .unwrap()
            .contains(&hook_data.branch)
        {
            return Ok(());
        }
    }

    let commits_range: Vec<String> =
        _get_commits_range(&hook_data.old_commit, &hook_data.new_commit);
    let commits: Vec<String> = _get_commits(&commits_range);
    let commit_titles: Vec<String> = _get_commit_titles(&commits);
    let commit_bodies = _get_commit_bodies(&commits);

    let title_regex_validator = create_regex(&hook_rules.title_format)?;

    // Title related validations.
    validate_title_format(&commit_titles, &title_regex_validator)?;
    _validate_title_max_length(&commit_titles, hook_rules.title_max_length)?;

    if hook_rules.body_required.is_some() {
        _validate_body_required(&commit_bodies)?;
    };

    if hook_rules.body_max_line_length.is_some() {
        _validate_body_max_line_length(&commit_bodies, hook_rules.body_max_line_length.unwrap())?;
    }

    // Todo: Pending.
    // if let Some(true) = hook_rules.enforce_squash_merge {
    //     _validator_enforce_squash_merge(&commits_range)?;
    // }

    Ok(())
}

pub fn create_regex(regex_str: &str) -> Result<Regex, crate::ValidationError> {
    match regex::Regex::new(regex_str) {
        Ok(r) => Ok(r),
        Err(_) => Err(crate::ValidationError::RegexCompilation(
            regex_str.to_owned(),
        )),
    }
}

// Private functions.
pub fn validate_title_format(
    commit_titles: &[String],
    regex_validator: &regex::Regex,
) -> Result<(), ValidationError> {
    for commit_title in commit_titles {
        if !regex_validator.is_match(commit_title) {
            return Err(ValidationError::TitleFormat(format!(
                "{:?}",
                regex_validator
            )));
        }
    }

    Ok(())
}

fn _validate_body_required(commit_bodies: &[Vec<String>]) -> Result<(), ValidationError> {
    for commit_body in commit_bodies {
        if commit_body.is_empty() {
            return Err(ValidationError::BodyRequired);
        }
    }
    Ok(())
}

fn _get_commits(commits_range: &[String]) -> Vec<String> {
    commits_range
        .iter()
        .map(|commit_hash| _get_commit(commit_hash))
        .collect()
}

/// Extracts the full commit, from a commit hash.
fn _get_commit(commit_hash: &str) -> String {
    let output = match Command::new("git")
        .arg("cat-file")
        .arg("commit")
        .arg(commit_hash)
        .output()
    {
        Ok(v) => v,
        Err(_e) => {
            let _ =
                log_to_file("_get_commit(): Failed to execute git cat-file commit <commit_hash>.");
            exit(1);
        }
    };

    match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(_e) => {
            let _ =
                log_to_file("_get_commit(): Failed to get utf8 string from git cat-file output");
            exit(1);
        }
    }
}

fn _get_commit_bodies(commits: &[String]) -> Vec<Vec<String>> {
    commits
        .iter()
        .map(|commit_hash| _get_commit_body(commit_hash))
        .collect()
}

/// Extracts the commit body from a full commit.
fn _get_commit_body(commit: &str) -> Vec<String> {
    let mut body_lines = vec![];
    let mut first_empty_line_found = false;
    let mut second_empty_line_found = false;

    // Start saving the commit lines that come after the second empty line.
    // The second empty line comes after the commit title and before the commit body.
    for line in commit.lines() {
        let line = line.trim();

        if line.is_empty() && !first_empty_line_found {
            first_empty_line_found = true;
            continue;
        }

        if line.is_empty() && first_empty_line_found {
            second_empty_line_found = true;
            continue;
        }

        if !line.is_empty() && second_empty_line_found {
            body_lines.push(line.to_owned());
            continue;
        }
    }

    body_lines
}

fn _get_commit_titles(commits: &[String]) -> Vec<String> {
    commits
        .iter()
        .map(|commit| _get_commit_title(commit))
        .collect()
}

/// Extracts the commit title from a full commit message.
fn _get_commit_title(commit: &str) -> String {
    let mut title = "";
    let mut found_first_empty_line = false;

    for line in commit.lines() {
        let line = line.trim();
        if line.is_empty() && !found_first_empty_line {
            found_first_empty_line = true;
        }
        if !line.is_empty() && found_first_empty_line {
            title = line;
            break;
        }
    }

    title.to_owned()
}

fn _validate_title_max_length(
    commit_titles: &[String],
    max_title_length: u8,
) -> Result<(), ValidationError> {
    for title in commit_titles.iter() {
        let number_of_characters = title.chars().count();
        if number_of_characters > max_title_length as usize {
            return Err(ValidationError::TitleMaxLength(max_title_length));
        }
    }
    Ok(())
}

fn _get_commits_range(old_commit: &str, new_commit: &str) -> Vec<String> {
    // Todo: This implementation does not correctly get all the commits.
    // Todo: Check the correct way to get all the commits.
    // Todo: Get all commits from current branch and remove the ones that exists in the target branch
    // git rev-list HEAD (and remove the ones that exists in the target branch)
    // git rev-list target_branch..HEAD (git rev-list master..HEAD)
    let commit_range = format!("{}..{}", old_commit, new_commit);
    let output = match Command::new("git")
        .arg("rev-list")
        .arg(commit_range)
        .output()
    {
        Ok(v) => v,
        Err(_e) => {
            let _ = log_to_file("_get_commits_range(): Failed to execute git rev-list");
            exit(1);
        }
    };
    let output_string = match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(_e) => {
            let _ = log_to_file("_get_commits_range(): Failed to utf8 the git rev result");
            exit(1);
        }
    };
    output_string
        .lines()
        .into_iter()
        .map(|line| line.to_owned())
        .collect()
}

fn _validator_enforce_squash_merge(commits_range: &[String]) -> Result<(), ValidationError> {
    if commits_range.len() > 1 {
        return Err(ValidationError::EnforceSquashMerge);
    }
    Ok(())
}

fn _validate_body_max_line_length(
    commit_bodies: &[Vec<String>],
    body_max_line_length: u8,
) -> Result<(), ValidationError> {
    for commit_body in commit_bodies {
        for line in commit_body {
            let number_of_characters = line.chars().count();
            if number_of_characters > body_max_line_length as usize {
                return Err(ValidationError::BodyMaxLineLength(body_max_line_length));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_title_format() {
        let regex_string = "^((\\bECSTU\\b)|(\\bINTERSCALE\\b))-\\d{1,}: \\w+.*$".to_owned();
        let regex = regex::Regex::new(&regex_string).unwrap();

        let commit_titles = vec!["ECSTU-123: This is the title description".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert!(result.is_ok());

        let commit_titles = vec!["ECSTU-: This is the title description".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert_eq!(
            result.err().unwrap(),
            ValidationError::TitleFormat(regex_string.clone())
        );

        let commit_titles = vec!["ECSTU-1:    ".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert_eq!(
            result.err().unwrap(),
            ValidationError::TitleFormat(regex_string.clone())
        );

        let commit_titles = vec!["ECSTU-1: a".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert!(result.is_ok());

        // New regex
        let regex_string = "^[A-Z]+-\\d{1,}: \\w+.*$".to_owned();
        let regex = regex::Regex::new(&regex_string).unwrap();

        let commit_titles = vec!["HELLO-1: a".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert!(result.is_ok());

        let commit_titles = vec!["HELLo-1: a".to_owned()];
        let result = validate_title_format(&commit_titles, &regex);
        assert_eq!(
            result.err().unwrap(),
            ValidationError::TitleFormat(regex_string.clone())
        );
    }

    #[test]
    fn test_validator_title_max_length() {
        let commit_titles = vec![
            "Title line 1".to_owned(),
            "Title line 2".to_owned(),
            "Title line 3".to_owned(),
        ];
        let result = _validate_title_max_length(&commit_titles, 12);
        assert!(result.is_ok());

        let commit_titles = vec![
            "Bigger title line 1".to_owned(),
            "Title line 2".to_owned(),
            "Title line 3".to_owned(),
        ];
        let result = _validate_title_max_length(&commit_titles, 12);
        assert_eq!(result.err().unwrap(), ValidationError::TitleMaxLength(12));

        let commit_titles = vec![
            "Title line 1".to_owned(),
            "Title line 2".to_owned(),
            "Bigger title line 3".to_owned(),
        ];
        let result = _validate_title_max_length(&commit_titles, 12);
        assert_eq!(result.err().unwrap(), ValidationError::TitleMaxLength(12));

        let commit_titles = vec![];
        let result = _validate_title_max_length(&commit_titles, 12);
        assert!(result.is_ok())
    }

    #[test]
    fn test_get_commit_title() {
        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title 1";
        assert_eq!(
            _get_commit_title(commit),
            "This is the commit title 1".to_owned()
        );

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

              This is the commit title 2";
        assert_eq!(
            _get_commit_title(commit),
            "This is the commit title 2".to_owned()
        );

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title 3         ";
        assert_eq!(
            _get_commit_title(commit),
            "This is the commit title 3".to_owned()
        );

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title 4

";
        assert_eq!(
            _get_commit_title(commit),
            "This is the commit title 4".to_owned()
        );

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title 5

This is the body
";
        assert_eq!(
            _get_commit_title(commit),
            "This is the commit title 5".to_owned()
        );
    }

    #[test]
    fn test_get_commit_body() {
        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title";
        let actual = _get_commit_body(commit);
        let expected: Vec<String> = vec![];
        assert_eq!(actual, expected);

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title

This is body line 1";
        let actual = _get_commit_body(commit);
        let expected: Vec<String> = vec!["This is body line 1".to_owned()];
        assert_eq!(actual, expected);

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title

This is body line 1

";
        let actual = _get_commit_body(commit);
        let expected: Vec<String> = vec!["This is body line 1".to_owned()];
        assert_eq!(actual, expected);

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title

This is body line 1
This is body line 2
This is body line 3
This is body line 4
This is body line 5";
        let actual = _get_commit_body(commit);
        let expected_length = 5;
        assert_eq!(actual.len(), expected_length);
        assert_eq!(actual.last().unwrap(), "This is body line 5");

        let commit = "tree d6b3dd4b08f63ba13479484508e0679d32a7891a
author John Doe <john.doe@gmail.com>
committer John Doe <john.doe@gmail.com>

This is the commit title

This is body line 1
This is body line 2
This is body line 3
This is body line 4
This is body line 5

              \t";
        let actual = _get_commit_body(commit);
        let expected_length = 5;
        assert_eq!(actual.len(), expected_length);
        assert_eq!(actual.last().unwrap(), "This is body line 5");
    }

    #[test]
    fn test_validator_body_required() {
        let commit_bodies = vec![
            vec!["Body line 1".to_owned()],
            vec!["Body line 1".to_owned()],
            vec!["Body line 1".to_owned()],
        ];
        let result = _validate_body_required(&commit_bodies);
        assert!(result.is_ok());

        let commit_bodies = vec![
            vec!["Body line 1".to_owned()],
            vec!["Body line 1".to_owned()],
            vec![],
        ];
        let result = _validate_body_required(&commit_bodies);
        assert_eq!(result.err().unwrap(), ValidationError::BodyRequired);

        let commit_bodies = vec![vec![]];
        let result = _validate_body_required(&commit_bodies);
        assert_eq!(result.err().unwrap(), ValidationError::BodyRequired);
    }

    #[test]
    fn test_validator_body_max_line_length() {
        let commit_bodies = vec![vec![
            "Body line 1".to_owned(),
            "Body line 2".to_owned(),
            "Body line 3".to_owned(),
        ]];
        let result = _validate_body_max_line_length(&commit_bodies, 11);
        assert!(result.is_ok());

        let commit_bodies = vec![vec![
            "Body line 1".to_owned(),
            "Body line 2".to_owned(),
            "Bigger body line 3".to_owned(),
        ]];
        let result = _validate_body_max_line_length(&commit_bodies, 11);
        assert_eq!(
            result.err().unwrap(),
            ValidationError::BodyMaxLineLength(11)
        );

        let commit_bodies = vec![vec![]];
        let result = _validate_body_max_line_length(&commit_bodies, 11);
        assert!(result.is_ok());
    }
}
