mod cli;
mod command;
mod launch;

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let status = cli::run(
        &arguments,
        &bourbon_core::default_database_path(),
        launch::launch_app,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    ExitCode::from(status)
}
