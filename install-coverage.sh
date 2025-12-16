#!/bin/bash

echo "🔧 Installing llvm-tools-preview component..."
rustup component add llvm-tools-preview

echo "🔧 Installing cargo-llvm-cov..."
cargo install cargo-llvm-cov

echo "📁 Creating coverage directory..."
mkdir -p coverage

echo "🧪 Running tests and generating coverage..."
cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info

echo "🌐 Generating HTML coverage report..."
cargo llvm-cov --workspace --html --output-dir coverage/html

echo "✅ Coverage generated!"
echo ""
echo "📊 To view coverage in VS Code:"
echo "1. Install 'Coverage Gutters' extension if you haven't already"
echo "2. Open Command Palette (Ctrl/Cmd+Shift+P)"
echo "3. Run 'Coverage Gutters: Display Coverage'"
echo "4. Toggle with Ctrl/Cmd+Shift+7"
echo ""
echo "🌐 To view HTML report in browser:"
echo "   open coverage/html/index.html"
echo ""
echo "📄 Coverage files generated:"
echo "   - coverage/lcov.info (for VS Code)"
echo "   - coverage/html/ (for browser)"