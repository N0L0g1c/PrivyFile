use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use privyfile_core::{
    batch, clean_file, collect_files_from_paths, get_metadata, profiles::apply_profile, shred_file,
    shred_folder, BatchAction, BatchItem, CleanOptions, CleanProfileId, ShredMethod, ShredOptions,
};

#[derive(Parser)]
#[command(name = "privyfile", about = "Sanitize files before you share or delete them")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Metadata {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Clean {
        path: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long, default_value_t = true)]
        preserve_original: bool,
        #[arg(long)]
        profile: Option<ProfileArg>,
    },
    Shred {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = MethodArg::Random)]
        method: MethodArg,
        #[arg(long, default_value_t = 1)]
        passes: u8,
    },
    Batch {
        path: PathBuf,
        #[arg(long)]
        clean: bool,
        #[arg(long)]
        shred: bool,
        #[arg(long)]
        recursive: bool,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum MethodArg {
    Random,
    Dod,
    Seven,
    Custom,
}

#[derive(Clone, Copy, ValueEnum)]
enum ProfileArg {
    Social,
    Legal,
    Photo,
    All,
}

impl From<MethodArg> for ShredMethod {
    fn from(value: MethodArg) -> Self {
        match value {
            MethodArg::Random => ShredMethod::Random1Pass,
            MethodArg::Dod => ShredMethod::Dod5220,
            MethodArg::Seven => ShredMethod::SevenPass,
            MethodArg::Custom => ShredMethod::Custom,
        }
    }
}

impl From<ProfileArg> for CleanProfileId {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Social => CleanProfileId::SocialMediaShare,
            ProfileArg::Legal => CleanProfileId::LegalDocument,
            ProfileArg::Photo => CleanProfileId::PhotoBackup,
            ProfileArg::All => CleanProfileId::RemoveAll,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Metadata { path, json } => {
            let report = get_metadata(&path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("File: {}", report.file_path);
                println!("Type: {}", report.file_type);
                println!("Privacy score: {}/100", report.privacy_score);
                for tag in report.tags {
                    println!("  {} = {} [{:?}]", tag.name, tag.value, tag.category);
                }
            }
        }
        Commands::Clean {
            path,
            output,
            preserve_original,
            profile,
        } => {
            let mut options = CleanOptions {
                output_dir: output.map(|value| value.to_string_lossy().into_owned()),
                preserve_original,
                ..Default::default()
            };
            if let Some(profile) = profile {
                options.profile = Some(profile.into());
            }
            options = apply_profile(options);
            let result = clean_file(&path, &options)?;
            println!("Cleaned: {}", result.source_path);
            if let Some(output_path) = result.output_path {
                println!("Output: {output_path}");
            }
            println!(
                "Privacy score: {} -> {}",
                result.privacy_score_before, result.privacy_score_after
            );
        }
        Commands::Shred {
            path,
            method,
            passes,
        } => {
            let options = ShredOptions {
                method: method.into(),
                passes,
            };
            if path.is_dir() {
                let results = shred_folder(&path, true, &options)?;
                println!("Shredded {} files", results.len());
            } else {
                let result = shred_file(&path, &options)?;
                println!(
                    "Shredded {} ({} passes)",
                    result.file_path, result.passes_completed
                );
            }
        }
        Commands::Batch {
            path,
            clean,
            shred,
            recursive,
            output,
        } => {
            let files = collect_files_from_paths(&[path.to_string_lossy().into_owned()], recursive)?;
            let items: Vec<BatchItem> = files
                .into_iter()
                .map(|file| BatchItem {
                    path: file,
                    action: if clean && shred {
                        BatchAction::CleanAndShred
                    } else if shred {
                        BatchAction::Shred
                    } else {
                        BatchAction::Clean
                    },
                })
                .collect();

            let clean_options = CleanOptions {
                output_dir: output.map(|value| value.to_string_lossy().into_owned()),
                ..Default::default()
            };
            let result = batch::process_batch(&items, &clean_options, &ShredOptions::default(), None)?;
            println!("Processed {} files", result.items.len());
            if let Some(report) = result.report_path {
                println!("Report: {report}");
            }
        }
    }

    Ok(())
}
