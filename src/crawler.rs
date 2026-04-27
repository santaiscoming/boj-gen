use std::fmt;

use scraper::{Html, Selector};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemData {
    pub title: String,
    pub sample_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrawlerError {
    Network,
    NotFound { problem_number: u32 },
    Forbidden,
    Parse,
    UnexpectedStatus { status_code: u16 },
}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => f.write_str("네트워크 연결을 확인해주세요"),
            Self::NotFound { problem_number } => {
                write!(f, "문제 번호 {problem_number}을 찾을 수 없습니다")
            }
            Self::Forbidden => {
                f.write_str("BOJ 서버가 요청을 거부했습니다. 잠시 후 다시 시도해주세요")
            }
            Self::Parse => f.write_str("페이지 구조가 변경되었을 수 있습니다"),
            Self::UnexpectedStatus { status_code } => {
                write!(f, "예상하지 못한 응답을 받았습니다. status={status_code}")
            }
        }
    }
}

impl std::error::Error for CrawlerError {}

pub fn fetch_problem(problem_number: u32) -> Result<ProblemData, CrawlerError> {
    let body = fetch_problem_html(problem_number)?;
    parse_problem_html(&body)
}

fn fetch_problem_html(problem_number: u32) -> Result<String, CrawlerError> {
    let url = format!("https://www.acmicpc.net/problem/{problem_number}");
    let mut response = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|err| map_transport_error(problem_number, err))?;

    response
        .body_mut()
        .read_to_string()
        .map_err(|_| CrawlerError::Network)
}

fn parse_problem_html(body: &str) -> Result<ProblemData, CrawlerError> {
    let document = Html::parse_document(body);
    let title_selector = Selector::parse("#problem_title").map_err(|_| CrawlerError::Parse)?;
    let sample_selector = Selector::parse("pre#sample-input-1").map_err(|_| CrawlerError::Parse)?;

    let title = document
        .select(&title_selector)
        .next()
        .map(|node| node.text().collect::<String>().trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or(CrawlerError::Parse)?;

    let sample_input = document
        .select(&sample_selector)
        .next()
        .map(|node| node.text().collect::<String>())
        .filter(|value| !value.trim().is_empty())
        .ok_or(CrawlerError::Parse)?;

    Ok(ProblemData {
        title,
        sample_input,
    })
}

fn map_transport_error(problem_number: u32, err: ureq::Error) -> CrawlerError {
    match err {
        ureq::Error::StatusCode(status_code) => map_status_code(problem_number, status_code),
        _ => CrawlerError::Network,
    }
}

fn map_status_code(problem_number: u32, status_code: u16) -> CrawlerError {
    match status_code {
        403 => CrawlerError::Forbidden,
        404 => CrawlerError::NotFound { problem_number },
        _ => CrawlerError::UnexpectedStatus { status_code },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HTML: &str = r#"
        <html>
            <body>
                <h1 id="problem_title">파일 합치기</h1>
                <pre id="sample-input-1">2
4
40 30 30 50
15
1 21 3 4 5 35 22 14 1 8 5 7 2 1 5</pre>
            </body>
        </html>
    "#;

    #[test]
    fn parse_problem_html_extracts_title_and_sample_input() {
        let actual = parse_problem_html(SAMPLE_HTML).unwrap();

        assert_eq!(actual.title, "파일 합치기");
        assert!(actual.sample_input.starts_with("2\n4\n40 30 30 50"));
    }

    #[test]
    fn parse_problem_html_fails_when_title_is_missing() {
        let html = r#"<html><body><pre id="sample-input-1">1 2 3</pre></body></html>"#;

        let err = parse_problem_html(html).unwrap_err();

        assert_eq!(err, CrawlerError::Parse);
    }

    #[test]
    fn parse_problem_html_fails_when_sample_input_is_missing() {
        let html = r#"<html><body><h1 id="problem_title">파일 합치기</h1></body></html>"#;

        let err = parse_problem_html(html).unwrap_err();

        assert_eq!(err, CrawlerError::Parse);
    }

    #[test]
    fn map_status_code_returns_not_found_error() {
        let err = map_status_code(11066, 404);

        assert_eq!(
            err,
            CrawlerError::NotFound {
                problem_number: 11066
            }
        );
    }

    #[test]
    fn map_status_code_returns_forbidden_error() {
        let err = map_status_code(11066, 403);

        assert_eq!(err, CrawlerError::Forbidden);
    }

    #[test]
    fn map_status_code_returns_network_message_for_network_error() {
        let err = CrawlerError::Network;

        assert_eq!(err.to_string(), "네트워크 연결을 확인해주세요");
    }

    #[test]
    fn map_status_code_returns_parse_message_for_parse_error() {
        let err = CrawlerError::Parse;

        assert_eq!(err.to_string(), "페이지 구조가 변경되었을 수 있습니다");
    }
}
