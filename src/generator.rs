use std::fmt;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::cli::Language;
use crate::crawler::ProblemData;
use crate::template;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateResult {
    Created { directory: PathBuf },
    Skipped { directory: PathBuf },
}

#[derive(Debug)]
pub enum GeneratorError {
    Io(io::Error),
    Prompt(io::Error),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "파일 시스템 오류: {err}"),
            Self::Prompt(err) => write!(f, "입력을 읽는 중 오류가 발생했습니다: {err}"),
        }
    }
}

impl std::error::Error for GeneratorError {}

impl From<io::Error> for GeneratorError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub fn sanitize_title(title: &str) -> String {
    title
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn generate(
    problem_number: u32,
    data: &ProblemData,
    language: Language,
) -> Result<GenerateResult, GeneratorError> {
    generate_in_dir_with_confirm(
        Path::new("."),
        problem_number,
        data,
        language,
        confirm_overwrite,
    )
}

fn generate_in_dir_with_confirm<F>(
    base_dir: &Path,
    problem_number: u32,
    data: &ProblemData,
    language: Language,
    confirm: F,
) -> Result<GenerateResult, GeneratorError>
where
    F: Fn(&Path) -> Result<bool, GeneratorError>,
{
    let directory = base_dir.join(format!(
        "{}_{}",
        problem_number,
        sanitize_title(&data.title)
    ));

    if directory.exists() && !confirm(&directory)? {
        return Ok(GenerateResult::Skipped { directory });
    }

    fs::create_dir_all(&directory)?;
    fs::write(directory.join("input.txt"), &data.sample_input)?;

    match language {
        Language::Rust => generate_rust_project(problem_number, &directory)?,
        _ => generate_simple_project(language, &directory)?,
    }

    Ok(GenerateResult::Created { directory })
}

fn generate_rust_project(problem_number: u32, directory: &Path) -> Result<(), GeneratorError> {
    let src_dir = directory.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        directory.join("Cargo.toml"),
        template::get_cargo_toml(problem_number),
    )?;
    fs::write(
        src_dir.join(template::get_filename(Language::Rust)),
        template::get_source_code(Language::Rust),
    )?;
    Ok(())
}

fn generate_simple_project(language: Language, directory: &Path) -> Result<(), GeneratorError> {
    fs::write(
        directory.join(template::get_filename(language)),
        template::get_source_code(language),
    )?;
    Ok(())
}

fn confirm_overwrite(path: &Path) -> Result<bool, GeneratorError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    confirm_overwrite_with_io(stdin.lock(), stdout.lock(), path)
}

