#!/usr/bin/env bash

set -euo pipefail

USER_AGENT="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
OUTPUT_FILE=""
HEADER_FILE=""
CLEANUP_OUTPUT="false"

usage() {
    cat <<'EOF'
Usage:
  script/check_boj_response.sh <problem_number> [output_html_path]

Examples:
  script/check_boj_response.sh 11066
  script/check_boj_response.sh 11066 /tmp/11066.html
EOF
}

extract_title() {
    perl -0ne '
        if (m{<([[:alnum:]]+)[^>]*id="problem_title"[^>]*>(.*?)</\1>}s) {
            my $title = $2;
            $title =~ s/<[^>]+>//g;
            $title =~ s/^\s+//;
            $title =~ s/\s+$//;
            print $title;
        }
    ' "$1"
}

extract_sample_input() {
    perl -0ne '
        if (m{<pre[^>]*id="sample-input-1"[^>]*>(.*?)</pre>}s) {
            my $sample = $1;
            $sample =~ s/\r\n/\n/g;
            $sample =~ s/^\n+//;
            $sample =~ s/\n+$//;
            print $sample;
        }
    ' "$1"
}

cleanup_files() {
    if [[ -n "$HEADER_FILE" ]]; then
        rm -f "$HEADER_FILE"
    fi
    if [[ "$CLEANUP_OUTPUT" == "true" && -n "$OUTPUT_FILE" ]]; then
        rm -f "$OUTPUT_FILE"
    fi
}

main() {
    if [[ $# -lt 1 || $# -gt 2 ]]; then
        usage >&2
        exit 1
    fi

    local problem_number="$1"
    local url
    local status_code
    local title
    local sample_input

    if [[ ! "$problem_number" =~ ^[1-9][0-9]*$ ]]; then
        echo "problem_number must be a positive integer: $problem_number" >&2
        exit 1
    fi

    if [[ $# -eq 2 ]]; then
        OUTPUT_FILE="$2"
    else
        OUTPUT_FILE="$(mktemp "/tmp/boj-${problem_number}-XXXXXX")"
        CLEANUP_OUTPUT="true"
    fi

    HEADER_FILE="$(mktemp "/tmp/boj-${problem_number}-headers-XXXXXX")"
    url="https://www.acmicpc.net/problem/${problem_number}"

    trap cleanup_files EXIT

    status_code="$(
        curl -sS -L \
            -A "$USER_AGENT" \
            -D "$HEADER_FILE" \
            -o "$OUTPUT_FILE" \
            -w '%{http_code}' \
            "$url"
    )"

    echo "URL: $url"
    echo "HTTP status: $status_code"
    echo "Saved HTML: $OUTPUT_FILE"

    if [[ ! "$status_code" =~ ^2 ]]; then
        echo "Request failed. Response headers:" >&2
        cat "$HEADER_FILE" >&2
        exit 1
    fi

    title="$(extract_title "$OUTPUT_FILE")"
    sample_input="$(extract_sample_input "$OUTPUT_FILE")"

    if [[ -z "$title" ]]; then
        echo "Failed to extract #problem_title from HTML." >&2
        exit 1
    fi

    if [[ -z "$sample_input" ]]; then
        echo "Failed to extract pre#sample-input-1 from HTML." >&2
        exit 1
    fi

    echo "Extracted title: $title"
    echo "Sample input preview:"
    printf '%s\n' "$sample_input" | sed -n '1,10p'
}

main "$@"
