use crate::error::ShitRustError;
use crate::interpreter::{Value, NativeFunctionSignature};
use std::io::{self, Write, Read, BufRead, BufReader, SeekFrom, Seek};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::collections::HashMap;

/// Standard library for IO operations
pub fn init_io_module() -> Vec<(String, Value)> {
    vec![
        // Print to stdout without newline
        (
            "print".to_string(),
            Value::NativeFunction {
                name: "print".to_string(),
                func: Box::new(|args| {
                    if args.is_empty() {
                        return Err(ShitRustError::RuntimeError("print requires at least one argument".to_string()));
                    }
                    
                    for arg in args {
                        print!("{}", arg);
                        io::stdout().flush().unwrap();
                    }
                    
                    Ok(Value::None)
                }),
                arity: -1, // variadic
            }
        ),
        
        // Print to stdout with newline
        (
            "println".to_string(),
            Value::NativeFunction {
                name: "println".to_string(),
                func: Box::new(|args| {
                    if args.is_empty() {
                        println!();
                        return Ok(Value::None);
                    }
                    
                    for arg in args {
                        print!("{}", arg);
                    }
                    println!();
                    
                    Ok(Value::None)
                }),
                arity: -1, // variadic
            }
        ),
        
        // Read a line from stdin
        (
            "input".to_string(),
            Value::NativeFunction {
                name: "input".to_string(),
                func: Box::new(|args| {
                    if !args.is_empty() {
                        // Print prompt if provided
                        for arg in args {
                            print!("{}", arg);
                        }
                        io::stdout().flush().unwrap();
                    }
                    
                    let mut buffer = String::new();
                    match io::stdin().read_line(&mut buffer) {
                        Ok(_) => {
                            // Remove trailing newline
                            if buffer.ends_with('\n') {
                                buffer.pop();
                                if buffer.ends_with('\r') {
                                    buffer.pop();
                                }
                            }
                            Ok(Value::String(buffer))
                        },
                        Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to read input: {}", e))),
                    }
                }),
                arity: -1, // variadic, optional prompt
            }
        ),
        
        // Open a file
        (
            "open".to_string(),
            Value::NativeFunction {
                name: "open".to_string(),
                func: Box::new(|args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err(ShitRustError::RuntimeError("open requires a filename and optional mode".to_string()));
                    }
                    
                    let filename = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Filename must be a string".to_string())),
                    };
                    
                    let mode = if args.len() > 1 {
                        match &args[1] {
                            Value::String(s) => s.clone(),
                            _ => return Err(ShitRustError::TypeError("Mode must be a string".to_string())),
                        }
                    } else {
                        "r".to_string() // default to read mode
                    };
                    
                    // Create a File object with appropriate methods
                    let mut file_obj = HashMap::new();
                    file_obj.insert("__type".to_string(), Value::String("File".to_string()));
                    file_obj.insert("path".to_string(), Value::String(filename.clone()));
                    file_obj.insert("mode".to_string(), Value::String(mode.clone()));
                    
                    // Add method to read entire file as text
                    file_obj.insert("read_text".to_string(), Value::NativeFunction {
                        name: "read_text".to_string(),
                        func: Box::new(move |_| {
                            let mut file = match File::open(&filename) {
                                Ok(f) => f,
                                Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to open file: {}", e))),
                            };
                            
                            let mut contents = String::new();
                            match file.read_to_string(&mut contents) {
                                Ok(_) => Ok(Value::String(contents)),
                                Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to read file: {}", e))),
                            }
                        }),
                        arity: 0,
                    });
                    
                    // Add method to read all lines as a list
                    file_obj.insert("read_lines".to_string(), Value::NativeFunction {
                        name: "read_lines".to_string(),
                        func: Box::new(move |_| {
                            let file = match File::open(&filename) {
                                Ok(f) => f,
                                Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to open file: {}", e))),
                            };
                            
                            let reader = BufReader::new(file);
                            let mut lines = Vec::new();
                            
                            for line in reader.lines() {
                                match line {
                                    Ok(line_str) => lines.push(Value::String(line_str)),
                                    Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to read line: {}", e))),
                                }
                            }
                            
                            Ok(Value::List(lines))
                        }),
                        arity: 0,
                    });
                    
                    // Add method to read binary data
                    file_obj.insert("read_bytes".to_string(), Value::NativeFunction {
                        name: "read_bytes".to_string(),
                        func: Box::new(move |args| {
                            let size = if args.len() > 0 {
                                match &args[0] {
                                    Value::Int(n) => *n as usize,
                                    _ => return Err(ShitRustError::TypeError("Size must be an integer".to_string())),
                                }
                            } else {
                                // Read all bytes if no size specified
                                usize::MAX
                            };
                            
                            let mut file = match File::open(&filename) {
                                Ok(f) => f,
                                Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to open file: {}", e))),
                            };
                            
                            if size == usize::MAX {
                                // Read all bytes
                                let mut bytes = Vec::new();
                                match file.read_to_end(&mut bytes) {
                                    Ok(_) => {
                                        let byte_values: Vec<Value> = bytes.into_iter()
                                            .map(|b| Value::Int(b as i64))
                                            .collect();
                                        Ok(Value::List(byte_values))
                                    },
                                    Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to read bytes: {}", e))),
                                }
                            } else {
                                // Read specified number of bytes
                                let mut bytes = vec![0; size];
                                match file.read_exact(&mut bytes) {
                                    Ok(_) => {
                                        let byte_values: Vec<Value> = bytes.into_iter()
                                            .map(|b| Value::Int(b as i64))
                                            .collect();
                                        Ok(Value::List(byte_values))
                                    },
                                    Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to read bytes: {}", e))),
                                }
                            }
                        }),
                        arity: -1, // 0 or 1 args
                    });
                    
                    // Add method to write text
                    file_obj.insert("write_text".to_string(), Value::NativeFunction {
                        name: "write_text".to_string(),
                        func: Box::new(move |args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("write_text requires one argument".to_string()));
                            }
                            
                            let data = match &args[0] {
                                Value::String(s) => s.clone(),
                                _ => return Err(ShitRustError::TypeError("Data must be a string".to_string())),
                            };
                            
                            let mut options = OpenOptions::new();
                            
                            match mode.as_str() {
                                "w" => {
                                    options.write(true).truncate(true).create(true);
                                },
                                "a" => {
                                    options.write(true).append(true).create(true);
                                },
                                "r+" => {
                                    options.read(true).write(true);
                                },
                                "w+" => {
                                    options.read(true).write(true).truncate(true).create(true);
                                },
                                "a+" => {
                                    options.read(true).write(true).append(true).create(true);
                                },
                                _ => {
                                    return Err(ShitRustError::RuntimeError(format!("Invalid file mode: {}", mode)));
                                }
                            }
                            
                            let mut file = match options.open(&filename) {
                                Ok(f) => f,
                                Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to open file: {}", e))),
                            };
                            
                            match file.write_all(data.as_bytes()) {
                                Ok(_) => Ok(Value::None),
                                Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to write to file: {}", e))),
                            }
                        }),
                        arity: 1,
                    });
                    
                    // Add method to write bytes
                    file_obj.insert("write_bytes".to_string(), Value::NativeFunction {
                        name: "write_bytes".to_string(),
                        func: Box::new(move |args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("write_bytes requires one argument".to_string()));
                            }
                            
                            let bytes = match &args[0] {
                                Value::List(list) => {
                                    let mut byte_array = Vec::new();
                                    for item in list {
                                        match item {
                                            Value::Int(n) => {
                                                if *n < 0 || *n > 255 {
                                                    return Err(ShitRustError::RuntimeError(format!("Byte value out of range: {}", n)));
                                                }
                                                byte_array.push(*n as u8);
                                            },
                                            _ => return Err(ShitRustError::TypeError("Byte list must contain integers".to_string())),
                                        }
                                    }
                                    byte_array
                                },
                                _ => return Err(ShitRustError::TypeError("Expected a list of bytes".to_string())),
                            };
                            
                            let mut options = OpenOptions::new();
                            
                            match mode.as_str() {
                                "w" => {
                                    options.write(true).truncate(true).create(true);
                                },
                                "a" => {
                                    options.write(true).append(true).create(true);
                                },
                                "r+" => {
                                    options.read(true).write(true);
                                },
                                "w+" => {
                                    options.read(true).write(true).truncate(true).create(true);
                                },
                                "a+" => {
                                    options.read(true).write(true).append(true).create(true);
                                },
                                _ => {
                                    return Err(ShitRustError::RuntimeError(format!("Invalid file mode: {}", mode)));
                                }
                            }
                            
                            let mut file = match options.open(&filename) {
                                Ok(f) => f,
                                Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to open file: {}", e))),
                            };
                            
                            match file.write_all(&bytes) {
                                Ok(_) => Ok(Value::None),
                                Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to write bytes to file: {}", e))),
                            }
                        }),
                        arity: 1,
                    });
                    
                    // Add method to close file
                    file_obj.insert("close".to_string(), Value::NativeFunction {
                        name: "close".to_string(),
                        func: Box::new(|_| {
                            // Files are automatically closed when dropped in Rust
                            Ok(Value::None)
                        }),
                        arity: 0,
                    });
                    
                    Ok(Value::Object(file_obj))
                }),
                arity: -1, // 1 or 2 args
            }
        ),
        
        // File system operations
        (
            "exists".to_string(),
            Value::NativeFunction {
                name: "exists".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("exists requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    Ok(Value::Bool(Path::new(&path).exists()))
                }),
                arity: 1,
            }
        ),
        
        (
            "is_file".to_string(),
            Value::NativeFunction {
                name: "is_file".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("is_file requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    Ok(Value::Bool(Path::new(&path).is_file()))
                }),
                arity: 1,
            }
        ),
        
        (
            "is_dir".to_string(),
            Value::NativeFunction {
                name: "is_dir".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("is_dir requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    Ok(Value::Bool(Path::new(&path).is_dir()))
                }),
                arity: 1,
            }
        ),
        
        (
            "create_dir".to_string(),
            Value::NativeFunction {
                name: "create_dir".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("create_dir requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    match fs::create_dir_all(&path) {
                        Ok(_) => Ok(Value::Bool(true)),
                        Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to create directory: {}", e))),
                    }
                }),
                arity: 1,
            }
        ),
        
        (
            "remove_file".to_string(),
            Value::NativeFunction {
                name: "remove_file".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("remove_file requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    match fs::remove_file(&path) {
                        Ok(_) => Ok(Value::Bool(true)),
                        Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to remove file: {}", e))),
                    }
                }),
                arity: 1,
            }
        ),
        
        (
            "remove_dir".to_string(),
            Value::NativeFunction {
                name: "remove_dir".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("remove_dir requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    match fs::remove_dir_all(&path) {
                        Ok(_) => Ok(Value::Bool(true)),
                        Err(e) => Err(ShitRustError::RuntimeError(format!("Failed to remove directory: {}", e))),
                    }
                }),
                arity: 1,
            }
        ),
        
        (
            "list_dir".to_string(),
            Value::NativeFunction {
                name: "list_dir".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("list_dir requires a path argument".to_string()));
                    }
                    
                    let path = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err(ShitRustError::TypeError("Path must be a string".to_string())),
                    };
                    
                    let entries = match fs::read_dir(&path) {
                        Ok(entries) => entries,
                        Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to read directory: {}", e))),
                    };
                    
                    let mut files = Vec::new();
                    
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                if let Ok(path) = entry.path().into_os_string().into_string() {
                                    files.push(Value::String(path));
                                }
                            },
                            Err(e) => return Err(ShitRustError::RuntimeError(format!("Failed to read directory entry: {}", e))),
                        }
                    }
                    
                    Ok(Value::List(files))
                }),
                arity: 1,
            }
        ),
    ]
} 