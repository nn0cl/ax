use archiver::delivery::{run, Cli};
use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    run(cli)
}
