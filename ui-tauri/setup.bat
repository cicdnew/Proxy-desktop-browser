@echo off
REM Virtual IP Browser - Quick Setup Script for Windows
REM This script sets up and runs the Virtual IP Browser application

echo.
echo Virtual IP Browser - Quick Setup
echo ====================================
echo.

REM Check if we're in the right directory
if not exist "package.json" (
    echo Error: This script must be run from the ui-tauri directory
    echo    Please run: cd ui-tauri
    echo    Then run: setup.bat
    exit /b 1
)

REM Check for Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Node.js is not installed
    echo    Please install Node.js from: https://nodejs.org/
    exit /b 1
)

for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo Node.js %NODE_VERSION% found

REM Check for npm
where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: npm is not installed
    exit /b 1
)

for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo npm %NPM_VERSION% found

REM Check for Rust/Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust is not installed
    echo    Please install Rust from: https://rustup.rs/
    exit /b 1
)

for /f "tokens=2" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo Cargo %CARGO_VERSION% found

REM Install dependencies
echo.
echo Installing dependencies...
if not exist "node_modules" (
    npm install --legacy-peer-deps
) else (
    echo    Dependencies already installed
)

REM Build the frontend
echo.
echo Building frontend...
npm run build

REM Check if build was successful
if not exist "dist" (
    echo Error: Build failed - dist directory not created
    exit /b 1
)

echo Frontend build successful

REM Run the application
echo.
echo Starting Virtual IP Browser...
echo (This may take a moment on first run to compile the Rust backend)
echo.

npm run tauri dev
