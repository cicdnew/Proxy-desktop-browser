#!/bin/bash

# Virtual IP Browser - Quick Setup Script
# This script sets up and runs the Virtual IP Browser application

set -e

echo "🌐 Virtual IP Browser - Quick Setup"
echo "===================================="
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: This script must be run from the ui-tauri directory"
    echo "   Please run: cd ui-tauri && ./setup.sh"
    exit 1
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js is not installed"
    echo "   Please install Node.js from: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js $(node --version) found"

# Check for npm
if ! command -v npm &> /dev/null; then
    echo "❌ Error: npm is not installed"
    exit 1
fi

echo "✅ npm $(npm --version) found"

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust is not installed"
    echo "   Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "✅ Cargo $(cargo --version | cut -d' ' -f2) found"

# Install dependencies
echo ""
echo "📦 Installing dependencies..."
if [ ! -d "node_modules" ]; then
    npm install --legacy-peer-deps
else
    echo "   Dependencies already installed"
fi

# Build the frontend
echo ""
echo "🔨 Building frontend..."
npm run build

# Check if build was successful
if [ ! -d "dist" ]; then
    echo "❌ Error: Build failed - dist directory not created"
    exit 1
fi

echo "✅ Frontend build successful"

# Run the application
echo ""
echo "🚀 Starting Virtual IP Browser..."
echo "   (This may take a moment on first run to compile the Rust backend)"
echo ""

npm run tauri dev
