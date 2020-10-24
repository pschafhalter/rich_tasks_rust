# Rich Tasks

Proof of concept for advanced scheduling of rich tasks on a Rust async runtime.

Rich tasks are annotated with additional properties such as priorities via a trait.
These properties enable more complex scheduling policies as opposed to FIFO scheduling.
In addition, rich tasks may enable dynamic scheduling where the order of scheduled tasks may change dynamically at runtime, e.g. via priority donation.

This PoC is based on the Rust Async Book's tutorial on [building an executor](https://rust-lang.github.io/async-book/02_execution/04_executor.html).
