#!/bin/bash

# Script para testar o Murasaki com conflitos git reais
# Usage: ./test-murasaki_rs.sh

echo "Creating test repository with conflicts..."

# Create test directory
TEST_DIR="/tmp/murasaki_rs-test-repo"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Initialize git repo
git init
git config user.name "Test User"
git config user.email "test@example.com"

# Create initial JavaScript file
cat > app.js << 'EOF'
// Simple application
function greet(name) {
    console.log("Hello, " + name);
}

function calculateSum(a, b) {
    return a + b;
}

const result = calculateSum(5, 10);
greet("World");
EOF

git add app.js
git commit -m "Initial commit with app.js"

# Create a branch and modify
git checkout -b feature-branch

cat > app.js << 'EOF'
// Enhanced application
function greet(name) {
    console.log("Hello, " + name + "!");
    console.log("Welcome to our app");
}

function calculateSum(a, b) {
    const sum = a + b;
    console.log("Sum is: " + sum);
    return sum;
}

const result = calculateSum(5, 10);
greet("User");
EOF

git add app.js
git commit -m "Feature: Enhanced greet function"

# Go back to main and make conflicting changes
git checkout main

cat > app.js << 'EOF'
// Simple application
function greet(name) {
    console.log("Hi there, " + name);
}

function calculateSum(a, b) {
    return a + b;
}

function multiply(x, y) {
    return x * y;
}

const result = calculateSum(5, 10);
greet("World");
EOF

git add app.js
git commit -m "Main: Different greeting and new multiply function"

# Create a Rust file
cat > main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

git add main.rs
git commit -m "Add Rust file"

# Try to merge (this will create conflicts)
echo ""
echo "Creating conflicts..."
git merge feature-branch || true

echo ""
echo "Test repository created at: $TEST_DIR"
echo ""
echo "Now run Murasaki to resolve conflicts:"
echo "   cd $TEST_DIR"
echo "   /Users/igorvieira/Projects/Personal/murasaki_rs/target/release/saki"
echo ""
echo "You should see:"
echo "  - ▎FILES indicator when focused on file list (cyan)"
echo "  - ▎CODE indicator when focused on code view (cyan)"
echo "  - Black text on cyan background for selected file"
echo "  - Syntax highlighting for JavaScript code in conflicts"
echo ""
