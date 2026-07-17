use archiver::delivery::{install_interrupt, run, Cli};
use clap::Parser;

fn main() -> std::process::ExitCode {
    install_interrupt();
    let cli = Cli::parse();
    run(cli)
}
