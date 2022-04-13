#![allow(clippy::needless_return)]

pub mod loggers;
pub mod structs;
pub mod traits;
pub mod validators;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use structs::*;

pub fn parse_rules<P: AsRef<Path>>(path: P) -> Result<Rules, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let rules: Rules = serde_json::from_reader(reader)?;
    return Ok(rules);
}

pub fn get_hook(path: &str) -> Hook {
    // The path is like this /aa/bbb/
    let parts: Vec<&str> = path
        .split('/')
        .collect::<Vec<&str>>()
        .iter()
        .filter(|value| !value.is_empty())
        .copied()
        .collect();
    Hook::from(*parts.last().unwrap_or(&""))
}

pub fn get_stdin_data() -> String {
    let mut stdin_input = String::new();
    let stdin = std::io::stdin();
    let mut stdin_handle = stdin.lock();
    stdin_handle.read_to_string(&mut stdin_input).unwrap();
    stdin_input = stdin_input.replace('\n', "");
    stdin_input
}

pub fn get_repo_path(input: &str) -> String {
    // Remove everything after .git, if exists and return the first part.
    let parts: Vec<&str> = input.split(".git").collect();
    format!("{}.git", parts[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path() {
        let path_str = "/home/user/repo.git";
        assert_eq!(get_repo_path(path_str), path_str);

        let path_str = "/home/user/repo.git/";
        assert_eq!(get_repo_path(path_str), "/home/user/repo.git");

        let path_str = "/home/user/repo.git/foo/bar";
        assert_eq!(get_repo_path(path_str), "/home/user/repo.git");

        let path_str = "/var/opt/gitlab/git-data/repositories/@hashed/4b/22/4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a.git/custom_hooks";
        assert_eq!(
            get_repo_path(path_str),
            "/var/opt/gitlab/git-data/repositories/@hashed/4b/22/4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a.git"
        );
    }
}
