use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "docling",
    about = "Universal document conversion for AI pipelines",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert one or more documents
    Convert {
        /// Input file path
        #[arg(long, value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        to: OutputFmt,

        /// Output directory
        #[arg(long, value_name = "DIR", default_value = ".")]
        output_dir: PathBuf,
    },

    /// Download model artifacts
    Tools {
        #[command(subcommand)]
        sub: ToolsCommands,
    },
}

#[derive(Subcommand)]
enum ToolsCommands {
    /// Download ONNX model weights
    DownloadModels {
        #[arg(long, default_value = "~/.cache/docling/models")]
        output_dir: String,
    },
}

#[derive(ValueEnum, Clone)]
enum OutputFmt {
    Markdown,
    Json,
    Text,
    Html,
    Doctags,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            to,
            output_dir,
        } => {
            let converter = docling::DocumentConverter::default();
            match converter.convert(&input) {
                Ok(result) => {
                    if let Some(doc) = &result.document {
                        let (content, ext) = match to {
                            OutputFmt::Markdown => (doc.export_to_markdown(), "md"),
                            OutputFmt::Json => {
                                (serde_json::to_string_pretty(&doc).unwrap(), "json")
                            }
                            OutputFmt::Text => (doc.export_to_text(), "txt"),
                            OutputFmt::Html => (doc.export_to_html(), "html"),
                            OutputFmt::Doctags => (doc.export_to_document_tokens(), "dt"),
                        };
                        let stem = input
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let out_file = output_dir.join(format!("{}.{}", stem, ext));
                        std::fs::create_dir_all(&output_dir).ok();
                        std::fs::write(&out_file, &content).expect("Failed to write output");
                        println!("✓ Written to {}", out_file.display());
                    } else {
                        eprintln!("✗ Conversion failed: {:?}", result.status);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Tools {
            sub: ToolsCommands::DownloadModels { output_dir },
        } => {
            eprintln!(
                "Model download not yet implemented. Models should be placed in: {}",
                output_dir
            );
            // TODO: download ONNX model artifacts
        }
    }
}
