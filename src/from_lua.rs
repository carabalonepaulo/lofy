use std::ffi::CStr;

use crate::{state::State, UserData};
use luajit2_sys as sys;
use macros::generate_from_lua_tuple_impl;

pub trait FromLua<'a> {
    type Output;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output>;

    fn len() -> i32 {
        1
    }
}

impl<'a> FromLua<'a> for i32 {
    type Output = i32;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        let ptr = state.as_ptr();
        if state.is::<i32>(idx) {
            Some(unsafe { sys::lua_tonumber(ptr, idx) } as i32)
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for f32 {
    type Output = f32;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        let ptr = state.as_ptr();
        if state.is::<f32>(idx) {
            Some(unsafe { sys::lua_tonumber(ptr, idx) } as f32)
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for f64 {
    type Output = f64;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        let ptr = state.as_ptr();
        if state.is::<f64>(idx) {
            Some(unsafe { sys::lua_tonumber(ptr, idx) })
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for bool {
    type Output = bool;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        let ptr = state.as_ptr();
        if state.is::<bool>(idx) {
            Some(unsafe { sys::lua_toboolean(ptr, idx) != 0 })
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for &str {
    type Output = &'a str;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        if state.is::<&str>(idx) {
            let ptr = unsafe { sys::lua_tostring(state.as_ptr(), idx) };
            let cstr = unsafe { CStr::from_ptr(ptr) };
            cstr.to_str().ok()
        } else {
            None
        }
    }
}

impl<'a> FromLua<'a> for String {
    type Output = String;

    fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
        if state.is::<&str>(idx) {
            let ptr = unsafe { sys::lua_tostring(state.as_ptr(), idx) };
            let cstr = unsafe { CStr::from_ptr(ptr) };
            Some(cstr.to_str().ok()?.to_string())
        } else {
            None
        }
    }
}

impl<'a, T: UserData + 'a> FromLua<'a> for T {
    type Output = &'a T;

    fn from_lua(_state: &State, _idx: i32) -> Option<Self::Output> {
        todo!()
    }
}

generate_from_lua_tuple_impl!();
