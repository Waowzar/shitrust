# ShitRust Language Reference

ShitRust is a programming language that combines features from Rust, Python, and C/C++. It aims to provide memory safety like Rust, simplicity like Python, and performance like C/C++.

## Basic Syntax

### Variables and Types

```sr
// Variable declaration
let name = "World";        // Type inference
let age: int = 30;         // Explicit type
let mut counter = 0;       // Mutable variable
```

### Basic Types

- `int`: Integer numbers
- `float`: Floating point numbers
- `bool`: Boolean values (true/false)
- `string`: Text strings
- `char`: Single character
- `void`: No return value

### Complex Types

- Lists: `[1, 2, 3]`
- Dictionaries: `{"key": "value", "another": 42}`
- Tuples: `(1, "hello", true)`
- Option: `Option<T>` (Some(T) or None)
- Result: `Result<T, E>` (Ok(T) or Err(E))

### Control Flow

#### If Statements

```sr
if condition {
    // code
} else if another_condition {
    // code
} else {
    // code
}
```

#### While Loops

```sr
while condition {
    // code
}
```

#### For Loops

```sr
for item in collection {
    // code
}

for i in range(0, 10) {
    // code
}
```

#### Match Statements (Pattern Matching)

```sr
match value {
    Pattern1 => {
        // code
    },
    Pattern2 => {
        // code
    },
    _ => {
        // default case
    },
}
```

### Functions

```sr
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // function body
    return value;
}

// Function with default parameters
fn greet(name: string = "World") -> string {
    return "Hello, " + name + "!";
}
```

### Lambda Functions (Closures)

```sr
let add = |a: int, b: int| -> int { return a + b; };
let multiply = |a, b| { a * b };  // Type inference
```

## Object-Oriented Features

### Structs

```sr
struct Point {
    x: float,
    y: float,
}

// Creating an instance
let p = Point { x: 5.0, y: 10.0 };
```

### Methods

```sr
struct Rectangle {
    width: float,
    height: float,
    
    // Method definition
    fn area() -> float {
        return self.width * self.height;
    }
    
    // Static method (constructor)
    fn new(w: float, h: float) -> Rectangle {
        return Rectangle { width: w, height: h };
    }
}

// Usage
let rect = Rectangle::new(5.0, 10.0);
let area = rect.area();
```

### Enums

```sr
enum Color {
    Red,
    Green,
    Blue,
    Custom(int, int, int),  // Enum with data
}

// Usage
let color = Color::Red;
let custom = Color::Custom(255, 128, 0);
```

### Traits (Interfaces)

```sr
trait Printable {
    // Required method
    fn to_string() -> string;
    
    // Method with default implementation
    fn print() -> void {
        println(self.to_string());
    }
}

// Implementing a trait
impl Printable for Rectangle {
    fn to_string() -> string {
        return "Rectangle(" + self.width.to_string() + ", " + self.height.to_string() + ")";
    }
}
```

## Advanced Features

### Generics

```sr
// Generic struct
struct Container<T> {
    value: T,
    
    fn get() -> T {
        return self.value;
    }
}

// Generic function
fn first<T>(list: [T]) -> Option<T> {
    if list.is_empty() {
        return None;
    }
    return Some(list[0]);
}
```

### Memory Management

ShitRust uses a ownership system similar to Rust:

```sr
let v = Vector::new();
v.push(1);
let v2 = v;  // Ownership transferred to v2
// v.push(2);  // This would cause a compile error!

// Borrowing
fn process(v: &Vector<int>) {
    // Can read from v but not modify it
}

// Mutable borrowing
fn add_items(v: &mut Vector<int>) {
    v.push(5);  // This is allowed
}
```

### Error Handling

```sr
// Using Result
fn read_file(filename: string) -> Result<string, string> {
    if !file_exists(filename) {
        return Err("File not found");
    }
    // Read file...
    return Ok(contents);
}

// Using ? operator for error propagation
fn process_file(filename: string) -> Result<int, string> {
    let contents = read_file(filename)?;  // Returns early if it's an Err
    let count = count_lines(contents);
    return Ok(count);
}
```

### List Comprehensions

```sr
let numbers = [1, 2, 3, 4, 5];
let squares = [x * x for x in numbers];
let even_squares = [x * x for x in numbers if x % 2 == 0];
```

### String Interpolation

```sr
let name = "World";
let greeting = f"Hello, {name}!";
```

### Concurrency

```sr
// Creating a thread
let handle = thread::spawn(|| {
    // thread code
});

// Joining a thread
handle.join();

// Channels
let (sender, receiver) = channel();
sender.send(42);
let value = receiver.receive();
```

## Modules and Imports

```sr
// Importing from standard library
import { Vector, HashMap } from "std/collections";

// Importing from another module
import { MyStruct } from "my_module";
import * as math from "std/math";

// Exporting
pub fn public_function() -> void {
    // This function can be imported by other modules
}
```

## Comments

```sr
// Single-line comment

/*
   Multi-line
   comment
*/

/// Documentation comment for the following item
fn documented_function() -> void {
    // ...
}
```

## File Extension

ShitRust files use the `.sr` extension.