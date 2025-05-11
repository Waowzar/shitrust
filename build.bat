@echo off
REM Build script for ShitRust compiler on Windows
setlocal EnableDelayedExpansion

echo [92m=== ShitRust Compiler Build Script ===[0m

REM Display version information
for /f "tokens=3" %%a in ('findstr /C:"version" Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
    set VERSION=!VERSION:,=!
)
echo [36mShitRust version !VERSION![0m

REM Display help if requested
if "%1"=="help" goto showhelp
if "%1"=="-h" goto showhelp
if "%1"=="--help" goto showhelp

REM Check for arguments
if "%1"=="clean" (
    echo [93mCleaning build artifacts...[0m
    cargo clean
    echo [92mClean complete![0m
    goto end
)

REM Build the compiler
if "%1"=="release" (
    echo [93mBuilding release version...[0m
    set BUILD_TYPE=release
    cargo build --release
    echo [92mBuild complete! Binary at .\target\release\shitrust.exe[0m
) else (
    echo [93mBuilding debug version...[0m
    set BUILD_TYPE=debug
    cargo build
    echo [92mBuild complete! Binary at .\target\debug\shitrust.exe[0m
)

REM Install if requested
if "%1"=="install" (
    echo [93mInstalling ShitRust...[0m
    cargo install --path .
    echo [92mInstallation complete![0m
)

REM Run tests if requested
if "%1"=="test" (
    echo [93mRunning tests...[0m
    cargo test
    echo [92mTests complete![0m
)

REM Create assets directory if requested
if "%1"=="assets" (
    echo [93mCreating assets directory...[0m
    if not exist ".\assets" mkdir assets
    echo [92mAssets directory created![0m
    goto end
)

REM Generate documentation if requested
if "%1"=="docs" (
    echo [93mGenerating documentation...[0m
    cargo doc --no-deps
    echo [92mDocumentation generated at .\target\doc\shitrust\index.html[0m
    goto end
)

REM Run example with timing if requested
if "%1"=="bench" (
    echo [93mRunning benchmark...[0m
    
    if exist ".\target\release\shitrust.exe" (
        set BIN=.\target\release\shitrust.exe
    ) else (
        echo [91mError: Release build not found. Building release version first...[0m
        cargo build --release
        if not exist ".\target\release\shitrust.exe" (
            echo [91mError: Failed to build release version.[0m
            goto end
        )
        set BIN=.\target\release\shitrust.exe
    )
    
    echo [93mRunning benchmark with timing information...[0m
    %BIN% -t run examples\advanced.sr
    
    echo [92mBenchmark complete![0m
    goto end
)

REM Run examples if requested
if "%1"=="examples" (
    echo [93mRunning examples...[0m
    
    if exist ".\target\%BUILD_TYPE%\shitrust.exe" (
        set BIN=.\target\%BUILD_TYPE%\shitrust.exe
    ) else if exist ".\target\debug\shitrust.exe" (
        set BIN=.\target\debug\shitrust.exe
    ) else if exist ".\target\release\shitrust.exe" (
        set BIN=.\target\release\shitrust.exe
    ) else (
        echo [91mError: ShitRust binary not found. Build first.[0m
        goto end
    )
    
    echo [93mRunning hello.sr...[0m
    %BIN% run examples\hello.sr
    
    echo [93mRunning features.sr...[0m
    %BIN% run examples\features.sr
    
    echo [92mExamples complete![0m
    goto end
)

REM Show version and info if requested
if "%1"=="info" (
    echo [93mShowing ShitRust information...[0m
    
    if exist ".\target\%BUILD_TYPE%\shitrust.exe" (
        set BIN=.\target\%BUILD_TYPE%\shitrust.exe
    ) else if exist ".\target\debug\shitrust.exe" (
        set BIN=.\target\debug\shitrust.exe
    ) else if exist ".\target\release\shitrust.exe" (
        set BIN=.\target\release\shitrust.exe
    ) else (
        echo [91mError: ShitRust binary not found. Build first.[0m
        goto end
    )
    
    %BIN% info
    goto end
)

goto end

:showhelp
echo [92mShitRust Build Script Help[0m
echo.
echo [93mUsage: build.bat [command][0m
echo.
echo [96mCommands:[0m
echo   [no command]   Build debug version
echo   release        Build release version
echo   clean          Clean build artifacts
echo   install        Install ShitRust compiler
echo   test           Run tests
echo   examples       Run example programs
echo   bench          Run benchmark with timing information
echo   docs           Generate documentation
echo   assets         Create assets directory
echo   info           Show ShitRust information
echo   help, -h       Show this help message
echo.

:end
echo [92m=== Build Script Complete ===[0m 