use crate::cli::Language;

const RUST_SOURCE: &str = r#"use std::{
    fs::File,
    io::{self, Read},
};

fn main() {
    let mut tokens = read!();
    // TODO: 풀이 작성
}

#[macro_export]
macro_rules! read {
    () => {{
        let mut buf = String::new();
        match File::open("input.txt") {
            Ok(mut f) => f.read_to_string(&mut buf).unwrap(),
            Err(_) => io::stdin().read_to_string(&mut buf).unwrap(),
        };
        Box::leak(buf.into_boxed_str()).split_ascii_whitespace()
    }};
}

#[macro_export]
macro_rules! next {
    ($tokens:expr) => { $tokens.next().unwrap() };
    ($tokens:expr, $($t:ty),+) => {
        ($($tokens.next().unwrap().parse::<$t>().unwrap()),+)
    };
}
"#;

const PYTHON_SOURCE: &str = r#"import sys
import os

if os.path.exists('input.txt'):
    sys.stdin = open('input.txt', 'r')

def main():
    # TODO: 풀이 작성
    pass

if __name__ == '__main__':
    main()
"#;

const CPP_SOURCE: &str = r#"#include <bits/stdc++.h>
#include <fstream>
using namespace std;

int main() {
    ifstream file("input.txt");
    if (file.is_open()) {
        cin.rdbuf(file.rdbuf());
    }

    // TODO: 풀이 작성

    return 0;
}
"#;

const JAVA_SOURCE: &str = r#"import java.util.*;
import java.io.*;

public class Main {
    public static void main(String[] args) throws Exception {
        File file = new File("input.txt");
        if (file.exists()) {
            System.setIn(new FileInputStream(file));
        }
        BufferedReader br = new BufferedReader(new InputStreamReader(System.in));

        // TODO: 풀이 작성
    }
}
"#;

const JAVASCRIPT_SOURCE: &str = r#"const { readFileSync } = require('fs');

const input = (() => {
    try {
        return readFileSync('input.txt', 'utf-8');
    } catch {
        return readFileSync(0, 'utf-8');
    }
})();

const lines = input.trim().split('\n');
let idx = 0;
const next = () => lines[idx++];

// TODO: 풀이 작성
"#;

pub fn get_source_code(language: Language) -> &'static str {
    match language {
        Language::Rust => RUST_SOURCE,
        Language::Python => PYTHON_SOURCE,
        Language::Cpp => CPP_SOURCE,
        Language::Java => JAVA_SOURCE,
        Language::JavaScript => JAVASCRIPT_SOURCE,
    }
}

pub fn get_filename(language: Language) -> &'static str {
    match language {
        Language::Rust => "main.rs",
        Language::Python => "main.py",
        Language::Cpp => "main.cpp",
        Language::Java => "Main.java",
        Language::JavaScript => "main.js",
    }
}

pub fn get_cargo_toml(problem_number: u32) -> String {
    format!(
        r#"[package]
name = "boj-{problem_number}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_filename_returns_expected_name_for_each_language() {
        let cases = [
            (Language::Rust, "main.rs"),
            (Language::Python, "main.py"),
            (Language::Cpp, "main.cpp"),
            (Language::Java, "Main.java"),
            (Language::JavaScript, "main.js"),
        ];

        for (language, expected) in cases {
            assert_eq!(get_filename(language), expected);
        }
    }

    #[test]
    fn templates_include_input_txt_priority_and_stdin_fallback_pattern() {
        let cases = [
            (
                Language::Rust,
                "input.txt",
                "io::stdin().read_to_string(&mut buf).unwrap()",
            ),
            (
                Language::Python,
                "input.txt",
                "sys.stdin = open('input.txt', 'r')",
            ),
            (Language::Cpp, "input.txt", "cin.rdbuf(file.rdbuf());"),
            (
                Language::Java,
                "input.txt",
                "System.setIn(new FileInputStream(file));",
            ),
            (
                Language::JavaScript,
                "input.txt",
                "readFileSync(0, 'utf-8')",
            ),
        ];

        for (language, input_pattern, fallback_pattern) in cases {
            let source = get_source_code(language);

            assert!(
                source.contains(input_pattern),
                "language={language:?} should reference input.txt"
            );
            assert!(
                source.contains(fallback_pattern),
                "language={language:?} should preserve stdin fallback behavior"
            );
        }
    }

    #[test]
    fn rust_template_contains_expected_main_flow() {
        let source = get_source_code(Language::Rust);

        assert!(source.contains("let mut tokens = read!();"));
        assert!(source.contains("macro_rules! read"));
        assert!(source.contains("macro_rules! next"));
    }

    #[test]
    fn get_cargo_toml_uses_problem_specific_name_and_edition_2021() {
        let cargo_toml = get_cargo_toml(11066);

        assert!(cargo_toml.contains(r#"name = "boj-11066""#));
        assert!(cargo_toml.contains(r#"edition = "2021""#));
        assert!(cargo_toml.contains("[dependencies]"));
    }
}
