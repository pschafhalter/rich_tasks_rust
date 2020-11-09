10/28/20

- We walked through and commented rich_task_rust
  - Keys to note: Pin type pins the address of the object
  - BoxedFuture just means Pinned
  - Algo: while the ready_queue is not empty and not disconnected move stuff to
    the run_queue; otherwise run thing on queue until they're blocked; repeat
- Try to add spawn_with_priority to runtime/spawner.rs and task/spawn.rs
  - this just prints the priority
- Add test/udp_prio.rs which narrows down to just run one of the udp test
  requiring spawn
- run test: 
``` 
  cargo build --all-features
  cargo check --all-features
  cargo test udp_prio -- --nocapture 2>&1| grep -A 10 prio
```
- edited mod.rs and lib.rs to make the new fn visible 

11/03/20

- Abortable futures: https://docs.rs/futures/0.3.7/futures/future/struct.Abortable.html
Questions: 
- What do about GPU not being abortable?
- Can all scheduling decisions be reduced to priorities?
- How does the Timestamps work (what are coordinates and when are they set)?
- 

11/05/20 - ??? (Eric notes on scheduling in Tokio)
- Looking in basic_scheduler.rs to understand what is going on
- They seem to have thread specific queues and a shared queue as well
- I need to look at how these queues are used (looks like FIFO lol)
- In addition how the scheduling actually happens (task/mod.rs? or if basic_scheduler is a red herring)

11/09/2020 (Eric notes on Abortable Future)
- successfully adapted tasks to use AbortableFutures (change ouput signature to match Result<(), Aborted>)
- tested scheduling abortables with priority and aborting them
- need to test aborting mid-execution with longer-running tasks than prints LOL