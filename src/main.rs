use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new(".") {
        let path = entry.unwrap().into_path().display().to_string();
        if path.ends_with(".rs") && !path.contains("target/") {
            fix_imports(path);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ImportType {
    Std,
    External,
    Crate,
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    RestOfFile,
    Imports,
    ImportsInABlock(ImportType),
}

fn is_import(line: &str) -> Option<ImportType> {
    if line.starts_with("use ")
        || line.starts_with("pub use ")
        || line.starts_with("pub(crate) use ")
    {
        if line.contains(" std::") {
            Some(ImportType::Std)
        } else if line.contains(" crate::") || line.contains(" self::") {
            Some(ImportType::Crate)
        } else {
            Some(ImportType::External)
        }
    } else {
        None
    }
}

fn fix_imports(path: String) {
    println!("Fixing imports for {}", path);

    let mut output = Vec::new();
    let mut std_imports = Vec::new();
    let mut external_imports = Vec::new();
    let mut crate_imports = Vec::new();
    let mut section = Section::RestOfFile;
    for line in BufReader::new(File::open(&path).unwrap()).lines() {
        let line = line.unwrap();

        if section == Section::RestOfFile && is_import(&line).is_some() {
            section = Section::Imports;
        }

        match section {
            Section::Imports => {
                if let Some(category) = is_import(&line) {
                    if line.contains("{") && !line.contains("}") {
                        section = Section::ImportsInABlock(category);
                    }
                    match category {
                        ImportType::Std => {
                            std_imports.push(line);
                        }
                        ImportType::External => {
                            external_imports.push(line);
                        }
                        ImportType::Crate => {
                            crate_imports.push(line);
                        }
                    }
                } else {
                    section = Section::RestOfFile;

                    // Write the outputs in the canonical order.
                    if !std_imports.is_empty() {
                        output.extend(std_imports.drain(..));
                        output.push("".to_string());
                    }
                    if !external_imports.is_empty() {
                        output.extend(external_imports.drain(..));
                        output.push("".to_string());
                    }
                    if !crate_imports.is_empty() {
                        output.extend(crate_imports.drain(..));
                        // Usually extraneous, run `cargo fmt` afterwards
                        output.push("".to_string());
                    }

                    output.push(line);
                }
            }
            Section::ImportsInABlock(category) => {
                if line.contains("}") {
                    section = Section::Imports;
                }
                match category {
                    ImportType::Std => {
                        std_imports.push(line);
                    }
                    ImportType::External => {
                        external_imports.push(line);
                    }
                    ImportType::Crate => {
                        crate_imports.push(line);
                    }
                }
            }
            Section::RestOfFile => {
                output.push(line);
            }
        }
    }

    if section == Section::Imports {
        // Write the outputs in the canonical order.
        // TODO Dedupe code
        if !std_imports.is_empty() {
            output.extend(std_imports.drain(..));
            output.push("".to_string());
        }
        if !external_imports.is_empty() {
            output.extend(external_imports.drain(..));
            output.push("".to_string());
        }
        if !crate_imports.is_empty() {
            output.extend(crate_imports.drain(..));
            output.push("".to_string());
        }
    }

    let mut f = File::create(path).unwrap();
    for line in output {
        writeln!(f, "{}", line).unwrap();
    }
}
