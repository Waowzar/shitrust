# Getting Started with ShitRust

Welcome to ShitRust! This guide will help you get started with the ShitRust programming language.

## Installation

Currently, ShitRust is in development, but you can build it from source:

```bash
# Clone the repository
git clone https://github.com/your-username/shitrust.git
cd shitrust

# Build the compiler
cargo build --release

# Add ShitRust to your PATH
# For Linux/Mac
export PATH="$PATH:$(pwd)/target/release"
# For Windows
# Add the path to your PATH environment variable
```

## Hello World

Let's start with a simple "Hello, World!" program. Create a file named `hello.sr` with the following content:

```sr
fn main() -> void {
    println("Hello, World!");
}
```

To run this program:

```bash
shitrust run hello.sr
```

Congratulations! You've just written and run your first ShitRust program.

## Variables and Types

ShitRust supports variable declarations with type inference:

```sr
// Variable declaration with type inference
let name = "World";

// Explicit type annotation
let age: int = 30;

// Mutable variables
let mut counter = 0;
counter += 1; // This is allowed because counter is mutable
```

ShitRust has several built-in types:

- `int`: Integer numbers
- `float`: Floating point numbers
- `bool`: Boolean values
- `string`: Text strings
- `char`: Single characters

## Control Flow

### If Statements

```sr
let x = 42;

if x > 100 {
    println("x is greater than 100");
} else if x > 50 {
    println("x is greater than 50 but not greater than 100");
} else {
    println("x is 50 or less");
}
```

### Loops

```sr
// While loop
let mut i = 0;
while i < 5 {
    println("i is " + i.to_string());
    i += 1;
}

// For loop
for j in range(0, 5) {
    println("j is " + j.to_string());
}

// Iterating over collections
let fruits = ["apple", "banana", "cherry"];
for fruit in fruits {
    println("I like " + fruit);
}
```

## Functions

```sr
// Simple function
fn greet(name: string) -> void {
    println("Hello, " + name + "!");
}

// Function with return value
fn add(a: int, b: int) -> int {
    return a + b;
}

// Function with default parameter
fn greet_with_default(name: string = "World") -> void {
    println("Hello, " + name + "!");
}

// Using the functions
greet("Alice");
let sum = add(5, 7);
println("Sum: " + sum.to_string());
greet_with_default(); // Uses default parameter
```

## Collections

### Lists

```sr
// Creating a list
let numbers = [1, 2, 3, 4, 5];

// Accessing elements
let first = numbers[0];

// Adding elements
numbers.push(6);

// List methods
let length = numbers.len();
let contains_three = numbers.contains(3);

// List comprehension
let squares = [x * x for x in numbers];
```

### Dictionaries

```sr
// Creating a dictionary
let user = {
    "name": "Alice",
    "age": 30,
    "is_admin": true
};

// Accessing elements
let name = user["name"];

// Adding elements
user["email"] = "alice@example.com";

// Dictionary methods
let keys = user.keys();
let has_email = user.contains_key("email");
```

## Structs and Methods

```sr
// Defining a struct
struct Rectangle {
    width: float,
    height: float,
    
    // Method
    fn area() -> float {
        return self.width * self.height;
    }
    
    // Static method (constructor)
    fn new(w: float, h: float) -> Rectangle {
        return Rectangle { width: w, height: h };
    }
}

// Creating an instance
let rect1 = Rectangle { width: 10.0, height: 5.0 };
let rect2 = Rectangle::new(20.0, 15.0);

// Calling methods
let area1 = rect1.area();
let area2 = rect2.area();

println("Area 1: " + area1.to_string());
println("Area 2: " + area2.to_string());
```

## Error Handling

ShitRust provides robust error handling through the `Result` type:

```sr
// Function that might fail
fn read_file(filename: string) -> Result<string, string> {
    if filename.is_empty() {
        return Result::Err("Filename cannot be empty");
    }
    
    // Simulating file reading
    if filename == "secret.txt" {
        return Result::Err("Access denied");
    }
    
    return Result::Ok("File contents: Hello from " + filename);
}

// Using pattern matching to handle errors
fn process_file(filename: string) -> void {
    let result = read_file(filename);
    match result {
        Result::Ok(contents) => {
            println("Success! " + contents);
        },
        Result::Err(error) => {
            println("Error: " + error);
        },
    }
}

// Using the ? operator for error propagation
fn process_and_count_words(filename: string) -> Result<int, string> {
    let contents = read_file(filename)?; // Returns early if an error
    let words = contents.split(" ");
    return Result::Ok(words.len());
}

// Example usage
process_file("data.txt");
process_file("");
process_file("secret.txt");

match process_and_count_words("data.txt") {
    Result::Ok(count) => println("Word count: " + count.to_string()),
    Result::Err(error) => println("Counting error: " + error),
}
```

## Next Steps

This tutorial has covered the basics of ShitRust. To learn more:

1. Check out the [Language Reference](language_reference.md) for a complete overview of ShitRust's features.
2. Explore the [Examples](../examples/) directory for more code samples.
3. Read the [Language Grammar](language_grammar.md) if you're interested in the formal specification.

Happy coding with fucking ShitRust!