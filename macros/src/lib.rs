pub use proc::*;

#[macro_export]
macro_rules! ref_to {
    ($ty:ty, $idx:literal) => {
        crate::RelativeValue::<$ty>::new($idx)
    };
}

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\x00").as_ptr() as *const i8
    };
}

#[macro_export]
macro_rules! lua_func {
    ($type:ty, $method:path, $name:literal) => {{
        sys::luaL_Reg {
            name: cstr!($name),
            func: {
                unsafe extern "C" fn trampoline(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
                    $method(&mut State::from_raw(raw_state)) as std::ffi::c_int
                }
                Some(trampoline)
            },
        }
    }};
}

#[macro_export]
macro_rules! lua_method {
    ($type:ty, $method:path, $name:literal) => {{
        sys::luaL_Reg {
            name: cstr!($name),
            func: {
                unsafe extern "C" fn trampoline(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
                    let mut state = State::from_raw(raw_state);
                    let mut user_data = unsafe { sys::lua_touserdata(raw_state, 1) as *mut $type };

                    let mut_ref = &mut *user_data;
                    let n = $method(mut_ref, &mut state);

                    n as std::ffi::c_int
                }
                Some(trampoline)
            },
        }
    }};
}

#[macro_export]
macro_rules! lua_fn {
    // single => single
    ($value_ty:ty => $return_value_ty:ty) => {
        LuaFunction<$value_ty, $return_value_ty>
    };
    // single => many
    ($value_ty:ty => $($return_value_ty:ty,)+) => {
        LuaFunction<$value_ty, ($($return_value_ty,)*)>
    };
    // many => single
    ($($value_ty:ty,)+ => $return_value_ty:ty) => {
        LuaFunction<($($value_ty,)*), $return_value_ty>
    };
    // many => many
    ($($value_ty:ty),+ => $($return_value_ty:ty),+) => {
        LuaFunction<($($value_ty,)*), ($($return_value_ty),*)>
    };
}
