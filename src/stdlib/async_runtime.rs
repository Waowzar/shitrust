use std::collections::{VecDeque, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{ShitRustError, Result};
use crate::interpreter::Value;

/// A simple task executor for ShitRust's async runtime
pub struct TaskExecutor {
    /// Queue of tasks ready to run
    ready_queue: VecDeque<Arc<Task>>,
    /// Tasks that are sleeping
    sleeping_tasks: Vec<(Arc<Task>, Instant)>,
}

struct Task {
    /// The future that this task is executing
    future: Mutex<Option<Pin<Box<dyn Future<Output = Value> + Send>>>>,
    /// Task-local data
    task_data: Mutex<TaskData>,
}

struct TaskData {
    /// Whether the task has been woken up
    is_woken: bool,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.mark_ready();
    }
    
    fn wake_by_ref(self: &Arc<Self>) {
        self.mark_ready();
    }
}

impl Task {
    /// Create a new task
    fn new(future: Pin<Box<dyn Future<Output = Value> + Send>>) -> Self {
        Task {
            future: Mutex::new(Some(future)),
            task_data: Mutex::new(TaskData {
                is_woken: false,
            }),
        }
    }
    
    /// Mark the task as ready to run
    fn mark_ready(&self) {
        let mut data = self.task_data.lock().unwrap();
        data.is_woken = true;
    }
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new() -> Self {
        TaskExecutor {
            ready_queue: VecDeque::new(),
            sleeping_tasks: Vec::new(),
        }
    }
    
    /// Spawn a new task
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = Value> + Send + 'static,
    {
        let task = Arc::new(Task::new(Box::pin(future)));
        self.ready_queue.push_back(task);
    }
    
    /// Put a task to sleep for a specified duration
    pub fn sleep(&mut self, task: Arc<Task>, duration: Duration) {
        let wake_time = Instant::now() + duration;
        self.sleeping_tasks.push((task, wake_time));
    }
    
    /// Run the task executor until all tasks are complete
    pub fn run(&mut self) -> Result<()> {
        while !self.ready_queue.is_empty() || !self.sleeping_tasks.is_empty() {
            // Wake up any sleeping tasks that are ready
            let now = Instant::now();
            let mut i = 0;
            while i < self.sleeping_tasks.len() {
                if self.sleeping_tasks[i].1 <= now {
                    let (task, _) = self.sleeping_tasks.swap_remove(i);
                    task.mark_ready();
                    self.ready_queue.push_back(task);
                } else {
                    i += 1;
                }
            }
            
            // Process ready tasks
            if let Some(task) = self.ready_queue.pop_front() {
                let waker = Waker::from(task.clone());
                let mut context = Context::from_waker(&waker);
                
                let mut future_slot = task.future.lock().unwrap();
                if let Some(mut future) = future_slot.take() {
                    match future.as_mut().poll(&mut context) {
                        Poll::Pending => {
                            // Put the future back and check if it's been woken
                            *future_slot = Some(future);
                            
                            let mut data = task.task_data.lock().unwrap();
                            if data.is_woken {
                                // If it's been woken, put it back in the queue
                                data.is_woken = false;
                                drop(data);
                                drop(future_slot);
                                self.ready_queue.push_back(task);
                            }
                        }
                        Poll::Ready(_) => {
                            // Future is complete, nothing to do
                        }
                    }
                }
            } else if !self.sleeping_tasks.is_empty() {
                // If we have no ready tasks but have sleeping tasks, 
                // sleep until the next task is ready
                let min_time = self.sleeping_tasks
                    .iter()
                    .map(|(_, time)| *time)
                    .min()
                    .unwrap();
                
                let now = Instant::now();
                if min_time > now {
                    thread::sleep(min_time - now);
                }
            }
        }
        
        Ok(())
    }
}

/// ShitRust async runtime
pub struct AsyncRuntime {
    /// Task executor
    executor: TaskExecutor,
}

impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> Self {
        AsyncRuntime {
            executor: TaskExecutor::new(),
        }
    }
    
    /// Run a future to completion
    pub fn block_on<F>(&mut self, future: F) -> Result<Value>
    where
        F: Future<Output = Value> + Send + 'static,
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        
        // Wrap the future to send its result over the channel
        let wrapped_future = async move {
            let result = future.await;
            let _ = sender.send(result.clone());
            result
        };
        
        // Spawn the future
        self.executor.spawn(wrapped_future);
        
        // Run the executor
        self.executor.run()?;
        
        // Get the result
        receiver.recv().map_err(|_| ShitRustError::RuntimeError("Future panicked".to_string()))
    }
    
    /// Create a future that resolves after a specified duration
    pub fn sleep(&self, duration: Duration) -> impl Future<Output = Value> {
        struct Sleep {
            duration: Duration,
        }
        
        impl Future for Sleep {
            type Output = Value;
            
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let waker = cx.waker().clone();
                
                // Spawn a new thread to wake up after the duration
                let duration = self.duration;
                thread::spawn(move || {
                    thread::sleep(duration);
                    waker.wake();
                });
                
                Poll::Pending
            }
        }
        
        Sleep { duration }
    }
}

/// Initialize the async runtime module
pub fn init_async_runtime_module() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    
    // Add the AsyncRuntime constructor
    exports.insert("AsyncRuntime".to_string(), Value::NativeFunction {
        name: "AsyncRuntime".to_string(),
        arity: 0,
        function: Box::new(|_, _| {
            Ok(Value::Object(Box::new(AsyncRuntime::new())))
        }),
    });
    
    // Add sleep function
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
            
            let runtime = AsyncRuntime::new();
            let future = runtime.sleep(Duration::from_millis(milliseconds));
            
            Ok(Value::Future(Box::new(future)))
        }),
    });
    
    exports
} 
