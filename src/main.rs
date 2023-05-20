use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

struct Task {
    wait_time_s: u64,
    id: u8,
}

impl Task {
    fn random() -> Task {
        let task = Task {
            wait_time_s: (rand::random::<u64>() % 5) + 5,
            id: rand::random(),
        };
        task.print();
        task
    }
    fn run(&self, thread: usize) {
        println!(
            "> Running task: id={} s={} thread={}",
            self.id, self.wait_time_s, thread
        );

        thread::sleep(std::time::Duration::from_secs(self.wait_time_s));

        println!(
            "> Finished task: id={} s={} thread={}",
            self.id, self.wait_time_s, thread
        );
    }
    fn print(&self) {
        println!("> Created task: id={} s={}", self.id, self.wait_time_s);
    }
}

struct ThreadPool {
    queue: Arc<(Mutex<Vec<Task>>, Condvar)>,
    is_paused: Arc<(Mutex<bool>, Condvar)>,
}

impl ThreadPool {
    fn new(n_threads: usize) -> ThreadPool {
        let queue = Arc::new((Mutex::new(Vec::<Task>::new()), Condvar::new()));
        let mut threads = Vec::with_capacity(n_threads);
        let is_paused = Arc::new((Mutex::new(false), Condvar::new()));

        for i in 0..n_threads {
            let queue = Arc::clone(&queue);
            let is_paused = Arc::clone(&is_paused);
            threads.push(thread::spawn(move || {
                loop {
                    let task = {
                        let (ref mutex, ref cvar) = *queue;
                        let mut queue = mutex.lock().unwrap();

                        while queue.is_empty() {
                            // The thread waits until notified by a signal.
                            queue = cvar.wait(queue).unwrap();
                        }

                        queue.remove(0)
                    };
                    task.run(i);

                    let (ref mutex, ref cvar) = *is_paused;
                    let mut is_paused = mutex.lock().unwrap();

                    while is_paused.clone() {
                        println!("> Thread {} is paused, waiting for resumal", i);
                        // The thread waits until notified by a signal.
                        is_paused = cvar.wait(is_paused).unwrap();

                        println!("> Thread {} is resumed", i);
                    }
                }
            }));
        }

        ThreadPool { queue, is_paused }
    }

    fn destroy(&self) {
        println!("> Waiting for all tasks to finish");

        loop {
            let queue = Arc::clone(&self.queue);
            let (ref mutex, ref cvar) = *queue;
            let queue = mutex.lock().unwrap();

            if queue.is_empty() {
                break;
            }
        }

        println!("> Exited successfully");
    }

    fn add_task(&self, task: Task) {
        let (ref mutex, ref cvar) = *self.queue;
        let mut queue = mutex.lock().unwrap();

        if queue.len() >= 20 {
            println!("> Task discarded, queue is full");
            return;
        }
        queue.push(task);
        cvar.notify_one();
    }

    fn pause(&self) {
        let (ref mutex, ref cvar) = *self.is_paused;
        let mut is_paused = mutex.lock().unwrap();

        if !*is_paused {
            *is_paused = true;
            println!("> Pausing all threads");
            cvar.notify_all();
        }
    }

    fn resume(&self) {
        let (ref mutex, ref cvar) = *self.is_paused;
        let mut is_paused = mutex.lock().unwrap();

        if *is_paused {
            *is_paused = false;
            println!("> Resuming all threads");
            cvar.notify_all();
        }
    }
}

fn main() {
    println!("Hello!");
    let thread_pool = ThreadPool::new(6);
    println!("ThreadPool created");

    println!(
        "
Available commands:
> exit - to exit the program
> add - add a task to the queue
> add_10 - add 10 tasks to the queue
> add_20 - add 20 tasks to the queue
> add_30 - add 30 tasks to the queue
> pause - pause all threads
> resume - resume the work
"
    );
    loop {
        let mut command = String::new();
        std::io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        match command {
            "exit" => {
                thread_pool.destroy();
                break;
            }
            "add" => {
                let task = Task::random();
                thread_pool.add_task(task);
            }
            "add_10" => {
                for _ in 0..10 {
                    let task = Task::random();
                    thread_pool.add_task(task);
                }
            }
            "add_20" => {
                for _ in 0..20 {
                    let task = Task::random();
                    thread_pool.add_task(task);
                }
            }

            "add_30" => {
                for _ in 0..30 {
                    let task = Task::random();
                    thread_pool.add_task(task);
                }
            }
            "pause" => {
                thread_pool.pause();
            }
            "resume" => {
                thread_pool.resume();
            }

            "remove" => println!("Removing task"),
            _ => println!("Invalid command"),
        }
    }
}
