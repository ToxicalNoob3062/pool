# ThreadPool

A simple, efficient thread pool for Rust.

## Overview

This crate provides a `ThreadPool` struct that allows you to execute jobs on a pool of worker threads. It's useful for offloading CPU-intensive or blocking I/O tasks to a separate set of threads, to avoid blocking the main thread of your application.

## Features

- Easy to use: Just create a `ThreadPool` and start adding jobs.
- Efficient: Workers are reused for multiple jobs, reducing the overhead of thread creation.
- Safe: Jobs are `Send` and `'static`, ensuring they can safely be sent across threads.
- Traceable: Enable the `trace` flag to print the states of the workers.

## License

This project is licensed under the MIT License.
