#!/bin/bash
# Build script for ShitRust compiler on Linux/macOS

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${GREEN}${BOLD}=== ShitRust Compiler Build Script ===${NC}"

# Display version information
VERSION=$(grep -m 1 "version" Cargo.toml | cut -d '"' -f2)
echo -e "${CYAN}ShitRust version ${VERSION}${NC}"

# Display help if requested
if [[ "$1" == "help" || "$1" == "-h" || "$1" == "--help" ]]; then
    echo -e "${GREEN}${BOLD}ShitRust Build Script Help${NC}"
    echo
    echo -e "${YELLOW}Usage: ./build.sh [command]${NC}"
    echo
    echo -e "${CYAN}Commands:${NC}"
    echo "  [no command]   Build debug version"
    echo "  release        Build release version"
    echo "  clean          Clean build artifacts"
    echo "  install        Install ShitRust compiler"
    echo "  test           Run tests"
    echo "  examples       Run example programs"
    echo "  bench          Run benchmark with timing information"
    echo "  docs           Generate documentation"
    echo "  assets         Create assets directory"
    echo "  info           Show ShitRust information"
    echo "  help, -h       Show this help message"
    echo
    exit 0
fi

# Check for arguments
if [ "$1" == "clean" ]; then
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}${BOLD}Clean complete!${NC}"
    exit 0
fi

# Set build type
BUILD_TYPE="debug"

# Build the compiler
if [ "$1" == "release" ]; then
    echo -e "${YELLOW}Building release version...${NC}"
    BUILD_TYPE="release"
    cargo build --release
    echo -e "${GREEN}${BOLD}Build complete! Binary at ./target/release/shitrust${NC}"
else
    echo -e "${YELLOW}Building debug version...${NC}"
    cargo build
    echo -e "${GREEN}${BOLD}Build complete! Binary at ./target/debug/shitrust${NC}"
fi

# Install if requested
if [ "$1" == "install" ]; then
    echo -e "${YELLOW}Installing ShitRust...${NC}"
    cargo install --path .
    echo -e "${GREEN}${BOLD}Installation complete!${NC}"
fi

# Run tests if requested
if [ "$1" == "test" ]; then
    echo -e "${YELLOW}Running tests...${NC}"
    cargo test
    echo -e "${GREEN}${BOLD}Tests complete!${NC}"
    exit 0
fi

# Create assets directory if requested
if [ "$1" == "assets" ]; then
    echo -e "${YELLOW}Creating assets directory...${NC}"
    mkdir -p assets
    echo -e "${GREEN}${BOLD}Assets directory created!${NC}"
    exit 0
fi

# Generate documentation if requested
if [ "$1" == "docs" ]; then
    echo -e "${YELLOW}Generating documentation...${NC}"
    cargo doc --no-deps
    echo -e "${GREEN}${BOLD}Documentation generated at ./target/doc/shitrust/index.html${NC}"
    
    # Open docs in browser if supported
    if command -v xdg-open &> /dev/null; then
        echo -e "${YELLOW}Opening documentation in browser...${NC}"
        xdg-open ./target/doc/shitrust/index.html
    elif command -v open &> /dev/null; then
        echo -e "${YELLOW}Opening documentation in browser...${NC}"
        open ./target/doc/shitrust/index.html
    fi
    
    exit 0
fi

# Run benchmark with timing if requested
if [ "$1" == "bench" ]; then
    echo -e "${YELLOW}Running benchmark...${NC}"
    
    if [ -f "./target/release/shitrust" ]; then
        BIN="./target/release/shitrust"
    else
        echo -e "${RED}Error: Release build not found. Building release version first...${NC}"
        cargo build --release
        if [ ! -f "./target/release/shitrust" ]; then
            echo -e "${RED}Error: Failed to build release version.${NC}"
            exit 1
        fi
        BIN="./target/release/shitrust"
    fi
    
    echo -e "${YELLOW}Running benchmark with timing information...${NC}"
    $BIN -t run examples/advanced.sr
    
    echo -e "${GREEN}${BOLD}Benchmark complete!${NC}"
    exit 0
fi

# Run examples if requested
if [ "$1" == "examples" ]; then
    echo -e "${YELLOW}Running examples...${NC}"
    
    if [ -f "./target/$BUILD_TYPE/shitrust" ]; then
        BIN="./target/$BUILD_TYPE/shitrust"
    elif [ -f "./target/debug/shitrust" ]; then
        BIN="./target/debug/shitrust"
    elif [ -f "./target/release/shitrust" ]; then
        BIN="./target/release/shitrust"
    else
        echo -e "${RED}Error: ShitRust binary not found. Build first.${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Running hello.sr...${NC}"
    $BIN run examples/hello.sr
    
    echo -e "${YELLOW}Running features.sr...${NC}"
    $BIN run examples/features.sr
    
    echo -e "${GREEN}${BOLD}Examples complete!${NC}"
    exit 0
fi

# Show version and info if requested
if [ "$1" == "info" ]; then
    echo -e "${YELLOW}Showing ShitRust information...${NC}"
    
    if [ -f "./target/$BUILD_TYPE/shitrust" ]; then
        BIN="./target/$BUILD_TYPE/shitrust"
    elif [ -f "./target/debug/shitrust" ]; then
        BIN="./target/debug/shitrust"
    elif [ -f "./target/release/shitrust" ]; then
        BIN="./target/release/shitrust"
    else
        echo -e "${RED}Error: ShitRust binary not found. Build first.${NC}"
        exit 1
    fi
    
    $BIN info
    exit 0
fi

echo -e "${GREEN}${BOLD}=== Build Script Complete ===${NC}" 