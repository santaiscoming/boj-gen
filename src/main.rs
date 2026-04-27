mod cli;
mod crawler;
mod template;

fn main() {
    let cli = cli::parse_args();

    match cli::resolve_language(cli.lang) {
        Ok(language) => match crawler::fetch_problem(cli.problem_number) {
            Ok(problem) => {
                println!(
                    "problem_number={}, language={}, title={}",
                    cli.problem_number,
                    language.as_str(),
                    problem.title
                );
            }
            Err(err) => {
                eprintln!("크롤링 실패: {err}");
                std::process::exit(1);
            }
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
