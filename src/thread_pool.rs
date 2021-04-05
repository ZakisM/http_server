use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type TaskType = VecDeque<Box<dyn Fn() + Send + 'static>>;

pub struct ThreadPool {
    _size: usize,
    condvar: Arc<(Mutex<TaskType>, Condvar)>,
    _workers: Vec<JoinHandle<()>>,
    number_of_tasks: Arc<AtomicUsize>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let tasks = VecDeque::new();

        let condvar = Arc::new((Mutex::new(tasks), Condvar::new()));

        let mut workers = Vec::with_capacity(size);

        let number_of_tasks = Arc::new(AtomicUsize::new(0));

        let drain_condvar = Arc::clone(&condvar);
        let number_of_tasks_clone = Arc::clone(&number_of_tasks);

        thread::spawn(move || loop {
            let number_of_tasks = number_of_tasks_clone.load(Ordering::Acquire);
            if number_of_tasks > 0 {
                if number_of_tasks >= size {
                    drain_condvar.1.notify_all();
                } else {
                    drain_condvar.1.notify_one();
                }
            }
            thread::sleep(Duration::from_millis(10));
        });

        for _ in 0..size {
            let condvar = Arc::clone(&condvar);

            workers.push(thread::spawn(move || {
                let (tasks_lock, cvar) = &*condvar;

                loop {
                    let task = {
                        let tasks_lock = tasks_lock.lock().unwrap();
                        let mut cvar_lock: MutexGuard<TaskType> = cvar.wait(tasks_lock).unwrap();

                        if let Some(task) = cvar_lock.pop_front() {
                            Some(task)
                        } else {
                            None
                        }
                    };

                    if let Some(task) = task {
                        (*task)()
                    }
                }
            }));
        }

        ThreadPool {
            _size: size,
            condvar,
            _workers: workers,
            number_of_tasks,
        }
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: Fn() + Send + 'static,
    {
        let mut tasks = self.condvar.0.lock().unwrap();
        tasks.push_back(Box::new(f));
        self.number_of_tasks.fetch_add(1, Ordering::Release);
    }
}
