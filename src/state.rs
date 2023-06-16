#![allow(unused)]

use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    ptr,
};

use anyhow::bail;
use luajit2_sys as sys;
use values::{LuaValue, UserData, Value};

use macros::*;

pub trait GlobalConstructor {
    fn new(state: &mut State) -> Self;
}

pub struct State(*mut sys::lua_State, bool);

impl State {
    const PP: &str = include_str!("./pp.lua");

    pub fn new() -> Self {
        State(unsafe { sys::luaL_newstate() }, true)
    }

    pub fn from_raw(ptr: *mut sys::lua_State) -> Self {
        State(ptr, false)
    }

    pub fn inspect(&mut self) {
        self.get_global("pp");
        self.push_value(-2);
        self.call(1, 0);
    }

    pub fn inspect_globals(&mut self) {
        self.get_global("_G");
        self.inspect();
        self.pop(1);
    }

    pub fn dump_stack(&self) {
        let size = self.get_top();
        println!("-----------------------------------");
        println!("- Stack: {}", size);
        println!("-----------------------------------");
        for i in 1..=size {
            print!("> [{i} / -{}] ", size - i + 1);
            if self.is_number(i) {
                println!("{}", self.to_number(i).unwrap());
            } else if self.is_string(i) {
                println!("{}", self.to_string(i).unwrap());
            } else if self.is_bool(i) {
                println!("{}", self.to_bool(i).unwrap());
            } else if self.is_function(i) {
                println!("func");
            } else if self.is_table(i) {
                println!("table");
            } else if self.is_native_function(i) {
                println!("native func");
            } else if self.is_user_data(i) {
                println!("user data");
            } else if self.is_light_user_data(i) {
                println!("light user data");
            } else if self.is_coroutine(i) {
                println!("coroutine");
            } else if self.is_nil(i) {
                println!("nil");
            }
        }
        println!("-----------------------------------");
    }

    pub fn owned(&self) -> bool {
        self.1
    }

    pub fn open_libs(&mut self) {
        unsafe { sys::luaL_openlibs(self.0) }
    }

    pub fn open_pp(&mut self) {
        self.do_string(Self::PP).unwrap();
        self.pop(1);
    }

    pub fn is_user_data(&self, idx: i32) -> bool {
        unsafe { sys::lua_isuserdata(self.0, idx) != 0 }
    }

    pub fn is_light_user_data(&self, idx: i32) -> bool {
        unsafe { sys::lua_islightuserdata(self.0, idx) != 0 }
    }

    pub fn is_coroutine(&self, idx: i32) -> bool {
        unsafe { sys::lua_isthread(self.0, idx) != 0 }
    }

    pub fn is_function(&self, idx: i32) -> bool {
        unsafe { sys::lua_isfunction(self.0, idx) != 0 }
    }

    pub fn is_native_function(&self, idx: i32) -> bool {
        unsafe { sys::lua_iscfunction(self.0, idx) != 0 }
    }

    pub fn is_bool(&self, idx: i32) -> bool {
        unsafe { sys::lua_isboolean(self.0, idx) != 0 }
    }

    pub fn is_number(&self, idx: i32) -> bool {
        unsafe { sys::lua_isnumber(self.0, idx) != 0 }
    }

    pub fn is_string(&self, idx: i32) -> bool {
        unsafe { sys::lua_isstring(self.0, idx) != 0 }
    }

    pub fn is_table(&self, idx: i32) -> bool {
        unsafe { sys::lua_istable(self.0, idx) != 0 }
    }

    pub fn is_nil(&self, idx: i32) -> bool {
        unsafe { sys::lua_isnil(self.0, idx) != 0 }
    }

    pub fn to_bool(&self, idx: i32) -> Option<bool> {
        if self.is_bool(idx) {
            Some(unsafe { sys::lua_toboolean(self.0, idx) } == 1)
        } else {
            None
        }
    }

    pub fn to_number(&self, idx: i32) -> Option<f64> {
        if self.is_number(idx) {
            Some(unsafe { sys::lua_tonumber(self.0, idx) })
        } else {
            None
        }
    }

    pub fn to_string(&self, idx: i32) -> Option<&str> {
        if self.is_string(idx) {
            let ptr = unsafe { sys::lua_tostring(self.0, idx) };
            let cstr = unsafe { CStr::from_ptr(ptr) };
            cstr.to_str().ok()
        } else {
            None
        }
    }

    pub fn do_string(&mut self, code: &str) -> anyhow::Result<()> {
        let cstring = CString::new(code).unwrap();
        unsafe { sys::luaL_loadstring(self.0, cstring.as_ptr() as *const i8) };
        let result = unsafe { sys::lua_pcall(self.0, 0, sys::LUA_MULTRET, 0) };
        if result != 0 {
            let err_message = self.to_string(-1).unwrap().to_string();
            self.pop(1);
            bail!(err_message);
        } else {
            Ok(())
        }
    }

    pub fn set_top(&mut self, idx: i32) {
        unsafe { sys::lua_settop(self.0, idx) }
    }

    pub fn get_top(&self) -> i32 {
        unsafe { sys::lua_gettop(self.0) }
    }

    pub fn pop(&mut self, idx: i32) {
        unsafe { sys::lua_pop(self.0, idx) }
    }

