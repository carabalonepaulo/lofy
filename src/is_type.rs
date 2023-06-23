use crate::{AnyUserData, Coroutine, LightUserData, LuaFunction, NativeFunction, Table, UserData};
use luajit2_sys as sys;

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
        let ptr = ptr;
        if unsafe { sys::lua_isuserdata(ptr, idx) == 0 } {
            return false;
        }

        // if unsafe { sys::lua_getmetatable(ptr, idx) == 0 } {
        //     return false;
        // }
        //
        // unsafe { sys::lua_getfield(ptr, -1, cstr!("__name")) }
        // if unsafe { sys::lua_isnil(ptr, -1) == 1 } {
        //     unsafe { sys::lua_pop(ptr, 1) };
        //     return false;
        // }
        //
        // if T::name() == unsafe { sys::lua_tostring(ptr, -1) } {
        //     unsafe { sys::lua_pop(ptr, 2) };
        //     return true;
        // }

        false
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
