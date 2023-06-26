use std::ffi::CString;

use luajit2_sys as sys;

use crate::{
    from_lua::FromLua, is_type::IsType, to_lua::ToLua, AnyLuaFunction, AnyUserData, Coroutine,
    LightUserData, NativeFunction, Table,
};

pub struct State(*mut sys::lua_State, bool);

impl State {
    const PP: &str = include_str!("./pp.lua");

    pub fn new() -> Self {
        State(unsafe { sys::luaL_newstate() }, true)
    }

    pub fn from_raw(ptr: *mut sys::lua_State) -> Self {
        State(ptr, false)
    }

    pub fn dump_stack(&self) {
        let size = self.get_top();
        println!("-----------------------------------");
        println!("- Stack: {}", size);
        println!("-----------------------------------");
        for i in 1..=size {
            print!("> [{i} / -{}] ", size - i + 1);
            if self.is::<f64>(i) {
                println!("{}", self.cast_to::<f64>(i).unwrap());
            } else if self.is::<&str>(i) {
                println!("{}", self.cast_to::<&str>(i).unwrap());
            } else if self.is::<bool>(i) {
                println!("{}", self.cast_to::<bool>(i).unwrap());
            } else if self.is::<AnyLuaFunction>(i) {
                println!("func");
            } else if self.is::<Table>(i) {
                println!("table");
            } else if self.is::<NativeFunction>(i) {
                println!("native func");
            } else if self.is::<AnyUserData>(i) {
                println!("user data");
            } else if self.is::<LightUserData>(i) {
                println!("light user data");
            } else if self.is::<Coroutine>(i) {
                println!("coroutine");
            } else if self.is::<()>(i) {
                println!("nil");
            }
        }
        println!("-----------------------------------");
    }

    pub(crate) fn owned(&self) -> bool {
        self.1
    }

    pub fn open_libs(&self) {
        unsafe { sys::luaL_openlibs(self.0) }
    }

    pub fn open_pp(&self) {
        self.do_string(Self::PP).unwrap();
        self.pop(1);
    }

    pub fn is<T: IsType>(&self, idx: i32) -> bool {
        T::is_type(self.0, idx)
    }

    pub fn do_string(&self, code: &str) -> Result<(), &str> {
        let cstring = CString::new(code).unwrap();
        unsafe { sys::luaL_loadstring(self.0, cstring.as_ptr() as *const i8) };
        if unsafe { sys::lua_pcall(self.0, 0, sys::LUA_MULTRET, 0) } != 0 {
            Err(self.cast_to::<&str>(-1).unwrap())
        } else {
            Ok(())
        }
    }

    pub fn set_top(&self, idx: i32) {
        unsafe { sys::lua_settop(self.0, idx) }
    }

    pub fn get_top(&self) -> i32 {
        unsafe { sys::lua_gettop(self.0) }
    }

    pub fn pop(&self, idx: i32) {
        unsafe { sys::lua_pop(self.0, idx) }
    }

    pub fn push(&self, value: impl ToLua) {
        value.to_lua(self.0);
    }

    pub fn set_field(&self, idx: i32, name: &str, value: impl ToLua) {
        self.push(value);
        let name = CString::new(name).unwrap();
        unsafe { sys::lua_setfield(self.0, idx, name.into_raw()) }
    }

    pub fn get_field<'a, A: FromLua<'a>>(&self, idx: i32, name: &str) -> Option<A::Output> {
        let name = CString::new(name).unwrap();
        unsafe { sys::lua_getfield(self.0, idx, name.into_raw()) };
        A::from_lua(self.0, idx)
    }

    pub fn set_global(&self, name: &str, value: impl ToLua) {
        self.push(value);
        let str = CString::new(name).unwrap();
        unsafe { sys::lua_setglobal(self.0, str.into_raw()) }
    }

