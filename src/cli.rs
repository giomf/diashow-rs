use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Fotobox diashow", long_about = None)]
#[clap(propagate_version = true)]
pub enum Cli {
    /// Start the diashow
    Start(Start),
}

#[derive(Debug, Parser, Clone)]
pub struct Start {
    /// Folder where to search for images
    #[arg(long)]
    pub images: String,

    /// Duration that one image is displayed in seconds
    #[arg(long)]
    pub duration: u64,

    /// Index where to start. A negative number will start at the end.
    #[arg(long, allow_negative_numbers(true))]
    pub start_index: Option<i64>,

    /// Duration of one fade iteration in miliseconds.
    #[arg(long)]
    pub fade_iteration_duration: Option<u64>,

    /// Step size of one fade iteration.
    #[arg(long)]
    pub fade_iteration_step: Option<u8>,
}
