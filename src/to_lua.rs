use luajit2_sys as sys;
use macros::{cstr, generate_to_lua_tuple_impl};
use std::{
    ffi::{c_void, CString},
    mem::size_of,
};

use crate::{state::State, RawFunction, RelativeValue, UserData};

pub trait ToLua {
    fn to_lua(self, state: *mut sys::lua_State);
    fn len() -> i32 {
        1
    }
}

impl<T> ToLua for RelativeValue<T> {
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushvalue(state, self.0) }
    }
}

impl ToLua for i32 {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushinteger(state, self as isize) }
    }
}

impl ToLua for i64 {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushinteger(state, self as isize) }
    }
}

impl ToLua for f32 {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushnumber(state, self as f64) }
    }
}

impl ToLua for f64 {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushnumber(state, self) }
    }
}

impl ToLua for bool {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushboolean(state, if self { 1 } else { 0 }) }
    }
}

impl ToLua for &str {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        #[allow(temporary_cstring_as_ptr)]
        unsafe {
            sys::lua_pushstring(state, CString::new(self).unwrap().as_ptr())
        }
    }
}

impl ToLua for String {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        #[allow(temporary_cstring_as_ptr)]
        unsafe {
            sys::lua_pushstring(state, CString::new(self).unwrap().as_ptr())
        }
    }
}

impl ToLua for RawFunction {
    #[inline]
    fn to_lua(self, state: *mut sys::lua_State) {
        unsafe { sys::lua_pushcfunction(state, Some(self)) }
    }
}

impl<T: UserData> ToLua for T {
    fn to_lua(self, state: *mut sys::lua_State) {
        let size = size_of::<T>();
        let name = T::name();
        let methods = T::functions();
        let ptr = Box::into_raw(Box::new(self));

        unsafe {
            let managed_ptr = sys::lua_newuserdata(state, size);
            std::ptr::copy_nonoverlapping(ptr as *mut c_void, managed_ptr, size);

            if sys::luaL_newmetatable(state, name) != 0 {
                sys::lua_newtable(state);
                sys::luaL_register(state, std::ptr::null(), methods.as_ptr());
                sys::lua_setfield(state, -2, cstr!("__index"));
            }

            sys::lua_setmetatable(state, -2);
        }
    }
}

generate_to_lua_tuple_impl!(25);
