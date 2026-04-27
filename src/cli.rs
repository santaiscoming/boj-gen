use std::fmt;
use std::io::{self, BufRead, Write};

use clap::Parser;

const MAX_RETRIES: usize = 3;

#[derive(Debug, Parser)]
#[command(name = "boj", about = "백준 문제 템플릿 생성기")]
pub struct Cli {
    /// 문제 번호
    #[arg(value_parser = clap::value_parser!(u32).range(1..))]
    pub problem_number: u32,

    /// 언어 선택 (rust, python, cpp, java, javascript)
    #[arg(short, long)]
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    Cpp,
    Java,
    JavaScript,
}

impl Language {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Cpp => "cpp",
            Self::Java => "java",
            Self::JavaScript => "javascript",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Cpp => "C++",
            Self::Java => "Java",
            Self::JavaScript => "JavaScript",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

pub fn parse_args() -> Cli {
    Cli::parse()
}

pub fn resolve_language(lang: Option<String>) -> Result<Language, CliError> {
    match lang {
        Some(value) => parse_language_arg(&value)
            .ok_or_else(|| CliError::new(format!("지원하지 않는 언어입니다: {value}"))),
        None => interactive_select(),
    }
}

pub fn interactive_select() -> Result<Language, CliError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    interactive_select_with_io(stdin.lock(), stdout.lock())
}

fn interactive_select_with_io<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
) -> Result<Language, CliError> {
    write_language_menu(&mut writer).map_err(io_error)?;

    for attempt in 0..MAX_RETRIES {
        write!(writer, "언어를 선택하세요 (1-5): ").map_err(io_error)?;
        writer.flush().map_err(io_error)?;

        let mut input = String::new();
        let read = reader.read_line(&mut input).map_err(io_error)?;
        if read == 0 {
            return Err(CliError::new("입력을 읽지 못했습니다"));
        }

        if let Some(language) = parse_language_menu_input(&input) {
            return Ok(language);
        }

        if attempt + 1 < MAX_RETRIES {
            writeln!(writer, "지원하지 않는 언어입니다. 다시 입력해주세요.").map_err(io_error)?;
        }
    }

    Err(CliError::new(
        "지원하지 않는 언어 입력이 3회 누적되어 종료합니다",
    ))
}

fn write_language_menu<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "지원 언어 목록:")?;
    for (idx, language) in Language::all().iter().enumerate() {
        writeln!(writer, "  {}. {}", idx + 1, language.display_name())?;
    }
    Ok(())
}

fn parse_language_arg(input: &str) -> Option<Language> {
    match normalize_input(input).as_str() {
        "rust" | "rs" => Some(Language::Rust),
        "python" | "py" => Some(Language::Python),
        "cpp" | "c++" => Some(Language::Cpp),
        "java" => Some(Language::Java),
        "javascript" | "js" => Some(Language::JavaScript),
        _ => None,
    }
}

fn parse_language_menu_input(input: &str) -> Option<Language> {
    match normalize_input(input).as_str() {
        "1" => Some(Language::Rust),
        "2" => Some(Language::Python),
        "3" => Some(Language::Cpp),
        "4" => Some(Language::Java),
        "5" => Some(Language::JavaScript),
        other => parse_language_arg(other),
    }
}

fn normalize_input(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

fn io_error(err: io::Error) -> CliError {
    CliError::new(format!("입출력 오류: {err}"))
}

impl Language {
    fn all() -> [Language; 5] {
        [
            Language::Rust,
            Language::Python,
            Language::Cpp,
            Language::Java,
            Language::JavaScript,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};
    use std::io::Cursor;

    #[test]
    fn parse_problem_number_and_lang_option() {
        let cli = Cli::try_parse_from(["boj", "11066", "--lang", "rust"]).unwrap();

        assert_eq!(cli.problem_number, 11066);
        assert_eq!(cli.lang.as_deref(), Some("rust"));
    }

    #[test]
    fn short_lang_option_is_supported() {
        let cli = Cli::try_parse_from(["boj", "11066", "-l", "python"]).unwrap();

        assert_eq!(cli.lang.as_deref(), Some("python"));
    }

    #[test]
    fn zero_problem_number_is_rejected() {
        let err = Cli::try_parse_from(["boj", "0", "--lang", "rust"]).unwrap_err();

        assert!(err.to_string().contains("1.."));
    }

    #[test]
    fn negative_problem_number_is_rejected() {
        let err = Cli::try_parse_from(["boj", "-1", "--lang", "rust"]).unwrap_err();

        assert!(err.to_string().contains("unexpected argument"));
    }

    #[test]
    fn help_includes_lang_option() {
        let mut command = Cli::command();
        let help = command.render_help().to_string();

        assert!(help.contains("--lang"));
        assert!(help.contains("문제 번호"));
    }

    #[test]
    fn resolve_language_accepts_supported_names_and_aliases() {
        let cases = [
            ("rust", Language::Rust),
            ("rs", Language::Rust),
            ("python", Language::Python),
            ("py", Language::Python),
            ("cpp", Language::Cpp),
            ("c++", Language::Cpp),
            ("java", Language::Java),
            ("javascript", Language::JavaScript),
            ("js", Language::JavaScript),
            ("RuSt", Language::Rust),
        ];

        for (input, expected) in cases {
            let actual = resolve_language(Some(input.to_string())).unwrap();
            assert_eq!(actual, expected, "input={input}");
        }
    }

    #[test]
    fn resolve_language_rejects_unknown_language() {
        let err = resolve_language(Some("go".to_string())).unwrap_err();

        assert_eq!(err.to_string(), "지원하지 않는 언어입니다: go");
    }

    #[test]
    fn interactive_select_accepts_menu_number() {
        let input = Cursor::new("2\n");
        let mut output = Vec::new();

        let actual = interactive_select_with_io(input, &mut output).unwrap();

        assert_eq!(actual, Language::Python);
    }

    #[test]
    fn interactive_select_accepts_language_alias() {
        let input = Cursor::new("JS\n");
        let mut output = Vec::new();

        let actual = interactive_select_with_io(input, &mut output).unwrap();

        assert_eq!(actual, Language::JavaScript);
    }

    #[test]
    fn interactive_select_retries_three_times_then_fails() {
        let input = Cursor::new("go\nruby\nswift\n");
        let mut output = Vec::new();

        let err = interactive_select_with_io(input, &mut output).unwrap_err();
        let rendered = String::from_utf8(output).unwrap();

        assert_eq!(
            err.to_string(),
            "지원하지 않는 언어 입력이 3회 누적되어 종료합니다"
        );
        assert_eq!(rendered.matches("언어를 선택하세요 (1-5): ").count(), 3);
        assert_eq!(
            rendered
                .matches("지원하지 않는 언어입니다. 다시 입력해주세요.")
                .count(),
            2
        );
    }
}
