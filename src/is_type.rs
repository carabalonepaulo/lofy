use crate::{AnyUserData, Coroutine, LightUserData, LuaFunction, NativeFunction, Table, UserData};
use luajit2_sys as sys;
use macros::cstr;

pub trait IsType {
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool;
}

impl IsType for i32 {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isnumber(ptr, idx) != 0 }
    }
}

impl IsType for f32 {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isnumber(ptr, idx) != 0 }
    }
}

impl IsType for f64 {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isnumber(ptr, idx) != 0 }
    }
}

impl IsType for bool {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isboolean(ptr, idx) != 0 }
    }
}

impl IsType for &str {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isstring(ptr, idx) != 0 }
    }
}

impl IsType for String {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isstring(ptr, idx) != 0 }
    }
}

impl<T: UserData> IsType for T {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isuserdata(ptr, idx) != 0 }
    }
}

impl IsType for NativeFunction {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_iscfunction(ptr, idx) != 0 }
    }
}

impl IsType for LuaFunction {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isfunction(ptr, idx) != 0 }
    }
}

impl IsType for AnyUserData {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isuserdata(ptr, idx) != 0 }
    }
}

impl IsType for LightUserData {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_islightuserdata(ptr, idx) != 0 }
    }
}

impl IsType for Coroutine {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isthread(ptr, idx) != 0 }
    }
}

impl IsType for Table {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_istable(ptr, idx) != 0 }
    }
}

impl IsType for () {
    #[inline]
    fn is_type(ptr: *mut sys::lua_State, idx: i32) -> bool {
        unsafe { sys::lua_isnil(ptr, idx) != 0 }
    }
}
