use clap::{Parser, Subcommand};
use kosei_cli::{commands, config::ConfigLoader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Apply all replacements for the named environment")]
    Switch {
        #[arg(help = "Environment name defined in kosei.yaml")]
        environment: String,

        #[arg(
            short = 'd',
            long,
            default_value_t = false,
            help = "Preview changes without writing to disk"
        )]
        dry_run: bool,
    },
    #[command(about = "Migrate kosei.config.json to kosei.yaml")]
    Migrate,
    #[command(about = "Initialize a new kosei.yaml configuration file")]
    Init {
        #[arg(help = "Path to initialize the config file in (defaults to current directory)")]
        path: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Init { path } => commands::init(&path),
        Commands::Migrate => commands::migrate(),
        Commands::Switch {
            environment,
            dry_run,
        } => {
            let config = match ConfigLoader::load() {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };
            commands::switch(&environment, &config, dry_run)
        }
    };

    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