fn confirm_overwrite_with_io<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    path: &Path,
) -> Result<bool, GeneratorError> {
    write!(
        writer,
        "{} 폴더가 이미 존재합니다. 덮어쓸까요? (y/N): ",
        path.display()
    )
    .map_err(GeneratorError::Prompt)?;
    writer.flush().map_err(GeneratorError::Prompt)?;

    let mut input = String::new();
    let read = reader
        .read_line(&mut input)
        .map_err(GeneratorError::Prompt)?;
    if read == 0 {
        return Ok(false);
    }

    let answer = input.trim().to_ascii_lowercase();
    Ok(matches!(answer.as_str(), "y" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_problem_data() -> ProblemData {
        ProblemData {
            title: "파일 합치기".to_string(),
            sample_input: "2\n4\n40 30 30 50\n".to_string(),
        }
    }

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let unique = format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn sanitize_title_replaces_spaces_with_underscores() {
        assert_eq!(sanitize_title("파일 합치기"), "파일_합치기");
        assert_eq!(sanitize_title("  Long   Title Name "), "Long_Title_Name");
    }

    #[test]
    fn generate_creates_expected_file_layout_for_simple_language() {
        let cases = [
            (Language::Python, "main.py"),
            (Language::Cpp, "main.cpp"),
            (Language::Java, "Main.java"),
            (Language::JavaScript, "main.js"),
        ];

        for (language, filename) in cases {
            let temp_dir = make_temp_dir("boj_gen_generator_simple");
            let data = sample_problem_data();

            let result =
                generate_in_dir_with_confirm(&temp_dir, 11066, &data, language, |_| Ok(true))
                    .unwrap();

            let directory = temp_dir.join("11066_파일_합치기");
            assert_eq!(
                result,
                GenerateResult::Created {
                    directory: directory.clone()
                }
            );
            assert!(directory.join("input.txt").exists());
            assert!(directory.join(filename).exists(), "language={language:?}");
            assert!(!directory.join("Cargo.toml").exists());
            assert_eq!(
                fs::read_to_string(directory.join("input.txt")).unwrap(),
                data.sample_input
            );

            fs::remove_dir_all(temp_dir).unwrap();
        }
    }

    #[test]
    fn generate_creates_rust_project_structure() {
        let temp_dir = make_temp_dir("boj_gen_generator_rust");
        let data = sample_problem_data();

        generate_in_dir_with_confirm(&temp_dir, 11066, &data, Language::Rust, |_| Ok(true))
            .unwrap();

        let directory = temp_dir.join("11066_파일_합치기");
        let cargo_toml = fs::read_to_string(directory.join("Cargo.toml")).unwrap();
        let main_rs = fs::read_to_string(directory.join("src/main.rs")).unwrap();

        assert!(directory.join("input.txt").exists());
        assert!(cargo_toml.contains(r#"name = "boj-11066""#));
        assert!(cargo_toml.contains(r#"edition = "2021""#));
        assert!(main_rs.contains("let mut tokens = read!();"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn generate_skips_when_directory_exists_and_user_declines_overwrite() {
        let temp_dir = make_temp_dir("boj_gen_generator_skip");
        let data = sample_problem_data();
        let directory = temp_dir.join("11066_파일_합치기");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("main.py"), "keep me").unwrap();

        let result =
            generate_in_dir_with_confirm(&temp_dir, 11066, &data, Language::Python, |_| Ok(false))
                .unwrap();

        assert_eq!(
            result,
            GenerateResult::Skipped {
                directory: directory.clone()
            }
        );
        assert_eq!(
            fs::read_to_string(directory.join("main.py")).unwrap(),
            "keep me"
        );

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn generate_overwrites_managed_files_when_user_confirms() {
        let temp_dir = make_temp_dir("boj_gen_generator_overwrite");
        let data = sample_problem_data();
        let directory = temp_dir.join("11066_파일_합치기");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("input.txt"), "old input").unwrap();
        fs::write(directory.join("main.py"), "old code").unwrap();

        let result =
            generate_in_dir_with_confirm(&temp_dir, 11066, &data, Language::Python, |_| Ok(true))
                .unwrap();

        assert_eq!(
            result,
            GenerateResult::Created {
                directory: directory.clone()
            }
        );
        assert_eq!(
            fs::read_to_string(directory.join("input.txt")).unwrap(),
            data.sample_input
        );
        assert_eq!(
            fs::read_to_string(directory.join("main.py")).unwrap(),
            template::get_source_code(Language::Python)
        );

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn generated_rust_project_builds_successfully() {
        let temp_dir = make_temp_dir("boj_gen_generator_build");
        let data = sample_problem_data();

        generate_in_dir_with_confirm(&temp_dir, 11066, &data, Language::Rust, |_| Ok(true))
            .unwrap();

        let directory = temp_dir.join("11066_파일_합치기");
        let target_dir = temp_dir.join("cargo-target");
        let output = Command::new("cargo")
            .arg("build")
            .arg("--quiet")
            .current_dir(&directory)
            .env("CARGO_TARGET_DIR", &target_dir)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn confirm_overwrite_accepts_yes_variants() {
        let mut output = Vec::new();

        let accepted =
            confirm_overwrite_with_io("YeS\n".as_bytes(), &mut output, Path::new("sample"))
                .unwrap();

        assert!(accepted);
        assert!(String::from_utf8(output).unwrap().contains("덮어쓸까요?"));
    }
}
