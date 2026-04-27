mod cli;
mod crawler;
mod generator;
mod template;

use std::path::Path;

struct AppExit {
    code: i32,
    stdout: Option<String>,
    stderr: Option<String>,
}

fn main() {
    let cli = cli::parse_args();
    let result = run_app(cli, crawler::fetch_problem, generator::generate);

    if let Some(message) = result.stdout {
        println!("{message}");
    }
    if let Some(message) = result.stderr {
        eprintln!("{message}");
    }
    if result.code != 0 {
        std::process::exit(result.code);
    }
}

fn run_app<F, G>(cli: cli::Cli, fetch_problem: F, generate: G) -> AppExit
where
    F: Fn(u32) -> Result<crawler::ProblemData, crawler::CrawlerError>,
    G: Fn(
        u32,
        &crawler::ProblemData,
        cli::Language,
    ) -> Result<generator::GenerateResult, generator::GeneratorError>,
{
    let language = match cli::resolve_language(cli.lang) {
        Ok(language) => language,
        Err(err) => {
            return AppExit {
                code: 1,
                stdout: None,
                stderr: Some(err.to_string()),
            };
        }
    };

    let problem = match fetch_problem(cli.problem_number) {
        Ok(problem) => problem,
        Err(err) => {
            return AppExit {
                code: 1,
                stdout: None,
                stderr: Some(format!("크롤링 실패: {err}")),
            };
        }
    };

    match generate(cli.problem_number, &problem, language) {
        Ok(generator::GenerateResult::Created { directory }) => AppExit {
            code: 0,
            stdout: Some(format!("{} 생성 완료!", format_directory(&directory))),
            stderr: None,
        },
        Ok(generator::GenerateResult::Skipped { directory }) => AppExit {
            code: 0,
            stdout: Some(format!(
                "{} 생성을 건너뜁니다.",
                format_directory(&directory)
            )),
            stderr: None,
        },
        Err(err) => AppExit {
            code: 1,
            stdout: None,
            stderr: Some(format!("파일 생성 실패: {err}")),
        },
    }
}

fn format_directory(path: &Path) -> String {
    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    format!("{label}/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    fn sample_cli() -> cli::Cli {
        cli::Cli {
            problem_number: 11066,
            lang: Some("rust".to_string()),
        }
    }

    fn sample_problem() -> crawler::ProblemData {
        crawler::ProblemData {
            title: "파일 합치기".to_string(),
            sample_input: "2\n4\n40 30 30 50\n".to_string(),
        }
    }

    #[test]
    fn run_app_returns_success_message_when_generation_completes() {
        let result = run_app(
            sample_cli(),
            |problem_number| {
                assert_eq!(problem_number, 11066);
                Ok(sample_problem())
            },
            |problem_number, problem, language| {
                assert_eq!(problem_number, 11066);
                assert_eq!(problem.title, "파일 합치기");
                assert_eq!(language, cli::Language::Rust);
                Ok(generator::GenerateResult::Created {
                    directory: PathBuf::from("11066_파일_합치기"),
                })
            },
        );

        assert_eq!(result.code, 0);
        assert_eq!(
            result.stdout.as_deref(),
            Some("11066_파일_합치기/ 생성 완료!")
        );
        assert_eq!(result.stderr, None);
    }

    #[test]
    fn run_app_returns_crawler_error_message_and_exit_code_one() {
        let result = run_app(
            sample_cli(),
            |_| {
                Err(crawler::CrawlerError::NotFound {
                    problem_number: 11066,
                })
            },
            |_, _, _| unreachable!(),
        );

        assert_eq!(result.code, 1);
        assert_eq!(result.stdout, None);
        assert_eq!(
            result.stderr.as_deref(),
            Some("크롤링 실패: 문제 번호 11066을 찾을 수 없습니다")
        );
    }

    #[test]
    fn run_app_returns_generator_error_message_and_exit_code_one() {
        let result = run_app(
            sample_cli(),
            |_| Ok(sample_problem()),
            |_, _, _| {
                Err(generator::GeneratorError::Io(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "permission denied",
                )))
            },
        );

        assert_eq!(result.code, 1);
        assert_eq!(result.stdout, None);
        assert!(
            result
                .stderr
                .as_deref()
                .unwrap()
                .starts_with("파일 생성 실패: 파일 시스템 오류:")
        );
    }

    #[test]
    fn run_app_returns_skip_message_when_overwrite_is_declined() {
        let result = run_app(
            sample_cli(),
            |_| Ok(sample_problem()),
            |_, _, _| {
                Ok(generator::GenerateResult::Skipped {
                    directory: PathBuf::from("11066_파일_합치기"),
                })
            },
        );

        assert_eq!(result.code, 0);
        assert_eq!(
            result.stdout.as_deref(),
            Some("11066_파일_합치기/ 생성을 건너뜁니다.")
        );
        assert_eq!(result.stderr, None);
    }
}
