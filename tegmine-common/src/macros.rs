#[macro_export]
macro_rules! str_err {
    ($obj:ident, $str:expr) => {{
        Err(ErrorCode::$obj($str.to_string()))
    }};
}

#[macro_export]
macro_rules! fmt_err {
    ($obj:ident, $($arg:tt)*) => {{
        Err(ErrorCode::$obj(format!($($arg)*)))
    }}
}

#[macro_export]
macro_rules! rcrefcell {
    ($obj:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($obj))
    };
}

#[macro_export]
macro_rules! shared_ref {
    ($obj:expr) => {
        std::sync::Arc::new(parking_lot::ReentrantMutex::new(std::cell::RefCell::new(
            $obj,
        )))
    };
}

#[macro_export]
pub macro addr_of($place:expr) {
    unsafe { $place as *const _ }
}

#[macro_export]
pub macro addr_of_mut($place:expr) {
    unsafe { $place as *mut _ }
}

#[macro_export]
pub macro addr_of_cast_mut($place:expr) {
    unsafe { $place as *const_ as *mut _ }
}

#[macro_export]
pub macro from_addr($place:expr) {
    unsafe { &*$place }
}

#[macro_export]
pub macro from_addr_mut($place:expr) {
    unsafe { &mut *$place }
}
