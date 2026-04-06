use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use fcars::cnc::{NominalDataset, cnc, cnc_bp};
use fcars::cnc_report::{render_cnc_bp_report_html, render_cnc_report_html};

#[derive(Debug)]
struct Config {
    input_path: PathBuf,
    output_path: PathBuf,
    class_attribute: String,
    object_column: Option<String>,
    bp: Option<usize>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(env::args().skip(1))?;
    let dataset = load_csv_dataset(
        &config.input_path,
        &config.class_attribute,
        config.object_column.as_deref(),
    )?;

    let title = default_title(&config);
    let html = match config.bp {
        Some(n) => {
            let result = cnc_bp(&dataset, n);
            render_cnc_bp_report_html(&dataset, &result, Some(&title))
        }
        None => {
            let result = cnc(&dataset);
            render_cnc_report_html(&dataset, &result, Some(&title))
        }
    }
    .map_err(|err| format!("Failed to render report HTML: {}", err))?;

    if let Some(parent) = config.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create output directory '{}': {}",
                    parent.display(),
                    err
                )
            })?;
        }
    }

    fs::write(&config.output_path, html).map_err(|err| {
        format!(
            "Failed to write report to '{}': {}",
            config.output_path.display(),
            err
        )
    })?;

    println!(
        "Report written to {}",
        config
            .output_path
            .canonicalize()
            .unwrap_or(config.output_path.clone())
            .display()
    );

    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let args = args.into_iter().collect::<Vec<_>>();
    if args.is_empty() {
        return Err(usage());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        process::exit(0);
    }

    let input_path = PathBuf::from(args[0].clone());
    let mut output_path = None;
    let mut class_attribute = None;
    let mut object_column = None;
    let mut bp = None;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--output" => {
                index += 1;
                output_path = Some(PathBuf::from(next_arg(&args, index, "--output")?));
            }
            "--class" => {
                index += 1;
                class_attribute = Some(next_arg(&args, index, "--class")?);
            }
            "--object-column" => {
                index += 1;
                object_column = Some(next_arg(&args, index, "--object-column")?);
            }
            "--bp" => {
                index += 1;
                let value = next_arg(&args, index, "--bp")?;
                let parsed = value.parse::<usize>().map_err(|_| {
                    format!(
                        "Invalid value '{}' for --bp. Expected a positive integer.",
                        value
                    )
                })?;
                if parsed == 0 {
                    return Err("--bp must be strictly positive.".to_string());
                }
                bp = Some(parsed);
            }
            unknown => return Err(format!("Unknown argument '{}'.\n\n{}", unknown, usage())),
        }
        index += 1;
    }

    Ok(Config {
        input_path,
        output_path: output_path
            .ok_or_else(|| format!("Missing required --output option.\n\n{}", usage()))?,
        class_attribute: class_attribute
            .ok_or_else(|| format!("Missing required --class option.\n\n{}", usage()))?,
        object_column,
        bp,
    })
}

fn next_arg(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index)
        .cloned()
        .ok_or_else(|| format!("Missing value for {}.\n\n{}", flag, usage()))
}

fn load_csv_dataset(
    path: &Path,
    class_attribute: &str,
    object_column: Option<&str>,
) -> Result<NominalDataset, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read '{}': {}", path.display(), err))?;
    let lines = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    if lines.len() < 2 {
        return Err("Input file must contain a header row and at least one data row.".to_string());
    }

    let headers = parse_record(lines[0]);
    if headers.is_empty() {
        return Err("Header row is empty.".to_string());
    }

    let class_index = headers
        .iter()
        .position(|header| header == class_attribute)
        .ok_or_else(|| {
            format!(
                "Class attribute '{}' is not present in the header.",
                class_attribute
            )
        })?;

    let object_index = match object_column {
        Some(column_name) => {
            let index = headers
                .iter()
                .position(|header| header == column_name)
                .ok_or_else(|| {
                    format!(
                        "Object column '{}' is not present in the header.",
                        column_name
                    )
                })?;
            if index == class_index {
                return Err("The object column cannot be the same as the class column.".to_string());
            }
            Some(index)
        }
        None => None,
    };

    let attributes = headers
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != object_index)
        .map(|(_, header)| header.clone())
        .collect::<Vec<_>>();

    let mut objects = Vec::new();
    let mut data = Vec::new();

    for (row_number, line) in lines.iter().skip(1).enumerate() {
        let fields = parse_record(line);
        if fields.len() != headers.len() {
            return Err(format!(
                "Row {} has {} field(s), expected {}. The parser expects a simple CSV file without quoted commas.",
                row_number + 2,
                fields.len(),
                headers.len()
            ));
        }

        let object_name = match object_index {
            Some(index) => fields[index].clone(),
            None => format!("obj{}", row_number + 1),
        };

        let mut row = HashMap::new();
        for (index, header) in headers.iter().enumerate() {
            if Some(index) == object_index {
                continue;
            }
            row.insert(header.clone(), fields[index].clone());
        }

        objects.push(object_name);
        data.push(row);
    }

    Ok(NominalDataset::new(
        objects,
        attributes,
        class_attribute.to_string(),
        data,
    ))
}

fn parse_record(line: &str) -> Vec<String> {
    line.split(',')
        .map(|field| field.trim().to_string())
        .collect()
}

fn default_title(config: &Config) -> String {
    let stem = config
        .input_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("dataset");
    match config.bp {
        Some(n) => format!(
            "CNC-BP Report - {} ({} minority class{})",
            stem,
            n,
            if n == 1 { "" } else { "es" }
        ),
        None => format!("CNC Report - {}", stem),
    }
}

fn usage() -> String {
    [
        "Usage:",
        "  cargo run --bin cnc_report -- <input-file.csv> --class <class-column> --output <report.html> [options]",
        "",
        "Options:",
        "  --object-column <name>   Column used as object identifier. If omitted, obj1, obj2, ... are generated.",
        "  --bp <n>                 Run CNC-BP instead of CNC, keeping the n most minority classes.",
        "  --help                   Show this help message.",
        "",
        "Notes:",
        "  The input parser expects a simple CSV file with a header row and without quoted commas inside values.",
        "",
        "Example:",
        "  cargo run --bin cnc_report -- data/weather.csv --class Play --object-column Id --output target/reports/weather.html",
    ]
    .join("\n")
}
