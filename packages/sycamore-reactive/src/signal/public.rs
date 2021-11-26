use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

use crate::*;

pub type Signal<T> = StaticSignal<T>;
pub type ReadSignal<T> = StaticReadSignal<T>;

/// A readonly static [`Signal`].
///
/// Returned by functions that provide a handle to access state.
/// Use [`Signal::handle`] or [`Signal::into_handle`] to retrieve a handle from a [`Signal`].
pub struct StaticReadSignal<T: 'static>(pub(crate) &'static RefCell<SignalInner<T>>);

impl<T> Copy for StaticReadSignal<T> {}

/// A readonly dynamic [`Signal`].
///
/// Returned by functions that provide a handle to access state.
/// Use [`Signal::handle`] or [`Signal::into_handle`] to retrieve a handle from a [`Signal`].
pub struct DynReadSignal<T: 'static>(pub(crate) Rc<RefCell<SignalInner<T>>>);

pub trait ReadSignalTrait<T>: ReadSignalPrivate<T> {
    fn get(&self) -> Rc<T> {
        // If inside an effect, add this signal to dependency list.
        // If running inside a destructor, do nothing.
        let _ = LISTENERS.try_with(|listeners| {
            if let Some(last_context) = listeners.borrow().last() {
                last_context
                    .upgrade()
                    .expect_throw("Running should be valid while inside reactive scope")
                    .borrow_mut()
                    .as_mut()
                    .unwrap_throw()
                    .dependencies
                    .insert(Dependency(self.as_anysigref()));
            }
        });

        self.get_untracked()
    }

    fn get_untracked(&self) -> Rc<T> {
        Rc::clone(&self.as_refcell().borrow().inner)
    }
}

impl<T> ReadSignalTrait<T> for StaticReadSignal<T> {}
impl<T> ReadSignalTrait<T> for DynReadSignal<T> {}

impl<T> Clone for StaticReadSignal<T> {
    fn clone(&self) -> Self {
        StaticReadSignal(self.0)
    }
}
impl<T> Clone for DynReadSignal<T> {
    fn clone(&self) -> Self {
        DynReadSignal(self.0.clone())
    }
}

impl<T: Default> Default for StaticReadSignal<T> {
    fn default() -> Self {
        StaticSignal::new(T::default()).into_handle()
    }
}

impl<T: Default> Default for DynReadSignal<T> {
    fn default() -> Self {
        DynSignal::new(T::default()).into_handle()
    }
}

impl<T: fmt::Debug> fmt::Debug for StaticReadSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StaticReadSignal")
            .field(&self.get_untracked())
            .finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for DynReadSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynReadSignal")
            .field(&self.get_untracked())
            .finish()
    }
}
#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for StaticReadSignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for DynReadSignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for StaticReadSignal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(StaticSignal::new(T::deserialize(deserializer)?).handle())
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for DynReadSignal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(DynSignal::new(T::deserialize(deserializer)?).handle())
    }
}

/// Static state that can be set.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
/// assert_eq!(*state.get(), 0);
///
/// state.set(1);
/// assert_eq!(*state.get(), 1);
/// ```
pub struct StaticSignal<T: 'static> {
    pub(crate) handle: StaticReadSignal<T>,
}

impl<T> Copy for StaticSignal<T> {}

impl<T> Clone for StaticSignal<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

/// Dynamic state that can be set.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
/// assert_eq!(*state.get(), 0);
///
/// state.set(1);
/// assert_eq!(*state.get(), 1);
/// ```
pub struct DynSignal<T: 'static> {
    pub(crate) handle: DynReadSignal<T>,
}

impl<T> Clone for DynSignal<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<T> SignalTrait<T> for StaticSignal<T> {
    type ReadSignalType = StaticReadSignal<T>;

    #[inline]
    fn new(initial: T) -> Self {
        Self {
            handle: StaticReadSignal(Box::leak(Box::new(RefCell::new(SignalInner::new(initial))))),
        }
    }
    #[inline]
    fn handle(&self) -> StaticReadSignal<T> {
        self.handle.clone()
    }

