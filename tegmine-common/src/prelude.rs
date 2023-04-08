#[rustfmt::skip]
pub use crate::exception::{ErrorCode, TegResult};
pub use crate::macros::{addr_of, addr_of_cast_mut, addr_of_mut, from_addr, from_addr_mut};
pub use crate::{fmt_err, rcrefcell, shared_ref, str_err};
pub type RcRefCell<T> = std::rc::Rc<std::cell::RefCell<T>>;
pub type SharedRef<T> = std::sync::Arc<parking_lot::ReentrantMutex<std::cell::RefCell<T>>>;

#[rustfmt::skip]
// std
pub use std::cell::RefCell;
pub use std::cmp::Ordering;
pub use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
pub use std::ops::ControlFlow;
pub use std::rc::{Rc, Weak};
pub use std::sync::atomic::{
    AtomicBool, AtomicI32, AtomicI64, AtomicUsize, Ordering as AtomicOrdering,
};
pub use std::sync::{Arc, Weak as AsyncWeek};

#[rustfmt::skip]
pub type InlineStr = smartstring::SmartString<smartstring::Compact>;
pub use lazy_static::lazy_static;
pub use once_cell::sync::{Lazy, OnceCell};
pub use parking_lot::{Mutex, RwLock};

#[rustfmt::skip]
pub use log::Level::{
    Debug as LogLevelDebug, Info as LogLevelInfo, Trace as LogLevelTrace, Warn as LogLevelWarn,
};
pub use log::{debug, error, info, log_enabled, trace, warn, LevelFilter};

#[rustfmt::skip]
pub use crate::common::Object;
