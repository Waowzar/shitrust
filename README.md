# ShitRust

<div align="center">
  <img src="assets/icon.svg" alt="ShitRust Logo" width="200" />
</div>

ShitRust is a powerful and flexible programming language that combines the best features from Rust, Python, and C/C++. It's designed to offer memory safety, readability, and performance while providing a modern, ergonomic syntax.

## Key Features

- **Memory Safety**: Inspired by Rust's ownership model to prevent memory leaks and race conditions
- **Type System**: Strong static typing with powerful type inference
- **Performance**: LLVM-based optimizing compiler for high-performance executables
- **Concurrency**: Built-in thread support, mutexes, condition variables, and thread-safe primitives
- **Async Runtime**: First-class async/await support with futures and non-blocking I/O
- **Cryptography**: Comprehensive crypto primitives for secure applications
- **Modern Syntax**: Clean, expressive syntax that combines Rust, Python, and C++ styles
- **Error Handling**: Robust error handling with Result types and pattern matching
- **Interactive Development**: Both compiled and interpreted execution modes
- **Tooling**: Formatter, type checker, and comprehensive CLI

## Recent Enhancements

### Type System
- Static type checking with generics and type inference
- User-defined types with struct and enum support
- Type aliases and trait-based polymorphism
- Improved error messages with source location

### Asynchronous Programming
- Complete async/await syntax for non-blocking code
- Future-based task execution system
- Async runtime with task scheduling and cancellation
- Sleep, timeout, and other timing primitives

### Concurrency Features
- Thread creation and management with join semantics
- Mutex, RwLock, and CondVar synchronization primitives
- Thread-local storage and Arc (atomic reference counting)
- Structured concurrency patterns

### Cryptographic Library
- SHA-256 and SHA-512 hashing functions
- HMAC-SHA256 for message authentication
- AES-256-GCM encryption and decryption
- Secure random number generation
- Hex encoding/decoding and Base64 utilities

### Developer Experience
- Rich CLI with compilation, running, and formatting commands
- Strict type checking mode for catching errors early
- Detailed error reporting with color-coded messages
- Performance timing for compilation and execution phases

## Installation

```bash
# Clone the repository
git clone https://github.com/Waowzar/shitrust.git
cd shitrust

# Build the compiler
# Windows
build.bat

# Linux/macOS
./build.sh

# Install the binary (optional)
cargo install --path .
```

## Usage

```bash
# Compile a ShitRust program
shitrust compile examples/hello.sr

# Run a ShitRust program
shitrust run examples/hello.sr

# Run a program with async mode
shitrust run-async examples/async_example.sr

# Type check a program
shitrust check examples/hello.sr

# Format a ShitRust program
shitrust format examples/hello.sr

# Show information about ShitRust
shitrust info

# Show help
shitrust --help
```

### Compiler Options

```bash
# Compile with verbose output
shitrust -v compile examples/hello.sr

# Compile with specific optimization level
shitrust -o aggressive compile examples/hello.sr

# Enable strict type checking
shitrust --strict-types compile examples/hello.sr

# Compile with debug information
shitrust -d compile examples/hello.sr

# Compile with timing information
shitrust -t compile examples/hello.sr

# Compile and emit LLVM IR (creates .ll file)
shitrust --emit-llvm compile examples/hello.sr

# Disable colored output
shitrust --no-color compile examples/hello.sr

# Format a file in-place
shitrust format -i examples/hello.sr
```

## Language Overview

### Basic Syntax

```rust
// Function definition
fn add(a: int, b: int) -> int {
    return a + b;
}

// Variables
let x = 5;          // Type inference
let mut y = 10;     // Mutable variable
let z: float = 3.14; // With type annotation

// Control flow
if x > y {
    println("x is greater");
} else {
    println("y is greater");
}

// Loops
while x > 0 {
    x = x - 1;
}

for i in 0..10 {
    println(i.to_string());
}

// Match statement with pattern matching
match x {
    0 => println("Zero"),
    1 => println("One"),
    n if n > 10 => println("Large number"),
    _ => println("Other"),
}
```

### Advanced Features

#### Structs and Methods

```rust
// Struct definition with methods
struct Point<T> {
    x: T,
    y: T,
    
    // Constructor
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
    
    // Method with self reference
    pub fn distance_from_origin(this) -> float {
        return ((this.x * this.x + this.y * this.y) as float).sqrt();
    }
}

// Creating and using a struct
let p = Point::new(3.0, 4.0);
println("Distance: " + p.distance_from_origin().to_string());
```