    pub fn get_global<'a, T: FromLua<'a>>(&self, name: &str) -> Option<T::Output> {
        let str = CString::new(name).unwrap();
        unsafe { sys::lua_getglobal(self.0, str.into_raw()) };
        self.cast_to::<T>(-1)
    }

    pub fn cast_to<'a, T: FromLua<'a>>(&self, idx: i32) -> Option<T::Output> {
        T::from_lua(self.0, idx)
    }

    pub fn protected_call<'a, A: ToLua, B: FromLua<'a>>(&self, args: A) -> Result<B::Output, &str> {
        self.push(args);
        if unsafe { sys::lua_pcall(self.0, A::len(), B::len(), 0) } != 0 {
            Err(self.cast_to::<&str>(-1).unwrap())
        } else {
            if let Some(v) = B::from_lua(self.0, B::len() * -1) {
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
    use macros::{cstr, lua_func, lua_method, ref_to, user_data};

    use crate::{LuaFunction, RelativeValue, UserData};

    use super::*;

    macro_rules! from_ptr {
        ($name:expr) => {
            unsafe { std::ffi::CStr::from_ptr($name) }.to_str().unwrap()
        };
    }

    #[test]
    fn push_int() {
        let state = State::new();
        state.push(10 as i32);
        state.push(20 as i64);

        assert!(state.get_top() == 2);
        assert_eq!(state.cast_to::<i32>(-2).unwrap(), 10);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 20.0);
    }

    #[test]
    fn push_float() {
        let state = State::new();
        state.push(10.5 as f32);
        state.push(9.8 as f64);

        assert!(state.get_top() == 2);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 10.5 as f64);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 9.8 as f64);
    }

