# BOJ Gen Phase별 구현 실행서

> 작성일: 2026-04-27
> 기준 문서: [PRD](../PRD.md), [RFC-001](../rfc/001-system-architecture.md), [ADR-001](../adr/001-dependency-selection.md)

## 실행 원칙

- [ ] 모든 phase는 첫 작업으로 테스트 코드를 먼저 작성한다.
- [ ] 각 phase는 해당 범위의 `cargo test` 통과 후 다음 phase로 진행한다.
- [ ] 구현은 RFC의 모듈 순서(`cli -> crawler -> template -> generator -> main`)를 따른다.
- [ ] Rust 생성물은 RFC 결정사항대로 `edition = "2021"`을 사용한다.
- [ ] 완료 직전에는 PRD Acceptance Criteria 기준으로 통합 검증한다.

## Phase 1. CLI 파싱과 언어 선택

목표: `boj <문제번호> [--lang <lang>]` 입력을 안정적으로 해석하고, 언어 미지정 시 대화형 선택까지 처리한다.

- [x] `cli.rs` 테스트 코드를 먼저 작성한다.
- [x] `Language` 파싱 테스트를 작성한다.
- [x] `rust`, `rs`, `python`, `py`, `cpp`, `c++`, `java`, `javascript`, `js` 매핑 테스트를 작성한다.
- [x] 잘못된 언어 입력, 잘못된 문제 번호, 재시도 3회 초과 케이스 테스트를 작성한다.
- [x] 대화형 입력 로직이 테스트 가능하도록 순수 함수 또는 입력 주입 구조로 분리한다.
- [x] `clap` 기반 `Cli` 구조체와 `parse_args()`를 구현한다.
- [x] `Language` enum과 `resolve_language()`를 구현한다.
- [x] `interactive_select()`를 구현한다.
- [x] `--help`, `--lang`, 위치 인수 에러 메시지가 PRD 요구사항과 맞는지 확인한다.
- [x] `cargo test cli` 수준으로 CLI 관련 테스트를 통과시킨다.

완료 기준:
- [x] FR-1, FR-2의 P0 요구사항이 충족된다.
- [x] 언어 미지정 시 메뉴가 뜨고, 유효하지 않은 입력은 최대 3회까지만 재시도한다.

## Phase 2. BOJ 크롤러 구현

목표: 문제 번호로 BOJ 페이지를 조회해 제목과 첫 번째 샘플 입력을 안정적으로 추출한다.

- [x] `crawler.rs` 테스트 코드를 먼저 작성한다.
- [x] HTML fixture 기반으로 `#problem_title`과 `pre#sample-input-1` 추출 테스트를 작성한다.
- [x] 404, 403, 네트워크 실패, 파싱 실패에 대한 에러 매핑 테스트를 작성한다.
- [x] 실제 HTTP 호출부와 HTML 파싱부를 분리해 테스트 가능하게 설계한다.
- [x] `ProblemData { title, sample_input }` 구조체를 정의한다.
- [x] `ureq` 3.x 기반 fetch 로직을 구현한다.
- [x] User-Agent 헤더를 설정한다.
- [x] `scraper` 기반 제목/샘플 입력 추출 로직을 구현한다.
- [x] 사용자 친화적 에러 메시지를 반환하도록 에러 분기 로직을 구현한다.
- [x] 실제 문제 번호 `11066` 기준 수동 검증 절차를 문서화하거나 실행한다.
- [x] `cargo test crawler` 수준으로 크롤러 관련 테스트를 통과시킨다.

완료 기준:
- [x] FR-3의 P0 요구사항이 충족된다.
- [x] PRD의 크롤링 수용 기준에 맞춰 제목과 샘플 입력을 읽어올 수 있다.

## Phase 3. 언어별 템플릿 모듈 구현

목표: 지원 언어별 파일명과 템플릿 문자열을 일관되게 제공한다.

- [x] `template.rs` 테스트 코드를 먼저 작성한다.
- [x] 언어별 파일명 반환 테스트를 작성한다.
- [x] 각 템플릿이 `input.txt` 우선, `stdin` fallback 패턴을 포함하는지 테스트를 작성한다.
- [x] Rust `Cargo.toml` 생성 테스트를 작성한다.
- [x] Rust 템플릿의 `src/main.rs` 내용과 `edition = "2021"` 생성 테스트를 작성한다.
- [x] `get_source_code(Language)`를 구현한다.
- [x] `get_filename(Language)`를 구현한다.
- [x] `get_cargo_toml(problem_number)`를 구현한다.
- [x] 템플릿 문자열이 PRD의 제출 가능 조건을 만족하는지 검토한다.
- [x] `cargo test template` 수준으로 템플릿 관련 테스트를 통과시킨다.

