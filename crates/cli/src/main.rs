use clap::{Args, Parser, Subcommand};
use docling_converter::DocumentConverter;
use docling_core::base_models::InputFormat;
use std::path::PathBuf;

// ============================================================
// CLI definition
// ============================================================

/// docling-rs — Fast document parsing for AI agents (Rust port of Docling)
#[derive(Parser, Debug)]
#[command(
    name = "docling",
    version = env!("CARGO_PKG_VERSION"),
    author,
    about = "Convert documents (PDF, DOCX, HTML, Markdown, ...) to a unified AI-ready format"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert one or more documents.
    Convert(ConvertArgs),
    /// Utility tools (model download, etc.)
    Tools(ToolsArgs),
}

// ─── Convert command ─────────────────────────────────────────────────────────

#[derive(Args, Debug)]
struct ConvertArgs {
    /// Input file path(s).
    #[arg(short, long, required = true, num_args = 1..)]
    input: Vec<PathBuf>,

    /// Output directory (defaults to same directory as input).
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Output format: json | markdown | text  (default: json)
    #[arg(long, default_value = "json")]
    to: String,

    /// Path to directory containing downloaded model artifacts.
    #[arg(long, env = "DOCLING_ARTIFACTS_PATH")]
    artifacts_path: Option<PathBuf>,

    /// Disable OCR (faster, but misses scanned content).
    #[arg(long)]
    no_ocr: bool,

    /// Disable table structure recognition.
    #[arg(long)]
    no_table_structure: bool,

    /// Document processing timeout in seconds.
    #[arg(long)]
    timeout: Option<f64>,

    /// Whether to output verbose logs.
    #[arg(short, long)]
    verbose: bool,
}

// ─── Tools command ────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
struct ToolsArgs {
    #[command(subcommand)]
    action: ToolsAction,
}

#[derive(Subcommand, Debug)]
enum ToolsAction {
    /// Download model artifacts from HuggingFace Hub.
    DownloadModels {
        /// Directory to save models into.
        #[arg(long, default_value = "~/.docling/models")]
        output_dir: PathBuf,
    },
    /// Print the resolved artifacts path.
    ShowArtifactsPath,
}

// ============================================================
// Entry point
// ============================================================

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    match cli.command {
        Commands::Convert(args) => run_convert(args),
        Commands::Tools(args) => run_tools(args),
    }
}

// ─── Convert handler ─────────────────────────────────────────────────────────

fn run_convert(args: ConvertArgs) {
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    // Build converter
    let mut converter = DocumentConverter::new();
    if let Some(path) = args.artifacts_path {
        converter = converter.with_artifacts_path(path);
    }
    if args.no_ocr {
        converter = converter.without_ocr();
    }
    if let Some(t) = args.timeout {
        converter = converter.with_timeout(t);
    }

    let output_format = args.to.to_lowercase();

    for input_path in &args.input {
        log::info!("Converting {:?}…", input_path);

        match converter.convert(input_path) {
            Ok(result) => {
                if let Some(doc) = &result.document {
                    let output_dir = args
                        .output_dir
                        .as_deref()
                        .or_else(|| input_path.parent())
                        .unwrap_or_else(|| std::path::Path::new("."));

                    let stem = input_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output");

                    let (content, extension) = match output_format.as_str() {
                        "markdown" | "md" => (docling_utils::export::to_markdown(doc), "md"),
                        "text" | "txt" => (docling_utils::export::to_text(doc), "txt"),
                        _ => match docling_utils::export::to_json(doc) {
                            Ok(json) => (json, "json"),
                            Err(e) => {
                                eprintln!("JSON serialization error: {}", e);
                                continue;
                            }
                        },
                    };

                    let out_path = output_dir.join(format!("{}.{}", stem, extension));
                    match std::fs::write(&out_path, &content) {
                        Ok(_) => {
                            println!("✓ {:?} → {:?}", input_path, out_path);
                        }
                        Err(e) => {
                            eprintln!("✗ Failed to write {:?}: {}", out_path, e);
                        }
                    }
                } else {
                    eprintln!(
                        "✗ Conversion of {:?} failed: {:?} — {:?}",
                        input_path, result.status, result.errors
                    );
                }
            }
            Err(e) => {
                eprintln!("✗ Error converting {:?}: {}", input_path, e);
            }
        }
    }
}

// ─── Tools handler ───────────────────────────────────────────────────────────

fn run_tools(args: ToolsArgs) {
    match args.action {
        ToolsAction::DownloadModels { output_dir } => {
            println!("Downloading models to {:?}…", output_dir);
            println!(
                "Note: Run `pip install docling && docling-tools download-models` to download\n\
                 pre-trained ONNX models, or set DOCLING_ARTIFACTS_PATH to an existing directory."
            );
            println!("Future: this will fetch models from HuggingFace Hub directly.");
        }
        ToolsAction::ShowArtifactsPath => {
            if let Ok(p) = std::env::var("DOCLING_ARTIFACTS_PATH") {
                println!("DOCLING_ARTIFACTS_PATH={}", p);
            } else {
                let home = dirs::home_dir()
                    .map(|h| h.join(".docling").join("models").display().to_string())
                    .unwrap_or_else(|| "~/.docling/models".to_string());
                println!("Default artifacts path: {}", home);
            }
        }
    }
}
