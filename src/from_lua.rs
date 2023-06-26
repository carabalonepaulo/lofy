use std::ffi::CStr;

use crate::{state::State, to_lua::ToLua, LuaFunction, RelativeValue, UserData};
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

impl<'a> FromLua<'a> for () {
    type Output = ();

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isnil(ptr, idx) != 0 } {
            Some(())
        } else {
            None
        }
    }
}

impl<'a, T: UserData + 'a> FromLua<'a> for &'a T {
    type Output = &'a T;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isuserdata(ptr, idx) != 0 } {
            let ud_ptr = unsafe { sys::lua_touserdata(ptr, idx) };
            unsafe { (ud_ptr as *const T).as_ref() }
        } else {
            None
        }
    }
}

impl<'a, T: UserData + 'a> FromLua<'a> for &'a mut T {
    type Output = &'a mut T;

    fn from_lua(ptr: *mut sys::lua_State, idx: i32) -> Option<Self::Output> {
        if unsafe { sys::lua_isuserdata(ptr, idx) != 0 } {
            let ud_ptr = unsafe { sys::lua_touserdata(ptr, idx) };
            unsafe { (ud_ptr as *mut T).as_mut() }
        } else {
            None
        }
    }
}

impl<'a, A, B> FromLua<'a> for LuaFunction<'a, A, B>
where
    A: ToLua,
    B: FromLua<'a>,
{
    type Output = Box<dyn Fn(A) -> Result<B::Output, &'a str>>;

    fn from_lua(ptr: *mut sys::lua_State, _: i32) -> Option<Self::Output> {
        Some(Box::new(move |args: A| {
            A::to_lua(args, ptr);
            if unsafe { sys::lua_pcall(ptr, A::len(), B::len(), 0) } != 0 {
                let msg = <&str as FromLua<'a>>::from_lua(ptr, -1).unwrap();
                Err(msg)
            } else {
                if let Some(value) = B::from_lua(ptr, B::len() * -1) {
                    Ok(value)
                } else {
                    Err("Failed to cast output.")
                }
            }
        }))
    }
}

generate_from_lua_tuple_impl!();
