use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

/// A conditional thread-safe cell optimized for frequent reads, some guarded updates, and rare atomic replacements.
///
/// # Thread Safety Model
/// - **Lock-Free Reads** (`load`): Always thread-safe.
/// - **Mutex-Guarded Updates** (`update`): Thread-safe for concurrent access.
/// - **Unsafe Atomic Stores** (`store`): Not thread-safe; caller must ensure no concurrent calls.
///
/// # Design
/// - Use `load()` for high-frequency reads (no locks).
/// - Use `update()` for safe in-place modifications (internal mutex).
/// - Avoid `store()` unless absolutely needed (it bypasses the update mutex).
///
/// # Example
/// ```
/// #[derive(Default)]
/// pub struct Foo {
///    pub bar: String,
/// }
///
/// let cell = AtomicCell::new(Foo::default());
/// // Lock-free read
/// let cfg = cell.load();
/// // Field-level update
/// cell.update(|cfg| cfg.bar = "new".to_string());
/// // Full replacement (must not race with update)
/// cell.store(Config::default());
/// ```
pub struct AtomicCell<T> {
    data: AtomicPtr<UnsafeCell<T>>,
    // Only protects `update`, does not block `load` or `store`.
    update_lock: Mutex<()>,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send + Sync> Send for AtomicCell<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicCell<T> {}

impl<T> AtomicCell<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new `AtomicCell` containing the given value.
    pub fn new(value: T) -> Self {
        let cell = Box::new(UnsafeCell::new(value));
        Self {
            data: AtomicPtr::new(Box::into_raw(cell)),
            update_lock: Mutex::new(()),
            _marker: PhantomData,
        }
    }

    /// Lock-free read access to the contained value.
    ///
    /// # Safety
    /// The returned reference is valid as long as there is no concurrent `store`.
    pub fn load(&self) -> &T {
        let ptr = self.data.load(Ordering::Acquire);
        unsafe { &*(*ptr).get() }
    }

    /// Replaces the current value atomically.
    ///
    /// # Safety
    /// This method is **not thread-safe by default**. Caller must ensure:
    /// - No concurrent calls to `store()`.
    /// - No outstanding references from `load()` when called.
    pub fn store(&self, new_value: T) {
        let cell = Box::new(UnsafeCell::new(new_value));
        let new_ptr = Box::into_raw(cell);
        let old_ptr = self.data.swap(new_ptr, Ordering::Release);
        unsafe {
            drop(Box::from_raw(old_ptr));
        }
    }

    /// Updates the value in-place with mutex protection.
    ///
    /// This is safe for concurrent use but will block if another thread is updating.
    /// ```
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let _guard = self.update_lock.lock();
        let ptr = self.data.load(Ordering::Acquire);
        updater(unsafe { &mut *(*ptr).get() });
    }
}

impl<T> Drop for AtomicCell<T> {
    fn drop(&mut self) {
        let ptr = self.data.load(Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}
