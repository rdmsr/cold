use clap::Parser;
use cold::error::*;
use cold::static_link::*;
use log::*;
use simplelog::*;

/// Simple program to greet a person
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
        true => LevelFilter::Trace,
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
        if let Err(e) = statically_link_files(args.files, args.output) {
            handle_error(e)
        }
    } else {
        todo!("Dynamic linking");
    }
}
