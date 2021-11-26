use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::*;

#[doc(hidden)]
pub enum AnySigRef {
    Static(&'static dyn AnySignalInner),
    Dynamic(Rc<dyn AnySignalInner>),
}

impl AnySigRef {
    pub(crate) fn as_ptr(&self) -> *const (dyn AnySignalInner + 'static) {
        match self {
            AnySigRef::Static(t) => *t,
            AnySigRef::Dynamic(rc) => Rc::as_ptr(rc),
        }
    }
}

#[doc(hidden)]
pub trait ReadSignalPrivate<T> {
    fn as_refcell(&self) -> &RefCell<SignalInner<T>>;
    fn as_anysigref(&self) -> AnySigRef;
}

impl<T> ReadSignalPrivate<T> for StaticReadSignal<T> {
    fn as_refcell(&self) -> &RefCell<SignalInner<T>> {
        self.0
    }
    fn as_anysigref(&self) -> AnySigRef {
        AnySigRef::Static(self.0)
    }
}

impl<T> ReadSignalPrivate<T> for DynReadSignal<T> {
    fn as_refcell(&self) -> &RefCell<SignalInner<T>> {
        self.0.as_ref()
    }
    fn as_anysigref(&self) -> AnySigRef {
        AnySigRef::Dynamic(self.0.clone())
    }
}

#[doc(hidden)]
pub trait SignalPrivate<T> {
    fn sig_as_refcell(&self) -> &RefCell<SignalInner<T>>;
}

impl<T> SignalPrivate<T> for StaticSignal<T> {
    fn sig_as_refcell(&self) -> &RefCell<SignalInner<T>> {
        self.handle.0
    }
}

impl<T> SignalPrivate<T> for DynSignal<T> {
    fn sig_as_refcell(&self) -> &RefCell<SignalInner<T>> {
        self.handle.0.as_ref()
    }
}

#[doc(hidden)]
pub struct SignalInner<T> {
    pub(crate) inner: Rc<T>,
    pub(crate) subscribers: IndexMap<CallbackPtr, Callback>,
}

impl<T> SignalInner<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            subscribers: IndexMap::new(),
        }
    }

    /// Adds a handler to the subscriber list. If the handler is already a subscriber, does nothing.
    pub(crate) fn subscribe(&mut self, handler: Callback) {
        self.subscribers.insert(handler.as_ptr(), handler);
    }

    /// Removes a handler from the subscriber list. If the handler is not a subscriber, does
    /// nothing.
    pub(crate) fn unsubscribe(&mut self, handler: CallbackPtr) {
        self.subscribers.remove(&handler);
    }

    /// Updates the inner value. This does **NOT** call the subscribers.
    /// You will have to do so manually with `trigger_subscribers`.
    pub(crate) fn update(&mut self, new_value: T) {
        self.inner = Rc::new(new_value);
    }
}

/// Trait for any [`SignalInner`], regardless of type param `T`.
#[doc(hidden)]
pub trait AnySignalInner {
    /// Wrapper around [`SignalInner::subscribe`].
    fn subscribe(&self, handler: Callback);
    /// Wrapper around [`SignalInner::unsubscribe`].
    fn unsubscribe(&self, handler: CallbackPtr);
}

impl<T> AnySignalInner for RefCell<SignalInner<T>> {
    fn subscribe(&self, handler: Callback) {
        self.borrow_mut().subscribe(handler);
    }

    fn unsubscribe(&self, handler: CallbackPtr) {
        self.borrow_mut().unsubscribe(handler);
    }
}

impl<T> AnySignalInner for &'static RefCell<SignalInner<T>> {
    fn subscribe(&self, handler: Callback) {
        self.borrow_mut().subscribe(handler);
    }

    fn unsubscribe(&self, handler: CallbackPtr) {
        self.borrow_mut().unsubscribe(handler);
    }
}