#### Traits and Polymorphism

```rust
// Trait definition
trait Printable {
    fn to_string(this) -> string;
    fn print(this) -> void {
        println(this.to_string());
    }
}

// Implementing a trait
impl Printable for Point<float> {
    fn to_string(this) -> string {
        return "Point(" + this.x.to_string() + ", " + this.y.to_string() + ")";
    }
}

// Using trait methods
let p = Point::new(3.0, 4.0);
p.print();  // Uses the trait implementation
```

#### Enums and Pattern Matching

```rust
// Result type for error handling
enum Result<T, E> {
    Ok(T),
    Err(E),
    
    pub fn unwrap(this) -> T {
        match this {
            Result::Ok(value) => value,
            Result::Err(err) => panic("Called unwrap on an Err: " + err.to_string())
        }
    }
}

// Function returning a Result
fn divide(a: int, b: int) -> Result<float, string> {
    if b == 0 {
        return Result::Err("Division by zero");
    }
    return Result::Ok(a as float / b as float);
}

// Pattern matching with Result
match divide(10, 2) {
    Result::Ok(value) => println("Result: " + value.to_string()),
    Result::Err(msg) => println("Error: " + msg)
}
```

#### Asynchronous Programming

```rust
// Async function
async fn fetch_data(url: string) -> string {
    // Simulate network delay
    await sleep(1000);
    return "Data from " + url;
}

// Using async/await
fn main() -> void {
    let runtime = AsyncRuntime::new();
    let data = runtime.block_on(fetch_data("https://example.com"));
    println(data);
}
```

#### Concurrency

```rust
// Using threads and shared state
fn main() -> void {
    // Create a shared counter with a mutex
    let counter = Arc::new(Mutex::new(0));
    
    // Clone the counter for the thread
    let thread_counter = counter.clone();
    
    // Spawn a thread
    let handle = spawn(fn() -> void {
        for i in 0..5 {
            let mut count = thread_counter.lock();
            *count += 1;
            println("Thread: " + count.to_string());
            sleep(100);
        }
    });
    
    // Wait for the thread to complete
    handle.join();
    
    // Show final counter value
    println("Final value: " + counter.lock().to_string());
}
```

#### Cryptography

```rust
// Using the crypto module
fn main() -> void {
    // Generate a key and nonce
    let key = generate_aes_key();
    let nonce = generate_nonce();
    
    // Original message
    let message = "Hello, encrypted world!";
    println("Original: " + message);
    
    // Compute SHA-256 hash
    let hash = sha256(message);
    println("SHA-256: " + hash);
    
    // Encrypt the message
    let encrypted = encrypt(key, nonce, message);
    println("Encrypted: " + encrypted);
    
    // Decrypt the message
    let decrypted = decrypt(key, nonce, encrypted);
    println("Decrypted: " + decrypted);
}
```

#### Pipeline Operator

```rust
// Using the pipeline operator for cleaner data transformation
fn main() -> void {
    let result = "  hello world  "
        |> str_trim           // Trim whitespace
        |> str_uppercase      // Convert to uppercase
        |> (s => s + "!")     // Append exclamation mark
        |> (s => s.len());    // Get length
        
    println("Result: " + result.to_string());
}
```

## Examples

Check the examples directory for sample ShitRust programs:

- `hello.sr` - A simple "Hello World" program
- `features.sr` - Demonstrates basic language features
- `advanced_features.sr` - Comprehensive example of advanced features
- `async_example.sr` - Demonstrates async/await functionality
- `crypto_demo.sr` - Showcases cryptographic operations
- `concurrent.sr` - Examples of thread-based concurrency

## Standard Library

ShitRust comes with a comprehensive standard library:

- `stdlib::io` - Input/output operations
- `stdlib::collections` - Data structures like HashMap, Queue
- `stdlib::time` - Time-related functions and types
- `stdlib::string` - String manipulation utilities
- `stdlib::math` - Mathematical functions
- `stdlib::fs` - File system operations
- `stdlib::net` - Networking functionality
- `stdlib::async_runtime` - Asynchronous programming support
- `stdlib::crypto` - Cryptographic functions
- `stdlib::concurrent` - Concurrency primitives

## Project Structure

- `src/` - Source code for the ShitRust compiler and interpreter
- `examples/` - Example ShitRust programs
- `tests/` - Test suite
- `assets/` - Logo and other assets
- `docs/` - Documentation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
