use std::io::{self, Write};
use std::path::Path;

/// Minimal interactive console (v1 scaffolding).
///
/// v1 behavior (mixed mode):
/// - Lines starting with '.' are handled as meta commands.
/// - Otherwise, the line is treated as a DQL statement for evaluation (TBD).
pub fn run_console(_store: &Path) -> Result<(), String> {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("dingo> ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        line.clear();

        let n = stdin.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == ".exit" || trimmed == ".quit" {
            break;
        }

        if trimmed == ".help" {
            print_help();
            continue;
        }

        if trimmed == ".collections" {
            // TODO: hook into cmd_list.
            println!("(collections: TBD)");
            continue;
        }

        if trimmed.starts_with(".use ") {
            // TODO: hook into session state.
            println!("(use: TBD)");
            continue;
        }

        if trimmed == ".status" {
            // TODO: show active collection/store.
            println!("(status: TBD)");
            continue;
        }

        // DQL-first: for now just echo until we wire the DQL engine.
        println!("(dql: TBD) {}", trimmed);
    }

    Ok(())
}

fn print_help() {
    println!("Residuum console v1 (mixed mode)");
    println!("Meta commands:");
    println!("  .help           Show this help");
    println!("  .collections   List collections");
    println!("  .use <name>    Set active collection (v1)");
    println!("  .status         Show console/session status");
    println!("  .exit / .quit   Exit console");
    println!("DQL-first: other lines are treated as DQL (v1 syntax TBD).\n");
}
