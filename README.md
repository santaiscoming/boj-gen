# boj-gen

백준 문제 번호만으로 풀이용 디렉터리, 언어별 템플릿, 첫 번째 샘플 입력 파일을 생성하는 Rust CLI입니다.

## Install

```bash
cargo install --path .
```

설치 후 `boj` 명령으로 실행합니다.

## Usage

```bash
boj <problem_number> --lang <language>
boj <problem_number>
```

지원 언어:

- `rust`
- `python`
- `cpp`
- `java`
- `javascript`

예시:

```bash
boj 11066 --lang rust
```

생성 결과:

```text
11066_파일_합치기/
├── Cargo.toml
├── input.txt
└── src/
    └── main.rs
```

`--lang` 없이 실행하면 대화형 메뉴로 언어를 선택합니다.

## Validation

현재 구현 검증에 사용한 대표 명령:

```bash
cargo test
cargo build
cargo run -- 11066 --lang rust
cargo install --path .
```

생성된 Rust 프로젝트는 별도 `cargo build`로 빌드 가능한지 검증합니다.

## Notes

- 샘플 입력은 첫 번째 입력만 `input.txt`로 저장합니다.
- 같은 문제 폴더가 이미 있으면 덮어쓰기 여부를 확인합니다.
- Rust 생성물의 `edition`은 `2021`입니다.
