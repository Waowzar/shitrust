use crate::error::ShitRustError;
use crate::interpreter::{Value, NativeFunctionSignature};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// Standard library for time operations
pub fn init_time_module() -> Vec<(String, Value)> {
    vec![
        // Sleep function (blocks the current thread)
        (
            "sleep".to_string(),
            Value::NativeFunction {
                name: "sleep".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("sleep requires one argument (milliseconds)".to_string()));
                    }
                    
                    let ms = match &args[0] {
                        Value::Int(ms) => *ms as u64,
                        Value::Float(ms) => *ms as u64,
                        _ => return Err(ShitRustError::TypeError("Expected number of milliseconds".to_string())),
                    };
                    
                    thread::sleep(Duration::from_millis(ms));
                    Ok(Value::None)
                }),
                arity: 1,
            }
        ),
        
        // Get current timestamp in milliseconds
        (
            "now".to_string(),
            Value::NativeFunction {
                name: "now".to_string(),
                func: Box::new(|_args| {
                    let now = SystemTime::now();
                    let duration = now.duration_since(UNIX_EPOCH)
                        .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?;
                    
                    Ok(Value::Int(duration.as_millis() as i64))
                }),
                arity: 0,
            }
        ),
        
        // Create a DateTime object
        (
            "DateTime".to_string(),
            create_datetime_constructor(),
        ),
        
        // Measure execution time of a function
        (
            "measure".to_string(),
            Value::NativeFunction {
                name: "measure".to_string(),
                func: Box::new(|args| {
                    if args.len() < 1 {
                        return Err(ShitRustError::RuntimeError("measure requires a function to execute".to_string()));
                    }
                    
                    let func = match &args[0] {
                        Value::Function { .. } => args[0].clone(),
                        Value::NativeFunction { .. } => args[0].clone(),
                        _ => return Err(ShitRustError::TypeError("Expected a function".to_string())),
                    };
                    
                    // Prepare arguments for the function
                    let func_args = if args.len() > 1 {
                        args[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    // Measure execution time
                    let start = Instant::now();
                    
                    // Create a placeholder for the result
                    // In a real implementation, this would call the function with args
                    let result = Value::None;
                    
                    let elapsed = start.elapsed();
                    let elapsed_ms = elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64;
                    
                    // Return the result and the time
                    let mut result_obj = HashMap::new();
                    result_obj.insert("time".to_string(), Value::Int(elapsed_ms as i64));
                    result_obj.insert("result".to_string(), result);
                    
                    Ok(Value::Object(result_obj))
                }),
                arity: -1, // 1 or more args
            }
        ),
        
        // Create a timer for benchmarking
        (
            "Timer".to_string(),
            Value::NativeFunction {
                name: "Timer".to_string(),
                func: Box::new(|_args| {
                    let mut timer_obj = HashMap::new();
                    timer_obj.insert("__type".to_string(), Value::String("Timer".to_string()));
                    timer_obj.insert("__start".to_string(), Value::Int(0));
                    timer_obj.insert("__running".to_string(), Value::Bool(false));
                    
                    // Method to start the timer
                    timer_obj.insert("start".to_string(), Value::NativeFunction {
                        name: "start".to_string(),
                        func: Box::new(|args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("Timer.start requires this argument".to_string()));
                            }
                            
                            let this = &args[0];
                            if let Value::Object(this_obj) = this {
                                let now = SystemTime::now();
                                let duration = now.duration_since(UNIX_EPOCH)
                                    .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?;
                                
                                let mut this_clone = this_obj.clone();
                                this_clone.insert("__start".to_string(), Value::Int(duration.as_millis() as i64));
                                this_clone.insert("__running".to_string(), Value::Bool(true));
                                
                                Ok(Value::Object(this_clone))
                            } else {
                                Err(ShitRustError::TypeError("Expected Timer object".to_string()))
                            }
                        }),
                        arity: 1, // just this
                    });
                    
                    // Method to stop the timer
                    timer_obj.insert("stop".to_string(), Value::NativeFunction {
                        name: "stop".to_string(),
                        func: Box::new(|args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("Timer.stop requires this argument".to_string()));
                            }
                            
                            let this = &args[0];
                            if let Value::Object(this_obj) = this {
                                let is_running = match this_obj.get("__running") {
                                    Some(Value::Bool(running)) => *running,
                                    _ => false,
                                };
                                
                                if !is_running {
                                    return Err(ShitRustError::RuntimeError("Timer not running".to_string()));
                                }
                                
                                let elapsed = match this_obj.get("elapsed") {
                                    Some(Value::NativeFunction { func, .. }) => {
                                        match func(&[Value::Object(this_obj.clone())]) {
                                            Ok(v) => v,
                                            Err(e) => return Err(e),
                                        }
                                    },
                                    _ => return Err(ShitRustError::RuntimeError("Failed to get elapsed time".to_string())),
                                };
                                
                                let mut this_clone = this_obj.clone();
                                this_clone.insert("__running".to_string(), Value::Bool(false));
                                
                                Ok(elapsed)
                            } else {
                                Err(ShitRustError::TypeError("Expected Timer object".to_string()))
                            }
                        }),
                        arity: 1, // just this
                    });
                    
                    // Method to get elapsed time
                    timer_obj.insert("elapsed".to_string(), Value::NativeFunction {
                        name: "elapsed".to_string(),
                        func: Box::new(|args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("Timer.elapsed requires this argument".to_string()));
                            }
                            
                            let this = &args[0];
                            if let Value::Object(this_obj) = this {
                                let start = match this_obj.get("__start") {
                                    Some(Value::Int(start)) => *start,
                                    _ => return Err(ShitRustError::RuntimeError("Timer not started".to_string())),
                                };
                                
                                let now = SystemTime::now();
                                let duration = now.duration_since(UNIX_EPOCH)
                                    .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?;
                                
                                let current = duration.as_millis() as i64;
                                Ok(Value::Int(current - start))
                            } else {
                                Err(ShitRustError::TypeError("Expected Timer object".to_string()))
                            }
                        }),
                        arity: 1, // just this
                    });
                    
                    // Method to reset the timer
                    timer_obj.insert("reset".to_string(), Value::NativeFunction {
                        name: "reset".to_string(),
                        func: Box::new(|args| {
                            if args.len() != 1 {
                                return Err(ShitRustError::RuntimeError("Timer.reset requires this argument".to_string()));
                            }
                            
                            let this = &args[0];
                            if let Value::Object(this_obj) = this {
                                let mut this_clone = this_obj.clone();
                                this_clone.insert("__start".to_string(), Value::Int(0));
                                this_clone.insert("__running".to_string(), Value::Bool(false));
                                
                                Ok(Value::Object(this_clone))
                            } else {
                                Err(ShitRustError::TypeError("Expected Timer object".to_string()))
                            }
                        }),
                        arity: 1, // just this
                    });
                    
                    Ok(Value::Object(timer_obj))
                }),
                arity: 0,
            }
        ),
        
        // Get formatted current time
        (
            "format_current_time".to_string(),
            Value::NativeFunction {
                name: "format_current_time".to_string(),
                func: Box::new(|args| {
                    let format = if args.len() > 0 {
                        match &args[0] {
                            Value::String(fmt) => fmt.clone(),
                            _ => "%Y-%m-%d %H:%M:%S".to_string(), // Default format
                        }
                    } else {
                        "%Y-%m-%d %H:%M:%S".to_string() // Default format
                    };
                    
                    // In a real implementation, this would use chrono or similar
                    // to format the current time according to the format string
                    
                    // For now, just return a placeholder with current timestamp
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?
                        .as_secs();
                    
                    // Format: YYYY-MM-DD HH:MM:SS (simple implementation)
                    let secs = now % 60;
                    let mins = (now / 60) % 60;
                    let hours = (now / 3600) % 24;
                    let days = (now / 86400) % 30 + 1; // Approximate
                    let months = (now / 2592000) % 12 + 1; // Approximate
                    let years = 1970 + (now / 31536000); // Approximate
                    
                    let formatted = format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                        years, months, days, hours, mins, secs);
                    
                    Ok(Value::String(formatted))
                }),
                arity: -1, // 0 or 1 args
            }
        ),
    ]
}

