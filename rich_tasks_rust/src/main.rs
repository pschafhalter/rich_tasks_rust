use {
    futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake},
        future::{Abortable, AbortHandle, Aborted},
    },
    // PS: The timer we wrote in the previous section:
    // timer_future::TimerFuture,
    std::{
        cmp::Ordering,
        collections::BinaryHeap,
        future::Future,
        sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError},
        // JW: mpsc stands for multi-producer single-consumer
        sync::{Arc, Mutex},
        task::{Context, Poll},
        result::Result,
    },
};
// JW: send means you can transfer across thread boundaries

/// PS: Task executor that receives tasks off of a channel and runs them.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    fn run(&self) {
        let mut run_queue = BinaryHeap::new();
        loop {
            // JW: Populate run queue.
            loop {
                match self.ready_queue.try_recv() {
                    Ok(task) => run_queue.push(task),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
                // JW: Only exit loop when there's nothing left on the ready_queue
                // Runtime quits when the ready_queue is disconnected
            }

            // PS: Run the next task.
            if let Some(task) = run_queue.pop() {
                // PS: Take the future, and if it has not yet completed (is still Some),
                // poll it in an attempt to complete it.
                let mut future_slot = task.future.lock().unwrap();
                // future_slot should be of type BoxFuture
                if let Some(mut future) = future_slot.take() {
                    // PS: Create a `LocalWaker` from the task itself
                    // JW: TODO: find out about LocalWaker;; https://docs.rs/futures/0.3.5/futures/task/struct.Waker.html
                    // Probably moves something from wait queue to run queue
                    let waker = waker_ref(&task);
                    let context = &mut Context::from_waker(&*waker);
                    // `BoxFuture<T>` is a type alias for
                    // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                    // We can get a `Pin<&mut dyn Future + Send + 'static>`
                    // from it by calling the `Pin::as_mut` method.
                    if let Poll::Pending = future.as_mut().poll(context) {
                        // JW: To poll futures they must be pinned; run the future as far as possible
                        // Alternatively, Poll::Ready if it's not pending and completed
                        //
                        // PS: We're not done processing the future, so put it
                        // back in its task to be run again in the future.
                        *future_slot = Some(future);
                    }
                }
            }
        }
    }
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
    // JW: is clone-able
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = Result<(), Aborted>> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            priority: 20,
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }

    fn spawn_abortable_with_priority(&self, future: impl Future<Output = Result<(), Aborted>> + 'static + Send, priority: u8) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            priority,
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, Result<(), Aborted>>>>,

    /// Tasks with smaller priorities are executed first.
    priority: u8,

    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

// Define an ordering across tasks based on priority.
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).reverse()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Task {}

//A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically Reference Counted'.
// Task imps ArcWake which means type wrapped in arc can be converted to a waker
// waker can indicate to the executor that it's ready to be polled again.

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // Maximum number of tasks to allow queueing in the channel at once.
    // This is just to make `sync_channel` happy, and wouldn't be present in
    // a real executor.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn an abortable task (WORKS)
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(async { println!("blah") }, abort_registration);
    spawner.spawn(future);
    abort_handle.abort();

    // duplicate code soz; testing priority
    let (_abort_handle1, abort_registration) = AbortHandle::new_pair();
    let future1 = Abortable::new(async { println!("p1 task") }, abort_registration);
    let (abort_handle2, abort_registration) = AbortHandle::new_pair();
    let future2 = Abortable::new(async { println!("p2 task") }, abort_registration);
    let (_abort_handle3, abort_registration) = AbortHandle::new_pair();
    let future3 = Abortable::new(async { println!("p3 task") }, abort_registration);
    let (_abort_handle4, abort_registration) = AbortHandle::new_pair();
    let future4 = Abortable::new(async { println!("p4 task") }, abort_registration);
    spawner.spawn_abortable_with_priority(future2, 2);
    spawner.spawn_abortable_with_priority(future1, 1);
    spawner.spawn_abortable_with_priority(future4, 4);
    spawner.spawn_abortable_with_priority(future3, 3);
    abort_handle2.abort();

    // Drop the spawner so that our executor knows it is finished and won't
    // receive more incoming tasks to run.
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    executor.run();
}