완료 기준:
- [x] FR-5의 P0 요구사항이 충족된다.
- [x] Rust, Python, C++, Java, JavaScript 템플릿이 모두 준비된다.

## Phase 4. 파일 생성기 구현

목표: 문제 정보와 언어를 받아 실제 풀이 디렉터리와 파일 구조를 생성한다.

- [x] `generator.rs` 테스트 코드를 먼저 작성한다.
- [x] 제목 정규화(`공백 -> _`) 테스트를 작성한다.
- [x] 언어별 생성 파일 구조 테스트를 작성한다.
- [x] Rust 선택 시 `Cargo.toml`, `src/main.rs`, `input.txt` 생성 테스트를 작성한다.
- [x] 기존 폴더 존재 시 덮어쓰기 확인 로직 테스트를 작성한다.
- [x] 파일 시스템 테스트는 임시 디렉터리 기준으로 격리되게 구성한다.
- [x] `sanitize_title()`을 구현한다.
- [x] `generate(problem_number, data, lang)`를 구현한다.
- [x] Rust 전용 생성 로직과 일반 언어 생성 로직을 분리 구현한다.
- [x] `confirm_overwrite()`를 구현하거나 확인 가능 구조로 분리한다.
- [x] 생성 후 경로/파일 내용이 PRD의 파일 생성 요구사항과 일치하는지 확인한다.
- [x] `cargo test generator` 수준으로 생성기 관련 테스트를 통과시킨다.

완료 기준:
- [x] FR-4의 P0 요구사항이 충족된다.
- [x] `11066_파일_합치기/` 형태의 폴더 구조를 언어별로 생성할 수 있다.

## Phase 5. 엔트리 포인트와 통합 플로우 완성

목표: CLI, 크롤러, 템플릿, 생성기를 연결해 실제 `boj` 명령어 흐름을 완성한다.

- [x] `main.rs` 및 통합 테스트 코드를 먼저 작성한다.
- [x] 성공 플로우 테스트를 작성한다.
- [x] 크롤링 실패 시 종료 코드와 에러 메시지 테스트를 작성한다.
- [x] 생성 실패 시 종료 코드와 에러 메시지 테스트를 작성한다.
- [x] 모듈 조합이 가능하도록 `main.rs` 의존 순서를 RFC대로 반영한다.
- [x] 성공 시 완료 메시지 출력 로직을 구현한다.
- [x] 실패 시 `stderr` 출력과 `process::exit(1)` 처리를 구현한다.
- [x] 필요 시 통합 테스트를 위해 크롤러/생성기 경계를 주입 가능하게 조정한다.
- [x] `cargo test` 전체 통과를 확인한다.
- [x] `cargo build` 성공을 확인한다.

완료 기준:
- [x] RFC의 실행 파이프라인이 실제 코드로 연결된다.
- [x] PRD AC-1, AC-2, AC-3의 핵심 흐름이 재현된다.

## Phase 6. 최종 검증과 배포 준비

목표: 문서 기준 수용 조건을 끝까지 검증하고 설치 가능한 CLI 상태로 마무리한다.

- [x] 최종 검증용 테스트 또는 스모크 테스트 코드를 먼저 작성한다.
- [x] `cargo run -- 11066 --lang rust` 기준 E2E 검증 절차를 정리한다.
- [x] 생성된 Rust 예제 프로젝트에서 `cargo build`가 성공하는지 확인한다.
- [x] `--lang` 없이 실행했을 때 대화형 메뉴 흐름을 수동 검증한다.
- [x] 네트워크 오류, 404, 403 메시지를 수동 또는 자동으로 검증한다.
- [x] `cargo install --path .` 설치 검증을 수행한다.
- [x] README 또는 사용 예시가 현재 구현과 어긋나지 않는지 점검한다.
- [x] 남은 미결정 사항(기존 폴더 처리, 다중 샘플 입력 범위 제외 등)을 명시한다.

완료 기준:
- [x] PRD Acceptance Criteria가 모두 체크 가능 상태가 된다.
- [x] 설치 후 `boj` 명령으로 기본 사용 시나리오를 재현할 수 있다.

## 최종 완료 체크

- [x] `src/cli.rs`
- [x] `src/crawler.rs`
- [x] `src/template.rs`
- [x] `src/generator.rs`
- [x] `src/main.rs`
- [x] 단위 테스트
- [x] 통합 테스트
- [x] 수동 검증
- [x] 설치 검증
