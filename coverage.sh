#!/bin/bash

echo "🧹 Cleaning up old coverage files..."
find . -name "*.profraw" -delete 2>/dev/null || true
rm -rf coverage/html 2>/dev/null || true
rm -f coverage/lcov.info 2>/dev/null || true

echo "📁 Ensuring coverage directory exists..."
mkdir -p coverage

echo "🧪 Running tests with coverage instrumentation..."
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='coverage-%p-%m.profraw' cargo test

echo "📊 Processing coverage data..."
# Check if we have profraw files
if ls *.profraw 1> /dev/null 2>&1; then
    echo "✅ Found raw coverage files, processing..."
    
    # Try to use llvm-profdata and llvm-cov if available
    if command -v llvm-profdata >/dev/null 2>&1 && command -v llvm-cov >/dev/null 2>&1; then
        echo "🔧 Using system LLVM tools..."
        llvm-profdata merge -sparse *.profraw -o coverage.profdata
        llvm-cov export \
            --format=lcov \
            --instr-profile=coverage.profdata \
            target/debug/deps/block_traits-* \
            target/debug/deps/example_block-* \
            target/debug/deps/channels-* > coverage/lcov.info
        llvm-cov show \
            --format=html \
            --instr-profile=coverage.profdata \
            target/debug/deps/block_traits-* \
            target/debug/deps/example_block-* \
            target/debug/deps/channels-* \
            --output-dir=coverage/html
        rm -f *.profraw coverage.profdata
    else
        echo "⚠️  LLVM tools not found. Raw files generated but not processed."
        echo "   Install with: rustup component add llvm-tools-preview"
        echo "   Or use cargo-llvm-cov: cargo install cargo-llvm-cov"
        mv *.profraw coverage/ 2>/dev/null || true
    fi
else
    echo "❌ No coverage data generated. Make sure tests ran successfully."
    exit 1
fi

echo "🎯 Coverage generation complete!"
echo ""
echo "📄 Files generated:"
if [ -f coverage/lcov.info ]; then
    echo "   ✅ coverage/lcov.info (for VS Code Coverage Gutters)"
fi
if [ -d coverage/html ]; then
    echo "   ✅ coverage/html/ (for browser viewing)"
fi
if [ -f coverage/*.profraw ]; then
    echo "   📁 coverage/*.profraw (raw data - process with LLVM tools)"
fi
echo ""
echo "💡 To view in VS Code:"
echo "   1. Open Command Palette (Cmd+Shift+P)"
echo "   2. Run 'Coverage Gutters: Display Coverage'"
echo "   3. Toggle with Cmd+Shift+7"
echo ""
if [ -f coverage/html/index.html ]; then
    echo "🌐 To view HTML report: open coverage/html/index.html"
fi