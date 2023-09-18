/*
 * Planet Alcatraz archive tool
 * Copyright (C) 2023 svenvvv
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::fs::File;
use std::str;
use std::io::{BufReader, Write};
use clap::{ArgGroup, Parser};
use tabled::{Table, Tabled};

use papak::Archive;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = "Planet Alcatraz pak archive extraction tool")]
#[clap(group(
    ArgGroup::new("operations")
        .required(true)
        .args(&["list", "extract"]),
    ))]
struct Cli {
    #[clap(short = 'l', long, help = "List files contained in the archive")]
    list: bool,
    #[clap(short = 'e', long, help = "Extract selected, or all, files from the archive")]
    extract: bool,
    #[clap(help = "Input archive file")]
    archive: std::path::PathBuf,
    #[clap(group = "input", help = "List of files to extract")]
    files: Option<Vec<String>>,
}

#[derive(Debug, Tabled)]
struct DisplayEntry<'a> {
    size: u32,
    compressed: u32,
    filename: &'a String,
}

enum CliOperation {
    LIST,
    EXTRACT,
}

fn main() {
    let args = Cli::parse();

    let input = File::open(args.archive)
        .expect("Failed to open archive");
    let reader = BufReader::new(input);
    let mut archive_obj  = Archive::from_reader(Box::new(reader))
        .expect("Failed to load archive");

    let operation = if args.list {
        CliOperation::LIST
    } else if args.extract {
        CliOperation::EXTRACT
    } else {
        panic!("Invalid operation")
    };

    match operation {
        CliOperation::LIST => {
            let entries: Vec<DisplayEntry> = archive_obj.entries.iter()
                .map(|(_, e)| DisplayEntry{
                    filename: &e.filename,
                    size: e.uncompressed_size.clone(),
                    compressed: e.compressed_size.clone(),
                })
                .collect();
            println!("{}", Table::new(entries));
        },
        CliOperation::EXTRACT => {
            let extract_filenames = match args.files.as_ref() {
                Some(files) => files.into_iter()
                    .map(|filename| filename.clone())
                    .collect::<Vec<String>>(),
                None => archive_obj.entries.iter()
                    .map(|(filename, _)| filename.clone())
                    .collect::<Vec<String>>(),
            };
            extract_filenames.into_iter().for_each(|filename| {
                println!("Extracting {}...", filename);
                let data = archive_obj.extract(&filename)
                    .expect("Failed to extract file");
                let mut outfile = File::create(filename)
                    .expect("Failed to create output file");
                outfile.write_all(&*data)
                    .expect("Failed to write output file");
            });
        },
    }
}
