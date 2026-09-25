mod report;
mod timesheet;

use std::env;
use std::fs;
use std::process;
use timesheet::ParseMode;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(e) = run(&args) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.get(1).map(String::as_str) {
        Some("convert") => cmd_convert(&args[2..]),
        Some("report") => cmd_report(&args[2..]),
        Some("help") | Some("--help") | None => {
            print!("{}", usage());
            Ok(())
        }
        Some(other) => Err(format!("unknown command '{}'\n\n{}", other, usage())),
    }
}

fn usage() -> String {
    "tsconv - convert timesheets between csv and punch formats\n\n\
     usage:\n\
     \x20 tsconv convert --from <csv|punch> --to <csv|punch> --input <file> [--output <file>] [--lenient]\n\
     \x20 tsconv report --input <file> --format <csv|punch> [--json] [--lenient]\n\n\
     \x20 --lenient skips malformed rows (with a warning on stderr) instead of\n\
     \x20 stopping at the first one, which is the default (strict) behavior.\n"
        .to_string()
}

fn cmd_convert(args: &[String]) -> Result<(), String> {
    let mut from = None;
    let mut to = None;
    let mut input = None;
    let mut output = None;
    let mut lenient = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => from = Some(next_value(args, &mut i, "--from")?),
            "--to" => to = Some(next_value(args, &mut i, "--to")?),
            "--input" => input = Some(next_value(args, &mut i, "--input")?),
            "--output" => output = Some(next_value(args, &mut i, "--output")?),
            "--lenient" => {
                lenient = true;
                i += 1;
            }
            other => return Err(format!("unknown argument '{}' for convert", other)),
        }
    }

    let from = from.ok_or("convert requires --from <csv|punch>")?;
    let to = to.ok_or("convert requires --to <csv|punch>")?;
    let input = input.ok_or("convert requires --input <file>")?;
    let mode = if lenient {
        ParseMode::Lenient
    } else {
        ParseMode::Strict
    };

    let contents =
        fs::read_to_string(&input).map_err(|e| format!("reading '{}': {}", input, e))?;

    let outcome = match from.as_str() {
        "csv" => timesheet::parse_csv(&contents, mode)?,
        "punch" => timesheet::parse_punch(&contents, mode)?,
        other => return Err(format!("unknown format '{}' (expected csv or punch)", other)),
    };
    for warning in &outcome.warnings {
        eprintln!("warning: {}", warning);
    }
    let entries = outcome.entries;

    let rendered = match to.as_str() {
        "csv" => timesheet::write_csv(&entries),
        "punch" => timesheet::write_punch(&entries),
        other => return Err(format!("unknown format '{}' (expected csv or punch)", other)),
    };

    match output {
        Some(path) => {
            fs::write(&path, rendered).map_err(|e| format!("writing '{}': {}", path, e))?;
        }
        None => print!("{}", rendered),
    }

    Ok(())
}

fn cmd_report(args: &[String]) -> Result<(), String> {
    let mut input = None;
    let mut format = None;
    let mut json = false;
    let mut lenient = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => input = Some(next_value(args, &mut i, "--input")?),
            "--format" => format = Some(next_value(args, &mut i, "--format")?),
            "--json" => {
                json = true;
                i += 1;
            }
            "--lenient" => {
                lenient = true;
                i += 1;
            }
            other => return Err(format!("unknown argument '{}' for report", other)),
        }
    }

    let input = input.ok_or("report requires --input <file>")?;
    let format = format.ok_or("report requires --format <csv|punch>")?;
    let mode = if lenient {
        ParseMode::Lenient
    } else {
        ParseMode::Strict
    };

    let contents =
        fs::read_to_string(&input).map_err(|e| format!("reading '{}': {}", input, e))?;

    let outcome = match format.as_str() {
        "csv" => timesheet::parse_csv(&contents, mode)?,
        "punch" => timesheet::parse_punch(&contents, mode)?,
        other => return Err(format!("unknown format '{}' (expected csv or punch)", other)),
    };
    for warning in &outcome.warnings {
        eprintln!("warning: {}", warning);
    }

    let summary = report::summarize(&outcome.entries);
    if json {
        print!("{}", report::render_json(&summary));
    } else {
        print!("{}", report::render_human(&summary));
    }

    Ok(())
}

fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    let value = args
        .get(*i)
        .ok_or_else(|| format!("{} requires a value", flag))?
        .clone();
    *i += 1;
    Ok(value)
}
