use crate::error::ShitRustError;
use crate::interpreter::{Value, NativeFunctionSignature};
use std::collections::{HashMap as RustHashMap, HashSet as RustHashSet};

/// Standard library for collections
pub fn init_collections_module() -> Vec<(String, Value)> {
    vec![
        // HashMap implementation
        (
            "HashMap".to_string(),
            create_hashmap_constructor(),
        ),
        
        // Vector implementation (more advanced than built-in list)
        (
            "Vector".to_string(),
            create_vector_constructor(),
        ),
        
        // Set implementation
        (
            "Set".to_string(),
            create_set_constructor(),
        ),
        
        // Queue implementation
        (
            "Queue".to_string(),
            create_queue_constructor(),
        ),
        
        // Priority Queue implementation
        (
            "PriorityQueue".to_string(),
            create_priority_queue_constructor(),
        ),
    ]
}

/// Creates a HashMap constructor function
fn create_hashmap_constructor() -> Value {
    Value::NativeFunction {
        name: "HashMap".to_string(),
        func: Box::new(|_args| {
            let mut map_obj = RustHashMap::new();
            
            // Create an empty HashMap object
            let mut obj = RustHashMap::new();
            obj.insert("__type".to_string(), Value::String("HashMap".to_string()));
            obj.insert("__data".to_string(), Value::Dict(map_obj));
            
            // Method to set a key-value pair
            obj.insert("set".to_string(), Value::NativeFunction {
                name: "set".to_string(),
                func: Box::new(|args| {
                    if args.len() != 3 {
                        return Err(ShitRustError::RuntimeError("HashMap.set requires this, key, and value arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            let mut map_clone = map.clone();
                            map_clone.insert(args[1].clone(), args[2].clone());
                            
                            // Update the map in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::Dict(map_clone));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: 3, // this, key, value
            });
            
            // Method to get a value by key
            obj.insert("get".to_string(), Value::NativeFunction {
                name: "get".to_string(),
                func: Box::new(|args| {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(ShitRustError::RuntimeError("HashMap.get requires this, key, and optional default value".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            if let Some(value) = map.get(&args[1]) {
                                Ok(value.clone())
                            } else if args.len() == 3 {
                                // Return default value if provided
                                Ok(args[2].clone())
                            } else {
                                Ok(Value::None)
                            }
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: -1, // 2 or 3 args
            });
            
            // Method to check if key exists
            obj.insert("contains_key".to_string(), Value::NativeFunction {
                name: "contains_key".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("HashMap.contains_key requires this and key arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            Ok(Value::Bool(map.contains_key(&args[1])))
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: 2, // this, key
            });
            
            // Method to get all keys
            obj.insert("keys".to_string(), Value::NativeFunction {
                name: "keys".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("HashMap.keys requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            let keys: Vec<Value> = map.keys().cloned().collect();
                            Ok(Value::List(keys))
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get all values
            obj.insert("values".to_string(), Value::NativeFunction {
                name: "values".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("HashMap.values requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            let values: Vec<Value> = map.values().cloned().collect();
                            Ok(Value::List(values))
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get size
            obj.insert("size".to_string(), Value::NativeFunction {
                name: "size".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("HashMap.size requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Dict(map)) = this_obj.get("__data") {
                            Ok(Value::Int(map.len() as i64))
                        } else {
                            Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected HashMap object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            Ok(Value::Object(obj))
        }),
        arity: 0, // no args for constructor
    }
}

/// Creates a Vector constructor function (more feature-rich than built-in lists)
fn create_vector_constructor() -> Value {
    Value::NativeFunction {
        name: "Vector".to_string(),
        func: Box::new(|args| {
            let mut vec_data = Vec::new();
            
            // If args provided, use them as initial values
            if !args.is_empty() {
                if let Value::List(items) = &args[0] {
                    vec_data = items.clone();
                } else {
                    vec_data = args.clone();
                }
            }
            
            // Create Vector object
            let mut obj = RustHashMap::new();
            obj.insert("__type".to_string(), Value::String("Vector".to_string()));
            obj.insert("__data".to_string(), Value::List(vec_data));
            
            // Method to add an item
            obj.insert("push".to_string(), Value::NativeFunction {
                name: "push".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Vector.push requires this and value arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(vec)) = this_obj.get("__data") {
                            let mut vec_clone = vec.clone();
                            vec_clone.push(args[1].clone());
                            
                            // Update the vector in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::List(vec_clone));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                    }
                }),
                arity: 2, // this, value
            });
            
            // Method to pop the last item
            obj.insert("pop".to_string(), Value::NativeFunction {
                name: "pop".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("Vector.pop requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(vec)) = this_obj.get("__data") {
                            let mut vec_clone = vec.clone();
                            
                            if vec_clone.is_empty() {
                                return Err(ShitRustError::RuntimeError("Cannot pop from empty Vector".to_string()));
                            }
                            
                            let item = vec_clone.pop().unwrap();
                            
                            // Update the vector in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::List(vec_clone));
                            
                            Ok(item)
                        } else {
                            Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get item at index
            obj.insert("get".to_string(), Value::NativeFunction {
                name: "get".to_string(),
                func: Box::new(|args| {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(ShitRustError::RuntimeError("Vector.get requires this, index, and optional default value".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(vec)) = this_obj.get("__data") {
                            let index = match &args[1] {
                                Value::Int(i) => *i as usize,
                                _ => return Err(ShitRustError::TypeError("Index must be an integer".to_string())),
                            };
                            
                            if index < vec.len() {
                                Ok(vec[index].clone())
                            } else if args.len() == 3 {
                                // Return default value if provided
                                Ok(args[2].clone())
                            } else {
                                Ok(Value::None)
                            }
                        } else {
                            Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                    }
                }),
                arity: -1, // 2 or 3 args
            });
            
            // Method to get size
            obj.insert("size".to_string(), Value::NativeFunction {
                name: "size".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("Vector.size requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(vec)) = this_obj.get("__data") {
                            Ok(Value::Int(vec.len() as i64))
                        } else {
                            Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Vector object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // More methods would be implemented in a real library
            
            Ok(Value::Object(obj))
        }),
        arity: -1, // 0 or more args
    }
}

/// Creates a Set constructor function with a complete implementation
fn create_set_constructor() -> Value {
    Value::NativeFunction {
        name: "Set".to_string(),
        func: Box::new(|args| {
            let mut set_data = RustHashSet::new();
            
            // If args provided, use them as initial values
            if !args.is_empty() {
                if let Value::List(items) = &args[0] {
                    for item in items {
                        set_data.insert(item.clone());
                    }
                } else {
                    for arg in args {
                        set_data.insert(arg);
                    }
                }
            }
            
            // Create Set object
            let mut obj = RustHashMap::new();
            obj.insert("__type".to_string(), Value::String("Set".to_string()));
            obj.insert("__data".to_string(), Value::Set(set_data));
            
            // Method to add an item
            obj.insert("add".to_string(), Value::NativeFunction {
                name: "add".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.add requires this and value arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(set)) = this_obj.get("__data") {
                            let mut set_clone = set.clone();
                            set_clone.insert(args[1].clone());
                            
                            // Update the set in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::Set(set_clone));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 2, // this, value
            });
            
            // Method to remove an item
            obj.insert("remove".to_string(), Value::NativeFunction {
                name: "remove".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.remove requires this and value arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(set)) = this_obj.get("__data") {
                            let mut set_clone = set.clone();
                            let removed = set_clone.remove(&args[1]);
                            
                            // Update the set in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::Set(set_clone));
                            
                            Ok(Value::Bool(removed))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 2, // this, value
            });
            
            // Method to check if item exists
            obj.insert("contains".to_string(), Value::NativeFunction {
                name: "contains".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.contains requires this and value arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(set)) = this_obj.get("__data") {
                            Ok(Value::Bool(set.contains(&args[1])))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 2, // this, value
            });
            
            // Method to get size
            obj.insert("size".to_string(), Value::NativeFunction {
                name: "size".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("Set.size requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(set)) = this_obj.get("__data") {
                            Ok(Value::Int(set.len() as i64))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to convert to list
            obj.insert("to_list".to_string(), Value::NativeFunction {
                name: "to_list".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("Set.to_list requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(set)) = this_obj.get("__data") {
                            let items: Vec<Value> = set.iter().cloned().collect();
                            Ok(Value::List(items))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to create intersection with another set
            obj.insert("intersection".to_string(), Value::NativeFunction {
                name: "intersection".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.intersection requires this and another Set".to_string()));
                    }
                    
                    let this = &args[0];
                    let other = &args[1];
                    
                    if let (Value::Object(this_obj), Value::Object(other_obj)) = (this, other) {
                        if let (Some(Value::Set(set1)), Some(Value::Set(set2))) = (this_obj.get("__data"), other_obj.get("__data")) {
                            let intersection: RustHashSet<Value> = set1.intersection(set2).cloned().collect();
                            
                            // Create a new Set with the intersection
                            let mut new_obj = RustHashMap::new();
                            new_obj.insert("__type".to_string(), Value::String("Set".to_string()));
                            new_obj.insert("__data".to_string(), Value::Set(intersection));
                            
                            // Copy methods from the original Set
                            for (key, value) in this_obj.iter() {
                                if !key.starts_with("__") {
                                    new_obj.insert(key.clone(), value.clone());
                                }
                            }
                            
                            Ok(Value::Object(new_obj))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                    }
                }),
                arity: 2, // this, other set
            });
            
            // Method to create union with another set
            obj.insert("union".to_string(), Value::NativeFunction {
                name: "union".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.union requires this and another Set".to_string()));
                    }
                    
                    let this = &args[0];
                    let other = &args[1];
                    
                    if let (Value::Object(this_obj), Value::Object(other_obj)) = (this, other) {
                        if let (Some(Value::Set(set1)), Some(Value::Set(set2))) = (this_obj.get("__data"), other_obj.get("__data")) {
                            let union: RustHashSet<Value> = set1.union(set2).cloned().collect();
                            
                            // Create a new Set with the union
                            let mut new_obj = RustHashMap::new();
                            new_obj.insert("__type".to_string(), Value::String("Set".to_string()));
                            new_obj.insert("__data".to_string(), Value::Set(union));
                            
                            // Copy methods from the original Set
                            for (key, value) in this_obj.iter() {
                                if !key.starts_with("__") {
                                    new_obj.insert(key.clone(), value.clone());
                                }
                            }
                            
                            Ok(Value::Object(new_obj))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                    }
                }),
                arity: 2, // this, other set
            });
            
            // Method to create difference with another set
            obj.insert("difference".to_string(), Value::NativeFunction {
                name: "difference".to_string(),
                func: Box::new(|args| {
                    if args.len() != 2 {
                        return Err(ShitRustError::RuntimeError("Set.difference requires this and another Set".to_string()));
                    }
                    
                    let this = &args[0];
                    let other = &args[1];
                    
                    if let (Value::Object(this_obj), Value::Object(other_obj)) = (this, other) {
                        if let (Some(Value::Set(set1)), Some(Value::Set(set2))) = (this_obj.get("__data"), other_obj.get("__data")) {
                            let difference: RustHashSet<Value> = set1.difference(set2).cloned().collect();
                            
                            // Create a new Set with the difference
                            let mut new_obj = RustHashMap::new();
                            new_obj.insert("__type".to_string(), Value::String("Set".to_string()));
                            new_obj.insert("__data".to_string(), Value::Set(difference));
                            
                            // Copy methods from the original Set
                            for (key, value) in this_obj.iter() {
                                if !key.starts_with("__") {
                                    new_obj.insert(key.clone(), value.clone());
                                }
                            }
                            
                            Ok(Value::Object(new_obj))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set objects".to_string()))
                    }
                }),
                arity: 2, // this, other set
            });
            
            // Method to clear the set
            obj.insert("clear".to_string(), Value::NativeFunction {
                name: "clear".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("Set.clear requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::Set(_)) = this_obj.get("__data") {
                            let empty_set = RustHashSet::new();
                            
                            // Update with empty set
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::Set(empty_set));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected Set object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected Set object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            Ok(Value::Object(obj))
        }),
        arity: -1, // 0 or more args
    }
}

fn create_queue_constructor() -> Value {
    Value::NativeFunction {
        name: "Queue".to_string(),
        func: Box::new(|_args| {
            let mut obj = RustHashMap::new();
            obj.insert("__type".to_string(), Value::String("Queue".to_string()));
            // Basic implementation, would be expanded in real code
            Ok(Value::Object(obj))
        }),
        arity: -1,
    }
}

/// Creates a Priority Queue constructor
fn create_priority_queue_constructor() -> Value {
    Value::NativeFunction {
        name: "PriorityQueue".to_string(),
        func: Box::new(|_args| {
            // Since Rust doesn't have a built-in priority queue, we'll simulate one 
            // using a sorted vector of (priority, value) pairs
            let mut pq_data = Vec::new();
            
            // Create PriorityQueue object
            let mut obj = RustHashMap::new();
            obj.insert("__type".to_string(), Value::String("PriorityQueue".to_string()));
            obj.insert("__data".to_string(), Value::List(pq_data));
            
            // Method to enqueue an item with priority
            obj.insert("enqueue".to_string(), Value::NativeFunction {
                name: "enqueue".to_string(),
                func: Box::new(|args| {
                    if args.len() != 3 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.enqueue requires this, value, and priority arguments".to_string()));
                    }
                    
                    let this = &args[0];
                    let value = &args[1];
                    let priority = match &args[2] {
                        Value::Int(p) => *p,
                        Value::Float(p) => *p as i64,
                        _ => return Err(ShitRustError::TypeError("Priority must be a number".to_string())),
                    };
                    
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(queue)) = this_obj.get("__data") {
                            let mut queue_clone = queue.clone();
                            
                            // Create priority-value pair
                            let mut pair = RustHashMap::new();
                            pair.insert("priority".to_string(), Value::Int(priority));
                            pair.insert("value".to_string(), value.clone());
                            
                            // Insert the pair into the queue
                            queue_clone.push(Value::Object(pair));
                            
                            // Sort by priority (higher numbers = higher priority)
                            queue_clone.sort_by(|a, b| {
                                if let (Value::Object(a_obj), Value::Object(b_obj)) = (a, b) {
                                    if let (Some(Value::Int(a_prio)), Some(Value::Int(b_prio))) = 
                                        (a_obj.get("priority"), b_obj.get("priority")) {
                                        return b_prio.cmp(a_prio); // Descending order
                                    }
                                }
                                std::cmp::Ordering::Equal
                            });
                            
                            // Update the queue in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::List(queue_clone));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 3, // this, value, priority
            });
            
            // Method to dequeue the highest priority item
            obj.insert("dequeue".to_string(), Value::NativeFunction {
                name: "dequeue".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.dequeue requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(queue)) = this_obj.get("__data") {
                            if queue.is_empty() {
                                return Err(ShitRustError::RuntimeError("PriorityQueue is empty".to_string()));
                            }
                            
                            let mut queue_clone = queue.clone();
                            
                            // Remove and return the highest priority item (first after sorting)
                            let item = queue_clone.remove(0);
                            
                            // Update the queue in this object
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::List(queue_clone));
                            
                            // Return the value part of the priority-value pair
                            if let Value::Object(item_obj) = item {
                                if let Some(value) = item_obj.get("value") {
                                    return Ok(value.clone());
                                }
                            }
                            
                            Err(ShitRustError::TypeError("Invalid PriorityQueue item format".to_string()))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to peek at the highest priority item without removing
            obj.insert("peek".to_string(), Value::NativeFunction {
                name: "peek".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.peek requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(queue)) = this_obj.get("__data") {
                            if queue.is_empty() {
                                return Ok(Value::None);
                            }
                            
                            // Return the value part of the priority-value pair
                            if let Value::Object(item_obj) = &queue[0] {
                                if let Some(value) = item_obj.get("value") {
                                    return Ok(value.clone());
                                }
                            }
                            
                            Err(ShitRustError::TypeError("Invalid PriorityQueue item format".to_string()))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to get size
            obj.insert("size".to_string(), Value::NativeFunction {
                name: "size".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.size requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(queue)) = this_obj.get("__data") {
                            Ok(Value::Int(queue.len() as i64))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to check if empty
            obj.insert("is_empty".to_string(), Value::NativeFunction {
                name: "is_empty".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.is_empty requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(queue)) = this_obj.get("__data") {
                            Ok(Value::Bool(queue.is_empty()))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            // Method to clear the queue
            obj.insert("clear".to_string(), Value::NativeFunction {
                name: "clear".to_string(),
                func: Box::new(|args| {
                    if args.len() != 1 {
                        return Err(ShitRustError::RuntimeError("PriorityQueue.clear requires this argument".to_string()));
                    }
                    
                    let this = &args[0];
                    if let Value::Object(this_obj) = this {
                        if let Some(Value::List(_)) = this_obj.get("__data") {
                            let empty_queue = Vec::new();
                            
                            // Update with empty queue
                            let mut this_clone = this_obj.clone();
                            this_clone.insert("__data".to_string(), Value::List(empty_queue));
                            
                            Ok(Value::Object(this_clone))
                        } else {
                            Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                        }
                    } else {
                        Err(ShitRustError::TypeError("Expected PriorityQueue object".to_string()))
                    }
                }),
                arity: 1, // just this
            });
            
            Ok(Value::Object(obj))
        }),
        arity: 0, // no args for constructor
    }
} 