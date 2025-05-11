# ShitRust Language Guide

<div align="center">
  <img src="../assets/icon.svg" alt="ShitRust Logo" width="200" />
  <h3>The pragmatic combination of Rust, Python, and C/C++</h3>
</div>

## Table of Contents

- [Introduction](#introduction)
- [Basic Syntax](#basic-syntax)
  - [Comments](#comments)
  - [Variables and Types](#variables-and-types)
  - [Control Flow](#control-flow)
  - [Functions](#functions)
  - [Loops](#loops)
- [Data Structures](#data-structures)
  - [Structs](#structs)
  - [Enums](#enums)
  - [Arrays and Vectors](#arrays-and-vectors)
  - [Maps](#maps)
- [Advanced Features](#advanced-features)
  - [Pattern Matching](#pattern-matching)
  - [Error Handling](#error-handling)
  - [Asynchronous Programming](#asynchronous-programming)
  - [Generics](#generics)
- [Memory Management](#memory-management)
- [Standard Library](#standard-library)
- [Interoperability](#interoperability)
- [Best Practices](#best-practices)

## Introduction

ShitRust is a modern programming language that combines features from Rust, Python, and C/C++. It aims to provide memory safety, performance, and ease of use in a pragmatic package.

Key design principles of ShitRust:

1. **Safety First**: Prevent memory errors and race conditions at compile time
2. **Developer Ergonomics**: Readable syntax and helpful error messages
3. **Performance**: Native code compilation with LLVM optimizations
4. **Pragmatism**: Choose practicality over dogmatic purity

## Basic Syntax

### Comments

```rust
// This is a single-line comment

/*
  This is a
  multi-line comment
*/

/// Documentation comment for functions, structs, etc.
/// Supports markdown formatting
```

### Variables and Types

```rust
// Variables are immutable by default
let x = 5;             // Type inference determines this is an int
let y: float = 3.14;   // Explicit type annotation
let name: string = "ShitRust";  // String type
let active: bool = true;        // Boolean type

// Mutable variables use the 'mut' keyword
let mut counter = 0;
counter = counter + 1;  // This works because counter is mutable

// Constants
const PI: float = 3.14159;
const MAX_USERS: int = 100;

// Basic types
let i: int = 42;        // Integer
let f: float = 2.71;    // Floating-point
let b: bool = false;    // Boolean
let c: char = 'A';      // Character
let s: string = "Hello"; // String

// Type aliases
type UserId = int;
let user_id: UserId = 1001;
```

### Control Flow

```rust
// If expressions
let max = if a > b { a } else { b };

// Multi-branch if
if x < 0 {
    println("Negative");
} else if x == 0 {
    println("Zero");
} else {
    println("Positive");
}

// Match expressions
match value {
    0 => println("Zero"),
    1 => println("One"),
    2 => println("Two"),
    _ => println("Something else"),  // Default case
}

// Match with binding
match point {
    Point{x: 0, y: 0} => println("At origin"),
    Point{x, y: 0} => println("On x-axis at " + x.to_string()),
    Point{x: 0, y} => println("On y-axis at " + y.to_string()),
    Point{x, y} => println("At " + x.to_string() + ", " + y.to_string()),
}
```

### Functions

```rust
// Basic function
fn add(a: int, b: int) -> int {
    return a + b;
}

// Function with implicit return (returns last expression)
fn multiply(a: int, b: int) -> int {
    a * b  // No 'return' keyword needed for the last expression
}

// Function with no return value
fn print_info(name: string, age: int) -> void {
    println("Name: " + name + ", Age: " + age.to_string());
}

// Function with default parameters
fn greet(name: string, greeting: string = "Hello") -> string {
    return greeting + ", " + name + "!";
}

// Function with variadic parameters
fn sum(...numbers: int) -> int {
    let total = 0;
    for n in numbers {
        total = total + n;
    }
    return total;
}

// Using functions
let result = add(5, 3);        // result = 8
let product = multiply(4, 7);  // product = 28
print_info("Alice", 30);       // Prints: Name: Alice, Age: 30
let message = greet("Bob");    // message = "Hello, Bob!"
let total = sum(1, 2, 3, 4);   // total = 10
```

### Loops

```rust
// While loop
let mut i = 0;
while i < 5 {
    println(i.to_string());
    i = i + 1;
}

// For loop with range
for i in 0..5 {
    println(i.to_string());
}

// For loop with step
for i in 0..10..2 {  // 0, 2, 4, 6, 8
    println(i.to_string());
}

// For loop with collection
let names = ["Alice", "Bob", "Charlie"];
for name in names {
    println(name);
}

// Loop with break and continue
let mut j = 0;
while j < 10 {
    j = j + 1;
    if j % 2 == 0 {
        continue;  // Skip even numbers
    }
    if j > 7 {
        break;     // Exit loop when j > 7
    }
    println(j.to_string());  // Prints 1, 3, 5, 7
}

// Infinite loop with break
let mut x = 0;
loop {
    x = x + 1;
    if x > 5 {
        break;
    }
    println(x.to_string());
}
```

## Data Structures

### Structs

```rust
// Basic struct
struct Point {
    x: float,
    y: float,
}

// Creating an instance
let p = Point{x: 1.0, y: 2.0};

// Accessing fields
println("Coordinates: " + p.x.to_string() + ", " + p.y.to_string());

// Struct with methods
struct Rectangle {
    width: float,
    height: float,
    
    // Method using the 'this' keyword to access fields
    fn area() -> float {
        return this.width * this.height;
    }
    
    // Static method (doesn't use 'this')
    fn create_square(size: float) -> Rectangle {
        return Rectangle{width: size, height: size};
    }
}

// Using methods
let rect = Rectangle{width: 10.0, height: 5.0};
let area = rect.area();  // area = 50.0

// Using static method
let square = Rectangle.create_square(8.0);
```

### Enums

```rust
// Basic enum
enum Color {
    Red,
    Green,
    Blue,
}

// Using enum values
let color = Color.Red;

// Pattern matching on enums
match color {
    Color.Red => println("It's red!"),
    Color.Green => println("It's green!"),
    Color.Blue => println("It's blue!"),
}

// Enum with associated values
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Using Result enum
let result = divide(10, 2);  // Returns Result.Ok(5)
let error_result = divide(5, 0);  // Returns Result.Err("Division by zero")

fn divide(a: int, b: int) -> Result<int, string> {
    if b == 0 {
        return Result.Err("Division by zero");
    }
    return Result.Ok(a / b);
}

// Pattern matching with associated values
match result {
    Result.Ok(value) => println("Result: " + value.to_string()),
    Result.Err(message) => println("Error: " + message),
}
```

### Arrays and Vectors

```rust
// Fixed-size arrays
let numbers = [1, 2, 3, 4, 5];
let zeros: [int; 5] = [0; 5];  // Creates [0, 0, 0, 0, 0]

// Accessing elements
let first = numbers[0];  // Index starts at 0

// Arrays have a fixed length
let length = numbers.length;  // length = 5

// Vectors (dynamic arrays)
let mut vec = [1, 2, 3];
vec.push(4);  // Adds an element to the end
vec.pop();    // Removes the last element

// Vector methods
let contains3 = vec.contains(3);  // true
let index = vec.index_of(2);     // 1
vec.insert(1, 10);  // Insert 10 at index 1
vec.remove(0);      // Remove element at index 0

// Iterating through arrays or vectors
for num in numbers {
    println(num.to_string());
}
```

### Maps

```rust
// Creating a map
let mut scores = {"Alice": 10, "Bob": 20, "Charlie": 15};

// Adding entries
scores["Dave"] = 25;

// Accessing values
let alice_score = scores["Alice"];  // 10

// Checking if a key exists
let has_dave = scores.contains_key("Dave");  // true

// Removing entries
scores.remove("Bob");

// Iterating through keys and values
for (name, score) in scores {
    println(name + ": " + score.to_string());
}
```

## Advanced Features

### Pattern Matching

```rust
// Pattern matching with range
match age {
    0..=12 => println("Child"),
    13..=19 => println("Teenager"),
    20..=64 => println("Adult"),
    _ => println("Senior"),
}

// Pattern matching with guard conditions
match number {
    n if n % 2 == 0 => println("Even"),
    n if n % 2 == 1 => println("Odd"),
    _ => println("Not a number"),
}

// Destructuring in pattern matching
struct Person {
    name: string,
    age: int,
}

let person = Person{name: "Alice", age: 30};

match person {
    Person{name: "Alice", _} => println("Found Alice"),
    Person{name: "Bob", age: 20..=30} => println("Found young Bob"),
    Person{name, age} => println("Found " + name + ", age " + age.to_string()),
}
```

### Error Handling

```rust
// Using Result type
fn read_file(path: string) -> Result<string, string> {
    // Implementation...
    if path_exists(path) {
        return Result.Ok("File contents");
    } else {
        return Result.Err("File not found");
    }
}

// Propagating errors with try operator
fn process_file(path: string) -> Result<string, string> {
    let contents = try? read_file(path);
    // If read_file returns Err, it will be returned from this function
    
    // Process the contents...
    return Result.Ok(contents + " processed");
}

// Using try/catch blocks
try {
    let result = read_file("config.txt");
    match result {
        Result.Ok(contents) => process_contents(contents),
        Result.Err(err) => println("Error: " + err),
    }
} catch (err) {
    println("Unexpected error: " + err.to_string());
} finally {
    cleanup();
}
```

### Asynchronous Programming

```rust
// Async function
async fn fetch_data(url: string) -> Result<string, string> {
    // Simulating network request
    await sleep(1000);  // Pause execution for 1 second
    return Result.Ok("Data from " + url);
}

// Using async functions
async fn main() -> void {
    println("Fetching data...");
    let result = await fetch_data("https://example.com");
    
    match result {
        Result.Ok(data) => println("Received: " + data),
        Result.Err(err) => println("Error: " + err),
    }
}

// Parallel execution
async fn process_all(urls: [string]) -> void {
    let futures = [];
    for url in urls {
        futures.push(fetch_data(url));
    }
    
    let results = await Promise.all(futures);
    for result in results {
        // Process each result
    }
}
```

### Generics

```rust
// Generic function
fn first<T>(list: [T]) -> Option<T> {
    if list.length > 0 {
        return Option.Some(list[0]);
    } else {
        return Option.None;
    }
}

// Generic struct
struct Box<T> {
    value: T,
    
    fn get() -> T {
        return this.value;
    }
    
    fn set(new_value: T) -> void {
        this.value = new_value;
    }
}

// Using generics
let int_box = Box{value: 42};
let str_box = Box{value: "Hello"};

let int_val = int_box.get();  // 42
let str_val = str_box.get();  // "Hello"
```

## Memory Management

ShitRust combines manual and automatic memory management for flexibility and safety:

```rust
// Stack allocation (automatic cleanup)
{
    let x = 5;
    let y = 10;
    // x and y are automatically cleaned up when they go out of scope
}

// Heap allocation with ownership
let s = String.from("Hello");  // Allocates on heap
let s2 = s;  // Ownership moves to s2, s is no longer valid

// References (borrowing)
fn print_string(s: &string) -> void {  // Borrows s immutably
    println(s);  // s is not changed or consumed
}

// Mutable references
fn append(s: &mut string, suffix: string) -> void {
    // s can be modified because it's a mutable reference
    s.append(suffix);
}

// Explicit cleanup with defer
fn process_file(path: string) -> void {
    let file = open_file(path);
    defer file.close();  // Will be called when function exits
    
    // Process file...
    if error_condition {
        return;  // file.close() still gets called
    }
    
    // More processing...
} // file.close() called here
```

## Standard Library

ShitRust includes a comprehensive standard library:

- `io`: File and stream I/O operations
- `net`: Networking utilities
- `collections`: Data structures like vectors, maps, sets
- `time`: Date and time utilities
- `math`: Mathematical functions
- `regex`: Regular expression support
- `json`: JSON parsing and serialization
- `crypto`: Cryptographic functions
- `sync`: Synchronization primitives
- `thread`: Multithreading support
- `path`: File path manipulation
- `os`: Operating system specific functionality

Example usage:

```rust
// File IO
let contents = io.read_file("data.txt").expect("Failed to read file");
io.write_file("output.txt", "Hello, World!").expect("Failed to write file");

// Date and time
let now = time.now();
println("Current time: " + now.format("YYYY-MM-DD HH:mm:ss"));

// Regular expressions
let re = regex.compile(r"^\d{3}-\d{2}-\d{4}$");
let is_ssn = re.matches("123-45-6789");  // true

// JSON
let json_str = '{"name": "Alice", "age": 30}';
let data = json.parse(json_str);
println(data["name"]);  // "Alice"
```

## Interoperability

ShitRust can interoperate with C and Rust libraries:

```rust
// C function binding
extern "C" {
    fn printf(format: *char, ...args) -> int;
    fn strlen(s: *char) -> size_t;
}

// Using C functions
fn print_message(msg: string) -> void {
    printf("Message: %s\n", msg.to_c_str());
}

// Calling Rust functions
extern "Rust" {
    fn process_data(data: *u8, len: usize) -> bool;
}

// Export ShitRust function for other languages
#[export]
fn calculate(x: int, y: int) -> int {
    return x * y + x;
}
```

## Best Practices

Here are some recommended practices when writing ShitRust code:

1. **Use strong typing**: Explicitly define types for function parameters and return values.
2. **Prefer immutability**: Use `let` instead of `let mut` when possible.
3. **Handle errors properly**: Use Result and Option types to handle potential failures.
4. **Use meaningful names**: Choose descriptive names for variables, functions, and types.
5. **Format your code**: Use the built-in formatter (`shitrust format`) to maintain consistent style.
6. **Write tests**: Include unit tests for your code to ensure correctness.
7. **Document your code**: Use documentation comments (`///`) to explain functionality.
8. **Use pattern matching**: Prefer pattern matching over complicated if-else chains.
9. **Leverage standard library**: Use built-in functions rather than reinventing common operations.
10. **Follow memory safety practices**: Be mindful of ownership and borrowing rules.

---

This guide provides an introduction to ShitRust programming. For a complete reference, please refer to the official documentation. 