    #[inline]
    fn into_handle(self) -> StaticReadSignal<T> {
        self.handle
    }
}

impl<T> SignalTrait<T> for DynSignal<T> {
    type ReadSignalType = DynReadSignal<T>;

    #[inline]
    fn new(initial: T) -> Self {
        Self {
            handle: DynReadSignal(Rc::new(RefCell::new(SignalInner::new(initial)))),
        }
    }

    #[inline]
    fn handle(&self) -> DynReadSignal<T> {
        self.handle.clone()
    }

    #[inline]
    fn into_handle(self) -> DynReadSignal<T> {
        self.handle
    }
}

pub trait SignalTrait<T: 'static>: SignalPrivate<T> {
    type ReadSignalType;

    /// Creates a new signal with the given value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// let state = Signal::new(0);
    /// # assert_eq!(*state.get(), 0);
    /// ```
    fn new(initial: T) -> Self;

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    ///
    /// let state = Signal::new(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// ```
    fn set(&self, new_value: T) {
        self.sig_as_refcell().borrow_mut().update(new_value);

        self.trigger_subscribers();
    }

    /// Get the [`ReadSignal`] associated with this signal.
    ///
    /// This is a shortcut for `(*signal).clone()`.
    fn handle(&self) -> Self::ReadSignalType;

    /// Consumes this signal and returns its underlying [`ReadSignal`].
    fn into_handle(self) -> Self::ReadSignalType;

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set`] instead.
    fn trigger_subscribers(&self) {
        // Clone subscribers to prevent modifying list when calling callbacks.
        let subscribers = self.sig_as_refcell().borrow().subscribers.clone();

        // Reverse order of subscribers to trigger outer effects before inner effects.
        for subscriber in subscribers.values().rev() {
            // subscriber might have already been destroyed in the case of nested effects
            if let Some(callback) = subscriber.try_callback() {
                // Might already be inside a callback, if infinite loop.
                // Do nothing if infinite loop.
                if let Ok(mut callback) = callback.try_borrow_mut() {
                    callback()
                }
            }
        }
    }
}

// @@@ can implmement once?
impl<T: Default> Default for StaticSignal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Default> Default for DynSignal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: 'static> Deref for StaticSignal<T> {
    type Target = StaticReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T: 'static> Deref for DynSignal<T> {
    type Target = DynReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T: PartialEq> PartialEq for StaticSignal<T> {
    fn eq(&self, other: &StaticSignal<T>) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}

impl<T: PartialEq> PartialEq for DynSignal<T> {
    fn eq(&self, other: &DynSignal<T>) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}

impl<T: Eq> Eq for StaticSignal<T> {}
impl<T: Eq> Eq for DynSignal<T> {}

impl<T: Hash> Hash for StaticSignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state);
    }
}
impl<T: Hash> Hash for DynSignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state);
    }
}

impl<T: fmt::Debug> fmt::Debug for StaticSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StaticSignal")
            .field(&self.get_untracked())
            .finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for DynSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSignal")
            .field(&self.get_untracked())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for StaticSignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for StaticSignal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(StaticSignal::new(T::deserialize(deserializer)?))
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for DynSignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for DynSignal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(DynSignal::new(T::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals() {
        let state = Signal::new(0);
        assert_eq!(*state.get(), 0);

        state.set(1);
        assert_eq!(*state.get(), 1);
    }

    #[test]
    fn signal_composition() {
        let state = Signal::new(0);

        let double = || *state.get() * 2;

        assert_eq!(double(), 0);

        state.set(1);
        assert_eq!(double(), 2);
    }

    #[test]
    fn state_handle() {
        let state = Signal::new(0);
        let readonly = state.handle();

        assert_eq!(*readonly.get(), 0);

        state.set(1);
        assert_eq!(*readonly.get(), 1);
    }
}
