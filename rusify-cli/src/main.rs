use clap::{Parser, Subcommand};
use std::process::ExitCode;
use rusify_cli::apple_target::ApplePlatform;
use rusify_cli::models::{LibType, Mode, FeatureOptions, Config};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands, // 将 Option<Commands> 改为 Commands
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(index = 1)]
        crate_name: String,
    },
    Build {
        #[arg(short, long, trailing_var_arg = true, num_args = 1..=4, ignore_case = true)]
        platforms: Option<Vec<ApplePlatform>>,

        #[arg(long)]
        /// Build package for the specified target triplet only.
        target: Option<String>,

        #[arg(short = 'n', long = "name")]
        package_name: Option<String>,

        #[arg(long, default_value = "RustFramework")]
        xcframework_name: String,

        #[arg(short, long)]
        /// Build package optimized for release (default: debug)
        release: bool,

        #[arg(long, ignore_case = true, default_value_t = LibType::Static)]
        /// Choose how the library should be built. By default, this will be derived from the lib type provided in Cargo.toml
        lib_type: LibType,

        #[arg(long)]
        /// Disable warnings in generated Swift package code
        suppress_warnings: bool,

        #[arg(short = 'F', long, trailing_var_arg = true)]
        features: Option<Vec<String>>,

        #[arg(long)]
        all_features: bool,

        #[arg(long)]
        no_default_features: bool,

        #[arg(short, long, global = true)]
        /// Silence all output except errors and interactive prompts
        silent: bool,
    
        #[arg(short = 'y', long, global = true)]
        /// Accept all default selections from all interactive prompts.
        ///
        /// This is especially useful when invoked in an environment, where no user interaction is possible,
        /// e.g. a test runner. Prompts without a default state will be skipped as well, resulting in an error
        /// if the corresponding value was not set as an argument beforehand.
        accept_all: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { crate_name } => {
            rusify_cli::init::init_crate(crate_name);
            Ok(())
        }
        Commands::Build {
            platforms,
            target,
            package_name,
            xcframework_name,
            release,
            lib_type,
            suppress_warnings,
            features,
            all_features,
            no_default_features,
            silent,
            accept_all,
        } => {
            rusify_cli::package::build_swift_package(
                platforms,
                target.as_deref(),
                package_name,
                xcframework_name,
                suppress_warnings,
                Config { silent, accept_all },
                if release { Mode::Release } else { Mode::Debug },
                lib_type,
                FeatureOptions {
                    features,
                    all_features,
                    no_default_features,
                },
            )
        }
    };

    if let Err(e) = result {
        eprintln!("\n");
        eprintln!("Failed due to the following error: \n{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}