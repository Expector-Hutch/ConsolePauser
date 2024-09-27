use clap::Parser;

use lib::*;

/// ConsolePauser
/// copyright (GPLv3) 2024 Expector(expector.netlify.app)
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to executable file
    #[clap(required = true)]
    file_path: String,

    /// Time limit
    #[clap(short, long, default_value_t = -1)]
    time_limit: i32,
}

fn main() {
    let args = Args::parse();

    let file_path: &str = &args.file_path.to_string();
    let _ = set_console_title(file_path);

    let return_value = execute_file(file_path);

    info("\nFinished", &format!("file exited after {}s with return value {}", return_value.1 , return_value.0), Status::SUCCESS);
    pause_exit(Status::SUCCESS);
}
