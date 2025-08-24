use clap::Parser;

use crate::{cli::Args, printer::print_results, processor::process_docks};

mod models;
mod cli;
mod processor;
mod printer;

fn main() {
  let args_raw = Args::parse();
  
  // 입력 유효성 검사
  if let Err(e) = args_raw.validate_input() {
    eprintln!("Error: {e}");
    std::process::exit(1);
  }

  // dock sorting 및 로직 processing
  let processing_result = process_docks(&args_raw);

  // print final results
  print_results(&args_raw, &processing_result);
}
