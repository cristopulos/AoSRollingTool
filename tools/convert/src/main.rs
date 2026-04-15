use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::process;

mod models;

use models::UnitDatabase;

fn print_usage() {
    eprintln!("Usage: convert --input-dir <DIR> --output <FILE>");
    eprintln!("       convert --input-dir <DIR> --dry-run");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input_dir: Option<String> = None;
    let mut output: Option<String> = None;
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input-dir" => {
                i += 1;
                if i < args.len() {
                    input_dir = Some(args[i].clone());
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(args[i].clone());
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Warning: unknown argument '{}'", args[i]);
            }
        }
        i += 1;
    }

    let input_dir = input_dir.unwrap_or_else(|| {
        eprintln!("Error: --input-dir is required");
        print_usage();
        process::exit(1);
    });

    if !dry_run && output.is_none() {
        eprintln!("Error: --output is required (or use --dry-run)");
        print_usage();
        process::exit(1);
    }

    let units = match merge_units_from_dir(&input_dir) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let db = UnitDatabase { units };

    let json = serde_json::to_string_pretty(&db).unwrap_or_else(|e| {
        eprintln!("Error serializing JSON: {}", e);
        process::exit(1);
    });

    if dry_run {
        println!("{}", json);
    } else {
        let output_path = output.unwrap();
        fs::write(&output_path, json).unwrap_or_else(|e| {
            eprintln!("Error writing to {}: {}", output_path, e);
            process::exit(1);
        });
        println!("Wrote {} units to {}", db.units.len(), output_path);
    }
}

#[derive(Debug)]
pub enum ConvertError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    NotADirectory(String),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::IoError(e) => write!(f, "IO error: {}", e),
            ConvertError::JsonError(e) => write!(f, "JSON error: {}", e),
            ConvertError::NotADirectory(s) => write!(f, "{} is not a directory", s),
        }
    }
}

impl Error for ConvertError {}

impl From<io::Error> for ConvertError {
    fn from(e: io::Error) -> Self {
        ConvertError::IoError(e)
    }
}

impl From<serde_json::Error> for ConvertError {
    fn from(e: serde_json::Error) -> Self {
        ConvertError::JsonError(e)
    }
}

fn merge_units_from_dir(input_dir: &str) -> Result<Vec<models::Unit>, ConvertError> {
    let path = Path::new(input_dir);
    let mut all_units = Vec::new();
    let mut id_counts: HashMap<String, usize> = HashMap::new();

    if !path.is_dir() {
        return Err(ConvertError::NotADirectory(input_dir.to_string()));
    }

    let entries = fs::read_dir(path)?;

    let mut json_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();

    json_files.sort_by_key(|e| e.path());

    for entry in json_files {
        let file_path = entry.path();
        let content = fs::read_to_string(&file_path)?;

        let db: UnitDatabase = serde_json::from_str(&content)?;

        for mut unit in db.units {
            let count = id_counts.entry(unit.id.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                let new_id = format!("{}_{}", unit.id, count);
                eprintln!(
                    "Warning: duplicate id '{}', renaming to '{}'",
                    unit.id, new_id
                );
                unit.id = new_id;
            }
            all_units.push(unit);
        }
    }

    Ok(all_units)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn merge_two_files() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.json");
        let f2 = dir.path().join("b.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"u1","name":"Unit 1","faction":"A","save":4,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w2 = fs::File::create(&f2).unwrap();
        write!(
            w2,
            r#"{{"units":[{{"id":"u2","name":"Unit 2","faction":"B","save":3,"ward":5,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].id, "u1");
        assert_eq!(units[1].id, "u2");
    }

    #[test]
    fn duplicate_ids_renamed() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.json");
        let f2 = dir.path().join("b.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"u1","name":"Unit 1","faction":"A","save":4,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w2 = fs::File::create(&f2).unwrap();
        write!(
            w2,
            r#"{{"units":[{{"id":"u1","name":"Unit 1B","faction":"B","save":3,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].id, "u1");
        assert_eq!(units[1].id, "u1_2");
    }

    #[test]
    fn empty_units_array() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("empty.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(w1, r#"{{"units":[]}}"#).unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 0);
    }

    #[test]
    fn invalid_json_returns_error() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("bad.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(w1, r#"{{invalid json"#).unwrap();

        let result = merge_units_from_dir(dir.path().to_str().unwrap());
        assert!(result.is_err());
        match result.unwrap_err() {
            ConvertError::JsonError(_) => {}
            e => panic!("Expected JsonError, got {:?}", e),
        }
    }

    #[test]
    fn invalid_json_missing_closing_brace() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("bad.json");

        fs::write(&f1, r#"{{"units":[{"id":"u1"}}"#).unwrap();

        let result = merge_units_from_dir(dir.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn non_directory_path_returns_error() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("not_a_dir.json");

        let mut f = fs::File::create(&file_path).unwrap();
        write!(f, r#"test"#).unwrap();

        let result = merge_units_from_dir(file_path.to_str().unwrap());
        assert!(result.is_err());
        match result.unwrap_err() {
            ConvertError::NotADirectory(_) => {}
            e => panic!("Expected NotADirectory, got {:?}", e),
        }
    }

    #[test]
    fn empty_directory() {
        let dir = TempDir::new().unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 0);
    }

    #[test]
    fn multiple_duplicate_ids_get_sequential_suffixes() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.json");
        let f2 = dir.path().join("b.json");
        let f3 = dir.path().join("c.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"u1","name":"Unit 1","faction":"A","save":4,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w2 = fs::File::create(&f2).unwrap();
        write!(
            w2,
            r#"{{"units":[{{"id":"u1","name":"Unit 1B","faction":"B","save":3,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w3 = fs::File::create(&f3).unwrap();
        write!(
            w3,
            r#"{{"units":[{{"id":"u1","name":"Unit 1C","faction":"C","save":2,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].id, "u1");
        assert_eq!(units[1].id, "u1_2");
        assert_eq!(units[2].id, "u1_3");
    }

    #[test]
    fn duplicate_ids_in_same_file() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"u1","name":"Unit 1","faction":"A","save":4,"ward":null,"weapons":[]}},{{"id":"u1","name":"Unit 2","faction":"B","save":3,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].id, "u1");
        assert_eq!(units[1].id, "u1_2");
    }

    #[test]
    fn file_without_json_extension_ignored() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.json");
        let f2 = dir.path().join("b.txt");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"u1","name":"Unit 1","faction":"A","save":4,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w2 = fs::File::create(&f2).unwrap();
        write!(
            w2,
            r#"{{"units":[{{"id":"u2","name":"Unit 2","faction":"B","save":3,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].id, "u1");
    }

    #[test]
    fn files_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("z_file.json");
        let f2 = dir.path().join("a_file.json");

        let mut w1 = fs::File::create(&f1).unwrap();
        write!(
            w1,
            r#"{{"units":[{{"id":"z_first","name":"Z","faction":"A","save":4,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let mut w2 = fs::File::create(&f2).unwrap();
        write!(
            w2,
            r#"{{"units":[{{"id":"a_second","name":"A","faction":"B","save":3,"ward":null,"weapons":[]}}]}}"#
        )
        .unwrap();

        let units = merge_units_from_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].id, "a_second"); // a_file.json comes first alphabetically
        assert_eq!(units[1].id, "z_first"); // z_file.json comes second
    }
}
