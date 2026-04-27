mod cli;

fn main() {
    let cli = cli::parse_args();

    match cli::resolve_language(cli.lang) {
        Ok(language) => {
            println!(
                "problem_number={}, language={}",
                cli.problem_number,
                language.as_str()
            );
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
