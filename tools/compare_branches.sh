#!/usr/bin/env bash
set -e

BASE_BRANCH="${1:-main}"
CAND_BRANCH="${2:-$(git rev-parse --abbrev-ref HEAD)}"
REPORT_DIR="${3:-report}"
PORT=8080

echo "🔍 Comparing PDF regression between '$BASE_BRANCH' and '$CAND_BRANCH'..."

# Create clean temporary dirs
mkdir -p .pdf-regression/baseline .pdf-regression/candidate

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# 1. Build baseline PDFs
echo "📦 Building baseline PDFs on branch '$BASE_BRANCH'..."
rm -rf tests/outputs
git checkout "$BASE_BRANCH" --quiet
cargo test --quiet
rm -rf .pdf-regression/baseline/*
cp -r tests/outputs/* .pdf-regression/baseline/

# 2. Build candidate PDFs
echo "📦 Building candidate PDFs on branch '$CAND_BRANCH'..."
rm -rf tests/outputs
git checkout "$CAND_BRANCH" --quiet
cargo test --quiet
rm -rf .pdf-regression/candidate/*
cp -r tests/outputs/* .pdf-regression/candidate/

# Restore original branch
git checkout "$CURRENT_BRANCH" --quiet

# 3. Compare with uv
echo "📊 Running visual comparison..."
uv run tools/pdf-regression-report/compare.py \
  --baseline-dir .pdf-regression/baseline \
  --candidate-dir .pdf-regression/candidate \
  --output-dir "$REPORT_DIR" \
  --base-ref "$BASE_BRANCH" \
  --head-ref "$CAND_BRANCH"

echo ""
echo "🚀 Static comparison report generated at: ./$REPORT_DIR"
echo "🌐 Starting local preview server at: http://localhost:$PORT"
echo "Press Ctrl+C to stop the server."
python3 -m http.server "$PORT" -d "$REPORT_DIR"