    pub fn push(&mut self, value: impl Value) {
        match value.to_lua_value() {
            LuaValue::Integer(value) => unsafe { sys::lua_pushinteger(self.0, value) },
            LuaValue::Number(value) => unsafe { sys::lua_pushnumber(self.0, value) },
            LuaValue::Bool(value) => unsafe {
                sys::lua_pushboolean(self.0, if value { 1 } else { 0 })
            },
            LuaValue::String(cstring) => unsafe { sys::lua_pushstring(self.0, cstring.into_raw()) },
            LuaValue::Function(raw_func) => unsafe {
                sys::lua_pushcfunction(self.0, Some(raw_func))
            },
            LuaValue::UserData(opaque_ptr, size, name, methods) => unsafe {
                let managed_ptr = sys::lua_newuserdata(self.0, size);
                ptr::copy_nonoverlapping(opaque_ptr, managed_ptr, size);

                if sys::luaL_newmetatable(self.0, name as *const c_char) != 0 {
                    sys::lua_newtable(self.0);
                    sys::luaL_register(self.0, ptr::null(), methods.as_ptr());
                    sys::lua_setfield(self.0, -2, cstr!("__index"));
                }

                sys::lua_setmetatable(self.0, -2);
            },
        }
    }

    pub fn set_field(&mut self, idx: i32, name: &str) {
        let name = CString::new(name).unwrap();
        unsafe { sys::lua_setfield(self.0, idx, name.into_raw()) }
    }

    pub fn get_field(&mut self, idx: i32, name: &str) {
        let name = CString::new(name).unwrap();
        unsafe { sys::lua_getfield(self.0, idx, name.into_raw()) }
    }

    pub fn set_global(&mut self, name: &str) {
        let str = CString::new(name).unwrap();
        unsafe { sys::lua_setglobal(self.0, str.into_raw()) }
    }

    pub fn get_global(&mut self, name: &str) {
        let str = CString::new(name).unwrap();
        unsafe { sys::lua_getglobal(self.0, str.into_raw()) }
    }

    pub fn push_value(&mut self, idx: i32) {
        unsafe { sys::lua_pushvalue(self.0, idx) }
    }

    pub fn call(&mut self, nargs: i32, nresults: i32) {
        unsafe { sys::lua_call(self.0, nargs, nresults) }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if self.owned() {
            unsafe { sys::lua_close(self.0) }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_int;

    use values::RawFunction;

    use super::*;

    #[test]
    fn push_int() {
        let mut state = State::new();
        state.push(10 as i32);
        state.push(20 as i64);

        assert!(state.get_top() == 2);
        assert_eq!(state.to_number(-2).unwrap(), 10 as f64);
        assert_eq!(state.to_number(-1).unwrap(), 20 as f64);
    }

    #[test]
    fn push_float() {
        let mut state = State::new();
        state.push(10.5 as f32);
        state.push(9.8 as f64);

        assert!(state.get_top() == 2);
        assert_eq!(state.to_number(-2).unwrap(), 10.5 as f64);
        assert_eq!(state.to_number(-1).unwrap(), 9.8 as f64);
    }

    #[test]
    fn push_string() {
        let mut state = State::new();
        let name = "Soreto".to_string();

        state.push(name.as_str());
        state.push(name.clone());

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.to_string(-2).unwrap(), name.as_str());
        assert_eq!(state.to_string(-1).unwrap(), name.as_str());
    }

    #[test]
    fn push_bool() {
        let mut state = State::new();
        state.push(false);
        state.push(true);

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.to_bool(-2).unwrap(), false);
        assert_eq!(state.to_bool(-1).unwrap(), true);
    }

    #[test]
    fn push_user_data() {
        struct Math;

        impl Math {
            fn new(state: &mut State) -> usize {
                state.push(Math {});
                1
            }

            fn sum(&mut self, state: &mut State) -> usize {
                let a = state.to_number(-2).unwrap();
                let b = state.to_number(-1).unwrap();

                state.push(a + b);
                1
            }
        }

        impl UserData for Math {
            fn name() -> *const i8 {
                cstr!("Math")
            }

            fn functions() -> Vec<sys::luaL_Reg> {
                vec![
                    lua_func!(Math, Math::new, "new"),
                    lua_method!(Math, Math::sum, "sum"),
                ]
            }
        }

        let mut state = State::new();
        state.push(Math {});

        assert_eq!(state.get_top(), 1);
        assert!(state.is_user_data(-1));

        state.set_global("math");
        assert_eq!(state.get_top(), 0);

        state.get_global("math");
        assert_eq!(state.get_top(), 1);

        state.get_field(-1, "sum");
        assert_eq!(state.get_top(), 2);

        state.push_value(-2);
        state.push(10);
        state.push(12.0);
        state.call(3, 1);
        assert_eq!(state.get_top(), 2);
        assert_eq!(state.to_number(-1).unwrap(), 22.0);

        state.dump_stack();
    }

    #[test]
    fn proc_macros() {
        // #[derive(UserData)]
        struct Test {
            a: usize,
        }

        #[user_data]
        impl Test {
            #[ctor]
            fn new() -> Test {
                Test { a: 10 }
            }

            #[method]
            fn foo(&self, a: f64, b: f64) -> f64 {
                a + b
            }
        }

        // let mut state = State::new();
        // state.push(Test::new());
        // state.set_global("test");
        // state.do_string("test:foo(2, 3)");
    }
}
