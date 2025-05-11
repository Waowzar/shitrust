use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Condvar};
use std::thread;
use std::time::Duration;

use crate::error::{ShitRustError, Result};
use crate::interpreter::{Value, Interpreter};

/// A thread handle for managing ShitRust threads
pub struct ThreadHandle {
    /// The join handle for the thread
    handle: Option<thread::JoinHandle<Result<Value>>>,
    /// Thread ID
    id: usize,
}

impl ThreadHandle {
    /// Create a new thread handle
    fn new(handle: thread::JoinHandle<Result<Value>>, id: usize) -> Self {
        ThreadHandle {
            handle: Some(handle),
            id,
        }
    }
    
    /// Join the thread, waiting for it to finish
    pub fn join(&mut self) -> Result<Value> {
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => result,
                Err(_) => Err(ShitRustError::RuntimeError(
                    format!("Thread {} panicked", self.id)
                )),
            }
        } else {
            Err(ShitRustError::RuntimeError(
                "Thread has already been joined".to_string()
            ))
        }
    }
    
    /// Check if the thread has finished
    pub fn is_finished(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.is_finished()
        } else {
            true
        }
    }
    
    /// Get the thread ID
    pub fn id(&self) -> usize {
        self.id
    }
}

/// A mutex for synchronizing access to shared data
pub struct SRMutex<T> {
    /// The underlying mutex
    mutex: Mutex<T>,
}

impl<T> SRMutex<T> {
    /// Create a new mutex
    pub fn new(value: T) -> Self {
        SRMutex {
            mutex: Mutex::new(value),
        }
    }
    
    /// Lock the mutex, blocking until it can be acquired
    pub fn lock(&self) -> Result<std::sync::MutexGuard<T>> {
        self.mutex.lock().map_err(|_| 
            ShitRustError::RuntimeError("Failed to acquire mutex lock".to_string())
        )
    }
}

/// A read-write lock for shared data
pub struct SRRwLock<T> {
    /// The underlying RwLock
    rwlock: RwLock<T>,
}

impl<T> SRRwLock<T> {
    /// Create a new read-write lock
    pub fn new(value: T) -> Self {
        SRRwLock {
            rwlock: RwLock::new(value),
        }
    }
    
    /// Acquire a read lock
    pub fn read(&self) -> Result<std::sync::RwLockReadGuard<T>> {
        self.rwlock.read().map_err(|_| 
            ShitRustError::RuntimeError("Failed to acquire read lock".to_string())
        )
    }
    
    /// Acquire a write lock
    pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<T>> {
        self.rwlock.write().map_err(|_| 
            ShitRustError::RuntimeError("Failed to acquire write lock".to_string())
        )
    }
}

/// A condition variable for thread synchronization
pub struct SRCondVar {
    /// The underlying condition variable
    condvar: Condvar,
}

impl SRCondVar {
    /// Create a new condition variable
    pub fn new() -> Self {
        SRCondVar {
            condvar: Condvar::new(),
        }
    }
    
    /// Notify one waiting thread
    pub fn notify_one(&self) {
        self.condvar.notify_one();
    }
    
    /// Notify all waiting threads
    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }
    
    /// Wait on the condition variable
    pub fn wait<T>(&self, guard: std::sync::MutexGuard<T>) -> Result<std::sync::MutexGuard<T>> {
        self.condvar.wait(guard).map_err(|_| 
            ShitRustError::RuntimeError("Failed to wait on condition variable".to_string())
        )
    }
    
    /// Wait on the condition variable with a timeout
    pub fn wait_timeout<T>(&self, guard: std::sync::MutexGuard<T>, duration: Duration) 
        -> Result<(std::sync::MutexGuard<T>, bool)> 
    {
        self.condvar.wait_timeout(guard, duration).map_err(|_| 
            ShitRustError::RuntimeError("Failed to wait on condition variable with timeout".to_string())
        )
    }
}

/// Thread-safe reference counted value
pub struct SRArc<T> {
    /// The underlying Arc
    arc: Arc<T>,
}

impl<T> SRArc<T> {
    /// Create a new Arc
    pub fn new(value: T) -> Self {
        SRArc {
            arc: Arc::new(value),
        }
    }
    
    /// Clone the Arc
    pub fn clone(&self) -> Self {
        SRArc {
            arc: self.arc.clone(),
        }
    }
}

/// Thread local storage
pub struct ThreadLocal<T> {
    /// The thread local storage
    storage: thread_local::ThreadLocal<T>,
}

impl<T: Send + 'static> ThreadLocal<T> {
    /// Create a new thread local storage
    pub fn new() -> Self {
        ThreadLocal {
            storage: thread_local::ThreadLocal::new(),
        }
    }
    
    /// Get the thread local value, or initialize it with the given function
    pub fn get_or<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.storage.get_or(init)
    }
}

/// Initialize the concurrent module
pub fn init_concurrent_module() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    
    // Thread creation function
    exports.insert("spawn".to_string(), Value::NativeFunction {
        name: "spawn".to_string(),
        arity: 1,
        function: Box::new(|args, interpreter| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("spawn() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let func = match &args[0] {
                Value::Function { .. } => args[0].clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "spawn() requires a function argument".to_string()
                )),
            };
            
            // Clone the interpreter for the new thread
            let mut thread_interpreter = interpreter.clone();
            
            // Create a static thread ID counter
            static NEXT_THREAD_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
            let thread_id = NEXT_THREAD_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            let handle = thread::spawn(move || {
                // Call the function with no arguments
                thread_interpreter.call_value(&func, &[])
            });
            
            let thread_handle = ThreadHandle::new(handle, thread_id);
            
            Ok(Value::Object(Box::new(thread_handle)))
        }),
    });
    
    // Sleep function
    exports.insert("sleep".to_string(), Value::NativeFunction {
        name: "sleep".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("sleep() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let milliseconds = match &args[0] {
                Value::Int(ms) => *ms as u64,
                _ => return Err(ShitRustError::RuntimeError(
                    "sleep() takes an integer milliseconds argument".to_string()
                )),
            };
            
            thread::sleep(Duration::from_millis(milliseconds));
            
            Ok(Value::Void)
        }),
    });
    
    // Mutex constructor
    exports.insert("Mutex".to_string(), Value::NativeFunction {
        name: "Mutex".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("Mutex() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let value = args[0].clone();
            let mutex = SRMutex::new(value);
            
            Ok(Value::Object(Box::new(mutex)))
        }),
    });
    
    // RwLock constructor
    exports.insert("RwLock".to_string(), Value::NativeFunction {
        name: "RwLock".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("RwLock() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let value = args[0].clone();
            let rwlock = SRRwLock::new(value);
            
            Ok(Value::Object(Box::new(rwlock)))
        }),
    });
    
    // CondVar constructor
    exports.insert("CondVar".to_string(), Value::NativeFunction {
        name: "CondVar".to_string(),
        arity: 0,
        function: Box::new(|args, _| {
            if !args.is_empty() {
                return Err(ShitRustError::RuntimeError(
                    format!("CondVar() takes 0 arguments, but {} were given", args.len())
                ));
            }
            
            let condvar = SRCondVar::new();
            
            Ok(Value::Object(Box::new(condvar)))
        }),
    });
    
    // Arc constructor
    exports.insert("Arc".to_string(), Value::NativeFunction {
        name: "Arc".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("Arc() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let value = args[0].clone();
            let arc = SRArc::new(value);
            
            Ok(Value::Object(Box::new(arc)))
        }),
    });
    
    exports
} 
