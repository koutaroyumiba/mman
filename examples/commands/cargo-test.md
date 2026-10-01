# cargo-test

## NAME

`cargo test` — compile and run Rust tests

## SYNOPSIS

```text
cargo test [OPTIONS] [TEST_FILTER] [-- TEST_OPTIONS]
```

## DESCRIPTION

`cargo test` compiles the current package in test mode and runs its unit,
integration, and documentation tests. By default, Cargo captures output from
passing tests and runs tests concurrently.

Arguments before `--` configure Cargo. Arguments after `--` are passed to the
Rust test harness.

## COMMON INVOCATIONS

Run all tests:

```sh
cargo test
```

Run tests whose names contain a string:

```sh
cargo test discover
```

Show output produced by passing tests:

```sh
cargo test -- --show-output
```

Run tests serially when they share process-global state:

```sh
cargo test -- --test-threads=1
```

Run one integration-test target:

```sh
cargo test --test cli
```

## EXIT STATUS

Returns success when compilation succeeds and every selected test passes.
Returns a non-zero status after a compilation error, test failure, or test
process failure.

## NOTES

- Unit tests commonly live in a `#[cfg(test)]` module beside the code they test.
- Integration tests live in `tests/` and use the package through its public
  interface.
- Tests that mutate environment variables or the working directory can
  interfere with one another when run concurrently.
- A filter selects tests by name; it is not necessarily an exact match.

## SEE ALSO

`cargo-check`, `cargo-build`, `cargo-clippy`, `rust-test`
