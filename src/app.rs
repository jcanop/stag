use anyhow::Result;
use clap::{ Parser, Subcommand };
use crate::{ stegano, tui, util, wire::EmbeddedData  };
use crate::crypto::{ self, HashAlgorithm, EncryptionAlgorithm };
use image::ImageReader;
use log::{ info, LevelFilter };
use std::fmt::Debug;
use std::fs;
use std::path::{ Path, PathBuf };
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Missing Input")]
    MissingInput,
    #[error("Missing Password")]
    MissingPassword,
}

#[derive(Debug, Parser)]
#[command(name = "Stag")]
#[command(version = "1.0.0")]
#[command(about = "Steganography command line utility", long_about = "This command-line utility lets users embed and extract data from images using steganography. Steganography is the practice of hiding a secret message, file, or image inside another ordinary, non-secret file or physical object to avoid detection; in this case, image files.")]
struct Args {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Command
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Embed data into an image
    Embed {
        /// Input image
        input_image: PathBuf,
        /// Output image
        output_image: PathBuf,
        /// Input filename
        #[arg(short, long, value_name = "FILE")]
        input: Option<PathBuf>,
        /// Encrypt input data
        #[arg(short, long)]
        encrypt: bool,
        /// Encryption password
        #[arg(short, long, value_name = "PASSPHRASE")]
        password: Option<String>,
    },
    /// Extract data from an image
    Extract {
        /// Input image
        input_image: PathBuf,
        /// Output filename
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Decrypt password
        #[arg(short, long, value_name = "PASSPHRASE")]
        password: Option<String>,
        /// Display the output in the console
        #[arg(short, long)]
        text: bool,
    },
}

/// Main execution method of the application.
///
/// Returns:
/// Ok or any error that ocurred during execution.
pub fn run() -> Result<()> {
    let args = Args::parse();

    let level = if args.verbose { LevelFilter::Info } else { LevelFilter::Warn };
    pretty_env_logger::formatted_timed_builder()
        .filter_level(level)
        .init();

    match &args.command {
        Command::Embed {
            input_image: img,
            output_image: output,
            input,
            encrypt: enc,
            password: pass,
        } => embed(img, output, input.as_ref(), *enc, pass.as_deref()),
        Command::Extract {
            input_image: img,
            output,
            password: pass,
            text: txt
        } => extract(img, output.as_ref(), pass.as_deref(), *txt),
    }
}

/// Embed a file or an input text into an image.
///
/// Parameters:
/// * input_image: Input image filename.
/// * output: Output image filename.
/// * input: An optional input filename.
/// * encrypt: Indicates if encrypt the input content.
/// * password: Password used to encrypt the input content.
///
/// Returns:
/// Ok or any error that ocurred during execution.
fn embed<P: AsRef<Path> + Debug>(input_image: P, output: P, input: Option<P>, encrypt: bool, password: Option<&str>) -> Result<()> {
    // --- Load input image ---
    info!("Loading image: {:?}", &input_image);
    let img = ImageReader::open(input_image)?.decode()?;

    // --- Load or read from the console the input data ---
    let mut buffer: Vec<u8> = match input {
        Some(f) => {
            info!("Loading input file: {:?}", f);
            fs::read(f)?
        },
        None => {
            info!("Input file not provided, rendering input text box");
            tui::input::read_input()?.ok_or(AppError::MissingInput)?.into_bytes()
        },
    };

    // --- If needed, encrypt the content ---
    if encrypt {
        info!("Encrypting input data");
        let pass = match password {
            Some(p) => {
                info!("Using provided password");
                p
            },
            None => {
                info!("Displaying password text box");
                &tui::password::read_password(true)?.ok_or(AppError::MissingPassword)?
            },
        };
        buffer = crypto::argon2::encrypt(&buffer, pass)?;
    }

    // --- Creates embedded data object ---
    let (hash_alg, enc_alg) = if encrypt {
        (HashAlgorithm::None, EncryptionAlgorithm::Argon2)
    } else {
        (HashAlgorithm::XXH32, EncryptionAlgorithm::None)
    };
    info!("Preparing embedded data: {:?}, {:?}", hash_alg, enc_alg);
    let ed = EmbeddedData::new(hash_alg, enc_alg, buffer)?;

    // --- Embed the input into the input image ---
    info!("Embedding the data");
    let img = stegano::embed(&img, ed)?;

    // --- Write output file ---
    let filename = util::ensure_png_extension(output.as_ref());
    info!("Writing output file: {:?}", &filename);
    img.save(filename)?;

    // --- Result ---
    info!("Done");
    Ok(())
}

/// Extract the embeded data from an image.
///
/// Parameters:
/// * input_image: Input image filename.
/// * output: Optional output filename.
/// * password: Optional password for decrypting the extracted data.
/// * text: Indicates if print the extracted data in the console.
///
/// Returns:
/// Ok or any error that ocurred during execution.
fn extract<P: AsRef<Path> + Debug>(input_image: P, output: Option<P>, password: Option<&str>, text: bool) -> Result<()> {
    // --- Load image file ---
    info!("Loading image: {:?}", &input_image);
    let img = ImageReader::open(input_image)?.decode()?;

    // --- Extract the data from the input image ---
    info!("Extracting embedded data");
    let ed = stegano::extract(&img)?;
    let decrypt = ed.header.enc_algorithm != EncryptionAlgorithm::None;
    let mut buffer = ed.data;

    // --- If needed, decrypt the content ---
    if decrypt {
        info!("Decrypting embedded data");
        let pass = match password {
            Some(p) => {
                info!("Using provided password");
                p
            },
            None => {
                info!("Displaying password text box");
                &tui::password::read_password(false)?.ok_or(AppError::MissingPassword)?
            },
        };
        buffer = crypto::argon2::decrypt(&buffer, pass)?;
    }

    // --- Wirte the output into a file ---
    if let Some(f) = output {
        info!("Writing output file: {:?}", f);
        fs::write(f, &buffer)?;
    }

    // --- Print in to the console if needed ---
    if text {
        info!("Displaying extracted data");
        let txt = String::from_utf8(buffer)?;
        println!("{}", &txt);
    }

    // --- Result ---
    Ok(())
}
