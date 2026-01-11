use futures::channel::oneshot;

/// Run a blocking computation in a separate thread and return a Future for its result.
pub fn blocking_compute<F, T>(compute: F) -> impl Future<Output = Result<T, oneshot::Canceled>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();

    std::thread::spawn(move || {
        let result = compute();
        let _ = tx.send(result);
    });

    rx
}
