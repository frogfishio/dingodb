Findings: crates/residuum-cli/src/console.rs exists with run_console(_store) but crates/residuum-cli/src/main.rs currently has no Console subcommand/variant dispatch.
Next steps: add Command::Console { store: PathBuf } and in run() match arm call console::run_console(&store).
Then implement minimal RQL evaluation path for scripted tests.
