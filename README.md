[![Build status](https://github.com/Vagelis-Prokopiou/GitPolicyEnforcer/workflows/ci/badge.svg)](https://github.com/Vagelis-Prokopiou/GitPolicyEnforcer/actions)

# GitPolicyEnforcer

This is a command line utility written in [Rust](https://www.rust-lang.org/),
that helps you utilize Git hooks,
to enforce various policies.

It currently supports only the `update` server side Git hook. The goal is to include client side hooks too.

The tool has been developed so far for usage with self-hosted GitLab CE instances, due to internal needs. It should also work though, for bare git servers (maybe with some minor changes).

This is work in progress. You are welcome to contribute if this project is useful to you.

## Usage
For now, please check the following YouTube video:

[Rust: GitPolicyEnforcer](https://youtu.be/CNmWKYmWAFU)

According to time availability, more documentation will be added, and further development will happen.

## On the fly regex validation

You can quickly test you regexes both for successful regex creation and value validation, before deploying by using the respective feature. Example usage:

Expected to succeed:
```shell
gpe --regex '^\d test$' --regex-value '1 test'
```

Expected to fail at value validation:
```shell
gpe --regex '^\d test$' --regex-value 'hello world'
```

Expected to fail during regex creation:
```shell
gpe --regex '^\e{1,1}$' --regex-value 'hello world'
```

## Building
You need a [Rust installation](https://www.rust-lang.org/) in order to compile it.

To build GitPolicyEnforcer:

```
git clone https://github.com/Vagelis-Prokopiou/GitPolicyEnforcer
cd GitPolicyEnforcer
cargo build --release
./target/release/gpe --help
```

The produced binary is `gpe` located in the `GitPolicyEnforcer/target/release` directory.


## Licence
GitPolicyEnforcer is distributed under the terms of both the [MIT license](https://opensource.org/licenses/MIT) and the [Apache License (Version 2.0)](https://www.apache.org/licenses/LICENSE-2.0).

