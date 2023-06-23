use std::ffi::CString;

use luajit2_sys as sys;

use crate::{from_lua::FromLua, is_type::IsType, to_lua::ToLua};

pub struct State(*mut sys::lua_State, bool);

impl Clone for State {
    fn clone(&self) -> Self {
        Self::from_raw(self.0)
    }
}

impl State {
    const PP: &str = include_str!("./pp.lua");

    pub fn new() -> Self {
        State(unsafe { sys::luaL_newstate() }, true)
    }

    pub fn from_raw(ptr: *mut sys::lua_State) -> Self {
        State(ptr, false)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut sys::lua_State {
        self.0
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

    pub fn is<T: IsType>(&self, idx: i32) -> bool {
        T::is_type(self, idx)
    }

    pub fn do_string(&mut self, code: &str) -> Result<(), &str> {
        let cstring = CString::new(code).unwrap();
        unsafe { sys::luaL_loadstring(self.0, cstring.as_ptr() as *const i8) };
        if unsafe { sys::lua_pcall(self.0, 0, sys::LUA_MULTRET, 0) } != 0 {
            Err(self.cast_to::<&str>(-1).unwrap())
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

    pub fn push(&mut self, value: impl ToLua) {
        value.to_lua(self.0);
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

    pub fn pcall(&mut self, nargs: i32, nresults: i32) -> Result<(), &str> {
        if unsafe { sys::lua_pcall(self.0, nargs, nresults, 0) } != 0 {
            Err(self.cast_to::<&str>(-1).unwrap())
        } else {
            Ok(())
        }
    }

    pub fn cast_to<'a, T: FromLua<'a>>(&mut self, idx: i32) -> Option<T::Output> {
        T::from_lua(self, idx)
    }

    pub fn protected_call<'a, A: ToLua, B: FromLua<'a>>(
        &mut self,
        args: A,
    ) -> Result<B::Output, &str> {
        self.push(args);
        if unsafe { sys::lua_pcall(self.0, A::len(), B::len(), 0) } != 0 {
            Err(self.cast_to::<&str>(-1).unwrap())
        } else {
            if let Some(v) = B::from_lua(self, -1) {
                Ok(v)
            } else {
                Err("Failed to cast output.")
            }
        }
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
pub mod tests {
    use macros::{cstr, lua_func, lua_method, user_data};

    use crate::UserData;

    use super::*;

    macro_rules! from_ptr {
        ($name:expr) => {
            unsafe { std::ffi::CStr::from_ptr($name) }.to_str().unwrap()
        };
    }

    #[test]
    fn push_int() {
        let mut state = State::new();
        state.push(10 as i32);
        state.push(20 as i64);

        assert!(state.get_top() == 2);
        assert_eq!(state.cast_to::<i32>(-2).unwrap(), 10);
        // assert_eq!(state.cast_to::<f64>(-1).unwrap(), 20.0);
    }

    #[test]
    fn push_float() {
        let mut state = State::new();
        state.push(10.5 as f32);
        state.push(9.8 as f64);

        assert!(state.get_top() == 2);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 10.5 as f64);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 9.8 as f64);
    }

    #[test]
    fn push_string() {
        let mut state = State::new();
        let name = "Soreto".to_string();

        state.push(name.as_str());
        state.push(name.clone());

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<&str>(-2).unwrap(), name.as_str());
        assert_eq!(state.cast_to::<&str>(-1).unwrap(), name.as_str());
    }

    #[test]
    fn protected_call_with_single_return_arg() {
        let mut state = State::new();
        state
            .do_string("function sum(a, b) return a + b end")
            .unwrap();
        state.get_global("sum");

        let result = state.protected_call::<_, i32>((2, 3));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn protected_call_with_multi_return() {
        let mut state = State::new();
        state
            .do_string("function double(a, b) return a * 2, b * 2 end")
            .unwrap();
        state.get_global("double");

        // let result = state.protected_call::<_, (i32, i32)>((4, 8));
        // assert!(result.is_ok());
    }

    #[test]
    fn push_bool() {
        let mut state = State::new();
        state.push(false);
        state.push(true);

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<bool>(-2).unwrap(), false);
        assert_eq!(state.cast_to::<bool>(-1).unwrap(), true);
    }

    #[test]
    fn push_tuple_with_different_types() {
        let mut state = State::new();
        state.push((10, false, "soreto"));

        assert_eq!(state.get_top(), 3);
        assert_eq!(state.cast_to::<f64>(-3).unwrap(), 10.0);
        assert_eq!(state.cast_to::<bool>(-2).unwrap(), false);
        assert_eq!(state.cast_to::<&str>(-1).unwrap(), "soreto");
    }

    #[test]
    fn push_tuples_2() {
        let mut state = State::new();
        state.push((10, 20));

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 10.0);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 20.0);
    }

    #[test]
    fn push_tuples_3() {
        let mut state = State::new();
        state.push((10, 20, 30));

        assert_eq!(state.get_top(), 3);
        assert_eq!(state.cast_to::<f64>(-3).unwrap(), 10.0);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 20.0);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 30.0);
    }

    #[test]
    fn tuple_from_lua() {
        let value = (10, 20, 30);
        let mut state = State::new();
        state.push(value);

        let result = state.cast_to::<(i32, i32, i32)>(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn mixed_tuple_from_lua() {
        let value = (10, false, "soreto");
        let mut state = State::new();
        state.push(value);

        let result = state.cast_to::<(i32, bool, &str)>(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
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
                let a = state.cast_to::<f64>(-2).unwrap();
                let b = state.cast_to::<f64>(-1).unwrap();

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
        assert!(state.is::<Math>(-1));

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
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 22.0);
    }

    #[test]
    fn proc_macro_pub_raw_function() {
        #[allow(dead_code)]
        struct Test {
            a: usize,
        }

        #[user_data]
        impl Test {
            pub fn foo(&mut self, state: &mut State) -> i32 {
                let is_user_data = state.is::<Test>(1);
                state.push(is_user_data);

                let a = state.cast_to::<f64>(2).unwrap_or(0.0);
                let b = state.cast_to::<f64>(3).unwrap_or(0.0);
                state.push(a + b);

                2
            }
        }

        let slice = from_ptr!(<Test as UserData>::name());
        assert_eq!(slice, "Test");

        let funcs = <Test as UserData>::functions();
        assert!(funcs.len() > 0);

        let func_name = from_ptr!(funcs[0].name);
        assert_eq!(func_name, "foo");

        let mut state = State::new();
        state.push(Test { a: 10 });
        state.get_field(-1, "foo");
        state.push_value(-2);
        state.push(2);
        state.push(3);

        // let result = state.protected_call::<f64>(3, 2);
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), 5);

        // assert_eq!(state.get_top(), 3);
        // assert_eq!(state.to_bool(-2).unwrap(), true);
        // assert_eq!(state.to_number(-1).unwrap(), 5.);

        // dbg!(state.stack());
    }

    #[test]
    fn proc_macro_raw_static_functions() {}

    #[test]
    fn proc_macro_wrapped_function() {}

    #[test]
    fn proc_macro_wrapped_static_function() {}

    #[test]
    fn return_multiple_values_using_tuple() {}
}