    #[test]
    fn push_string() {
        let state = State::new();
        let name = "Soreto".to_string();

        state.push(name.as_str());
        state.push(name.clone());

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<&str>(-2).unwrap(), name.as_str());
        assert_eq!(state.cast_to::<&str>(-1).unwrap(), name.as_str());
    }

    #[test]
    fn protected_call_with_single_return_arg() {
        let state = State::new();
        state
            .do_string("function sum(a, b) return a + b end")
            .unwrap();
        state.get_global::<LuaFunction<(i32, i32), i32>>("sum");

        let result = state.protected_call::<_, i32>((2, 3));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn calling_lua_from_rust_multiple_return() {
        let state = State::new();
        state
            .do_string("function double(a, b) return a * 2, b * 2 end")
            .unwrap();

        let option = state.get_global::<LuaFunction<(i32, i32), (i32, i32)>>("double");
        assert!(option.is_some());

        let double = option.unwrap();
        let result = double((4, 8));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (8, 16));
    }

    #[test]
    fn calling_lua_from_rust_single_return() {
        let state = State::new();
        state
            .do_string("function double(a) return a * 2 end")
            .unwrap();

        let double = state.get_global::<LuaFunction<i32, i32>>("double").unwrap();
        let result = double(10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);
    }

    #[test]
    fn push_bool() {
        let state = State::new();
        state.push(false);
        state.push(true);

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<bool>(-2).unwrap(), false);
        assert_eq!(state.cast_to::<bool>(-1).unwrap(), true);
    }

    #[test]
    fn push_tuple_with_different_types() {
        let state = State::new();
        state.push((10, false, "soreto"));

        assert_eq!(state.get_top(), 3);
        assert_eq!(state.cast_to::<f64>(-3).unwrap(), 10.0);
        assert_eq!(state.cast_to::<bool>(-2).unwrap(), false);
        assert_eq!(state.cast_to::<&str>(-1).unwrap(), "soreto");
    }

    #[test]
    fn push_tuples_2() {
        let state = State::new();
        state.push((10, 20));

        assert_eq!(state.get_top(), 2);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 10.0);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 20.0);
    }

    #[test]
    fn push_tuples_3() {
        let state = State::new();
        state.push((10, 20, 30));

        assert_eq!(state.get_top(), 3);
        assert_eq!(state.cast_to::<f64>(-3).unwrap(), 10.0);
        assert_eq!(state.cast_to::<f64>(-2).unwrap(), 20.0);
        assert_eq!(state.cast_to::<f64>(-1).unwrap(), 30.0);
    }

    #[test]
    fn tuple_from_lua() {
        let value = (10, 20, 30);
        let state = State::new();
        state.push(value);

        let result = state.cast_to::<(i32, i32, i32)>(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn mixed_tuple_from_lua() {
        let value = (10, false, "soreto");
        let state = State::new();
        state.push(value);

        let result = state.cast_to::<(i32, bool, &str)>(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn take_user_data_ref_from_stack() {
        struct Test;

        #[user_data]
        impl Test {
            pub fn foo(&mut self, state: &State) -> i32 {
                state.push(10);
                1
            }

            fn bar(&self) -> i32 {
                123
            }
        }

        let state = State::new();
        state.push(Test {});

        let result = state.cast_to::<&Test>(-1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().bar(), 123);
    }

    #[test]
    fn push_user_data() {
        struct Math;

        impl Math {
            fn new(state: &State) -> usize {
                state.push(Math {});
                1
            }

            fn sum(&mut self, state: &State) -> usize {
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

        let state = State::new();
        state.push(Math {});

        assert_eq!(state.get_top(), 1);
        assert!(state.is::<Math>(-1));

        let option = state.get_field::<LuaFunction<(f64, f64), f64>>(-1, "sum");
        assert!(option.is_some());

        let sum = option.unwrap();
        let result = sum((10.0, 12.0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 22.0);
    }

    #[test]
    fn proc_macro_pub_raw_function() {
        struct Test {}

        #[user_data]
        impl Test {
            pub fn foo(&mut self, state: &State) -> i32 {
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

        let state = State::new();
        state.push(Test {});
        let option =
            state.get_field::<LuaFunction<(RelativeValue<Test>, i32, i32), (bool, f32)>>(-1, "foo");
        assert!(option.is_some());

        let foo = option.unwrap();
        let result = foo((ref_to!(Test, -2), 2, 3));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (true, 5.0));
    }

    #[test]
    fn proc_macro_pub_function() {}

    #[test]
    fn proc_macro_pub_method_manual_impl() {
        struct Test {
            a: i32,
        }

        impl Test {
            pub fn foo(&self, b: i32, c: i32) -> i32 {
                self.a + b + c
            }
        }

        impl UserData for Test {
            fn name() -> *const i8 {
                cstr!("Test")
            }

            fn functions() -> Vec<sys::luaL_Reg> {
                vec![{
                    type Args = (i32, i32);
                    extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                        let state = State::from_raw(ptr);
                        let len = <Args as FromLua>::len() + 1;
                        let idx = len * -1;

                        let ud = state.cast_to::<&Test>(idx).unwrap();
                        let args = state.cast_to::<Args>(idx + 1).unwrap();
                        let result = ud.foo(args.0, args.1);

                        state.push(result);
                        <i32 as ToLua>::len() as std::ffi::c_int
                    }
                    luajit2_sys::luaL_Reg {
                        name: cstr!("foo"),
                        func: Some(step),
                    }
                }]
            }
        }

        let state = State::new();
        state.push(Test { a: 2 });
        let option =
            state.get_field::<LuaFunction<(RelativeValue<Test>, i32, i32), i32>>(-1, "foo");
        assert!(option.is_some());

        let foo = option.unwrap();
        let result = foo((ref_to!(Test, -2), 2, 3));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2 + 2 + 3);
    }

    #[test]
    fn proc_macro_pub_method() {
        // struct Test;

        // #[user_data]
        // impl Test {
        //     pub fn foo(&self, a: i32, b: i32) -> i32 {
        //         a + b
        //     }
        // }

        // let state = State::new();
        // state.push(Test {});
        // state.get_field(-1, "foo");

        // let result = state.protected_call::<_, i32>((ref_to!(Test, -2), 2, 3));
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), 2 + 3);
    }
}