/// Creates a DateTime constructor
fn create_datetime_constructor() -> Value {
    Value::NativeFunction {
        name: "DateTime".to_string(),
        func: Box::new(|args| {
            let timestamp = if args.len() > 0 {
                match &args[0] {
                    Value::Int(ts) => *ts,
                    Value::Float(ts) => *ts as i64,
                    _ => {
                        // If no valid timestamp is provided, use current time
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?
                            .as_millis() as i64
                    }
                }
            } else {
                // If no args, use current time
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| ShitRustError::RuntimeError(format!("Time error: {}", e)))?
                    .as_millis() as i64
            };
            
            // Create DateTime object
            let mut obj = HashMap::new();
            obj.insert("__type".to_string(), Value::String("DateTime".to_string()));
            obj.insert("__timestamp".to_string(), Value::Int(timestamp));
            
            // Method to get year
            obj.insert("year".to_string(), Value::NativeFunction {
                name: "year".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.year requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            // Simple calculation - in real implementation would use chrono
                            let seconds = ts / 1000;
                            let year = 1970 + (seconds / 31536000);
                            Ok(Value::Int(year))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get month (1-12)
            obj.insert("month".to_string(), Value::NativeFunction {
                name: "month".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.month requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            // Simple calculation - in real implementation would use chrono
                            let seconds = ts / 1000;
                            let month = (seconds / 2592000) % 12 + 1;
                            Ok(Value::Int(month))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get day (1-31)
            obj.insert("day".to_string(), Value::NativeFunction {
                name: "day".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.day requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            // Simple calculation - in real implementation would use chrono
                            let seconds = ts / 1000;
                            let day = (seconds / 86400) % 30 + 1; // Approximate
                            Ok(Value::Int(day))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get hour (0-23)
            obj.insert("hour".to_string(), Value::NativeFunction {
                name: "hour".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.hour requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            let seconds = ts / 1000;
                            let hour = (seconds / 3600) % 24;
                            Ok(Value::Int(hour))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get minute (0-59)
            obj.insert("minute".to_string(), Value::NativeFunction {
                name: "minute".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.minute requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            let seconds = ts / 1000;
                            let minute = (seconds / 60) % 60;
                            Ok(Value::Int(minute))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get second (0-59)
            obj.insert("second".to_string(), Value::NativeFunction {
                name: "second".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.second requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            let seconds = ts / 1000;
                            let second = seconds % 60;
                            Ok(Value::Int(second))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to format date to string
            obj.insert("format".to_string(), Value::NativeFunction {
                name: "format".to_string(),
                func: Box::new(|args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err(ShitRustError::RuntimeError("DateTime.format requires this and optional format string".to_string()));
                    }
                    
                    let this = &args[0];
                    let format = if args.len() > 1 {
                        match &args[1] {
                            Value::String(fmt) => fmt.clone(),
                            _ => "%Y-%m-%d %H:%M:%S".to_string(), // Default format
                        }
                    } else {
                        "%Y-%m-%d %H:%M:%S".to_string() // Default format
                    };
                    
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            let seconds = ts / 1000;
                            
                            // Extract components
                            let secs = seconds % 60;
                            let mins = (seconds / 60) % 60;
                            let hours = (seconds / 3600) % 24;
                            let days = (seconds / 86400) % 30 + 1; // Approximate
                            let months = (seconds / 2592000) % 12 + 1; // Approximate
                            let years = 1970 + (seconds / 31536000); // Approximate
                            
                            // In a real implementation, proper parsing of the format string would be done
                            // For simplicity, we'll just return a fixed format
                            let formatted = format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                                years, months, days, hours, mins, secs);
                            
                            Ok(Value::String(formatted))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: -1, // 1 or 2 args
            });
            
            // Method to get timestamp
            obj.insert("timestamp".to_string(), Value::NativeFunction {
                name: "timestamp".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("DateTime.timestamp requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            Ok(Value::Int(*ts))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to add time
            obj.insert("add".to_string(), Value::NativeFunction {
                name: "add".to_string(),
                func: Box::new(|args| {
                    if args.len() != 3 {
                        return Err(ShitRustError::RuntimeError("DateTime.add requires this, amount, and unit arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    let amount = match &args[1] {
                        Value::Int(n) => *n,
                        Value::Float(n) => *n as i64,
                        _ => return Err(ShitRustError::TypeError("Amount must be a number".to_string())),
                    };
                    
                    let unit = match &args[2] {
                        Value::String(u) => u.as_str(),
                        _ => return Err(ShitRustError::TypeError("Unit must be a string".to_string())),
                    };
                    
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Int(ts)) = this_obj.get("__timestamp") {
                            let milliseconds = match unit {
                                "milliseconds" | "ms" => amount,
                                "seconds" | "s" => amount * 1000,
                                "minutes" | "m" => amount * 60 * 1000,
                                "hours" | "h" => amount * 60 * 60 * 1000,
                                "days" | "d" => amount * 24 * 60 * 60 * 1000,
                                _ => return Err(ShitRustError::RuntimeError(format!("Unknown time unit: {}", unit))),
                            };
                            
                            let new_ts = ts + milliseconds;
                            
                            // Create a new DateTime with the new timestamp
                            let mut new_obj = this_obj.clone();
                            new_obj.insert("__timestamp".to_string(), Value::Int(new_ts));
                            
                            Ok(Value::Object(new_obj))
                        } else {
                            Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected DateTime object".to_string()))
                    }
                }),
                arity: 3, // this, amount, unit
            });
            
            Ok(Value::Object(obj))
        }),
        arity: -1, // 0 or 1 args
    }
} 