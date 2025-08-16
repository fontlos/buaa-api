use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

/// You need to implement this trait if you want to use `update_atomic` method
pub trait AtomicType: Send + Sync {
    type Wrapped;
    fn store(&self, value: Self::Wrapped, order: Ordering);
}

/// A conditional thread-safe cell optimized for frequent reads, some guarded updates, and rare atomic replacements.
///
/// # Thread Safety Model
///
/// - **Lock-Free Reads** (`load`): Always thread-safe.
/// - **Mutex-Guarded Updates** (`update`): Thread-safe for concurrent access.
/// - **Unsafe Atomic Stores** (`store`): Not thread-safe; caller must ensure no concurrent calls.
///
/// # Design
///
/// - Use `load()` for high-frequency reads (no locks).
/// - Use `update()` for safe in-place modifications (internal mutex).
/// - Avoid `store()` unless absolutely needed (it bypasses the update mutex).
///
/// # Example
///
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
    /// Create a new `AtomicCell` containing the given value.
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
    ///
    /// The returned reference is valid as long as there is no concurrent `store`.
    pub fn load(&self) -> &T {
        let ptr = self.data.load(Ordering::Acquire);
        unsafe { &*(*ptr).get() }
    }

    /// Replace the current value atomically.
    ///
    /// # Safety
    ///
    /// This method is **not thread-safe by default**. Caller must ensure:
    ///
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

    /// Update the value in-place with mutex protection.
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
    /// If the field is atomic and implements the [AtomicType] trait,
    /// you can use this method to update the corresponding field atomically without locking.
    ///
    /// The closure must return a mutable reference to the atomic field and the new value.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::atomic::{AtomicU64, Ordering};
    ///
    /// impl AtomicType for AtomicU64 {
    ///     type Wrapped = u64;
    ///     fn store(&self, value: u64, order: Ordering) { self.store(value, order); }
    /// }
    /// struct Data {
    ///     non_atomic_value: String,
    ///     atomic_value: AtomicU64,
    /// }
    ///
    /// let cell = AtomicCell::new(Data {
    ///     non_atomic_value: "test".to_string(),
    ///     atomic_value: AtomicU64::new(0),
    /// });
    ///
    /// // Update
    /// cell.update(|data| {
    ///     data.non_atomic_value = "updated".to_string();
    /// });
    /// // Update atomic field
    /// cell.update_atomic(|data| (&data.atomic_value, 42));
    ///
    /// let data = cell.load();
    /// assert_eq!(data.non_atomic_value, "updated");
    /// assert_eq!(data.atomic_value.load(Ordering::Relaxed), 42);
    /// ```
    pub fn update_atomic<A, F>(&self, updater: F)
    where
        // The constraint field must implement AtomicType
        A: AtomicType,
        // The closure must return a mutable reference to the atomic field and the new value.
        F: FnOnce(&mut T) -> (&A, A::Wrapped),
    {
        let ptr = self.data.load(Ordering::Acquire);
        let (atomic_ref, value) = updater(unsafe { &mut *(*ptr).get() });
        atomic_ref.store(value, Ordering::Release);
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
