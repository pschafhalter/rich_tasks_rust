//use {
//    futures::{
//        future::{BoxFuture, FutureExt},
//        task::{waker_ref, ArcWake},
//        future::{Abortable, AbortHandle, Aborted},
//    },
//    // PS: The timer we wrote in the previous section:
//    // timer_future::TimerFuture,
//    std::{
//        cmp::Ordering,
//        collections::BinaryHeap,
//        future::Future,
//        sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError},
//        // JW: mpsc stands for multi-producer single-consumer
//        sync::{Arc, Mutex},
//        task::{Context, Poll},
//        result::Result,
//    },
//};
use futures::future::{Abortable,AbortHandle};
use rich_tasks;
// JW: send means you can transfer across thread boundaries

#[test]
fn abortable() {
    let (executor, spawner) = rich_tasks::new_executor_and_spawner();

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
