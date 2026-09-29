#![forbid(unsafe_code)]

//! FocusCompass komut satiri giris noktasi.
//!
//! Burada yalnizca arguman ayristirma, cikti yazma ve hata yonlendirmesi vardir;
//! butun is mantigi `focuscompass::cli::calistir` icinde ve test edilebilir
//! sekilde durur.

use std::process::ExitCode;

use clap::Parser;

use focuscompass::cli::{calistir, Cli};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match calistir(&cli) {
        Ok(cikti) => {
            if !cikti.is_empty() {
                println!("{cikti}");
            }
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("hata: {hata}");
            ExitCode::FAILURE
        }
    }
}
