use std::ffi::CStr;

use crate::{state::State, RelativeValue, UserData};
use luajit2_sys as sys;
use macros::generate_from_lua_tuple_impl;

pub trait FromLua<'a> {
    type Output;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output>;

    fn len() -> i32 {
        1
    }
}

impl<'a, T> FromLua<'a> for RelativeValue<T>
where
    T: FromLua<'a> + UserData,
{
    type Output = T::Output;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        let state = State::from_raw(ptr);
        if state.is::<T>(idx) {
            T::from_lua(ptr, idx)
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for i32 {
    type Output = i32;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isnumber(ptr, idx) != 0 } {
            Some(unsafe { sys::lua_tonumber(ptr, idx) } as i32)
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for f32 {
    type Output = f32;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isnumber(ptr, idx) != 0 } {
            Some(unsafe { sys::lua_tonumber(ptr, idx) } as f32)
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for f64 {
    type Output = f64;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isnumber(ptr, idx) != 0 } {
            Some(unsafe { sys::lua_tonumber(ptr, idx) })
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for bool {
    type Output = bool;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isboolean(ptr, idx) != 0 } {
            Some(unsafe { sys::lua_toboolean(ptr, idx) != 0 })
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for &str {
    type Output = &'a str;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isstring(ptr, idx) != 0 } {
            let ptr = unsafe { sys::lua_tostring(ptr, idx) };
            let cstr = unsafe { CStr::from_ptr(ptr) };
            cstr.to_str().ok()
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for String {
    type Output = String;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isstring(ptr, idx) != 0 } {
            let ptr = unsafe { sys::lua_tostring(ptr, idx) };
            let cstr = unsafe { CStr::from_ptr(ptr) };
            Some(cstr.to_str().ok()?.to_string())
        } else {
            None
        }
    }
}

impl<'a, T: UserData + 'a> FromLua<'a> for T {
    type Output = &'a T;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isuserdata(ptr, idx) != 0 } {
            let ptr = unsafe { sys::lua_touserdata(ptr, idx) };
            let aligned = ptr.cast() as *mut T;
            unsafe { aligned.as_ref() }
        } else {
            None
        }
    }
}

generate_from_lua_tuple_impl!();
