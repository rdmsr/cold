use clap::Parser;
use cold::static_link::statically_link_files;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 's', long = "static")]
    _static: bool,

    #[arg(short, long)]
    debug: bool,

    #[arg(short, long, value_name = "FILE")]
    output: String,

    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let level_filter = match args.debug {
        false => LevelFilter::Error,
        true => LevelFilter::Info,
    };

    TermLogger::init(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )
    .unwrap();

    info!(
        "input files: {:?}, output file: {}",
        args.files, args.output
    );

    if args._static {
        if let Err(err) = statically_link_files(args.files, args.output) {
            err.report();
        }
    } else {
        todo!("Dynamic linking");
    }
}
