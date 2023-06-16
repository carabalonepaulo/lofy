use luajit2_sys as sys;
use std::{
    ffi::{c_int, c_void, CString},
    mem::size_of,
};

pub enum LuaValue {
    Integer(isize),
    Number(f64),
    Bool(bool),
    String(CString),
    Function(RawFunction),
    UserData(*mut c_void, usize, *const i8, Vec<sys::luaL_Reg>),
}

pub type RawFunction = unsafe extern "C" fn(state: *mut sys::lua_State) -> c_int;

pub trait UserData {
    fn name() -> *const i8;
    fn functions() -> Vec<sys::luaL_Reg>;
}

pub trait MetaTable {
    fn index(&mut self) {}
    fn new_index(&mut self) {}
    fn mode(&mut self) {}
    fn call(&mut self) {}
    fn metatable(&mut self) {}
    fn to_string(&mut self) {}
    fn gc(&mut self) {}
    fn name(&mut self) {}
    //
    fn unary_minus(&mut self) {}
    fn add(&mut self) {}
    fn sub(&mut self) {}
    fn mul(&mut self) {}
    fn div(&mut self) {}
    fn modulo(&mut self) {}
    fn pow(&mut self) {}
    fn concat(&mut self) {}
    //
    fn eq(&mut self) {}
    fn lt(&mut self) {}
    fn le(&mut self) {}
}

pub trait Value {
    fn to_lua_value(self) -> LuaValue;
}

impl Value for i32 {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Integer(self as isize)
    }
}

impl Value for i64 {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Integer(self as isize)
    }
}

impl Value for f32 {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Number(self as f64)
    }
}

impl Value for f64 {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Number(self)
    }
}

impl Value for bool {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Bool(self)
    }
}

impl Value for *const i8 {
    fn to_lua_value(self) -> LuaValue {
        unsafe { LuaValue::String(CString::from_raw(self as *mut i8)) }
    }
}

impl Value for *mut i8 {
    fn to_lua_value(self) -> LuaValue {
        unsafe { LuaValue::String(CString::from_raw(self)) }
    }
}

impl Value for &str {
    fn to_lua_value(self) -> LuaValue {
        let str = CString::new(self).unwrap();
        LuaValue::String(str)
    }
}

impl Value for String {
    fn to_lua_value(self) -> LuaValue {
        let str = CString::new(self.as_str()).unwrap();
        LuaValue::String(str)
    }
}

impl Value for RawFunction {
    fn to_lua_value(self) -> LuaValue {
        LuaValue::Function(self)
    }
}

impl<T: UserData> Value for T {
    fn to_lua_value(self) -> LuaValue {
        let size = size_of::<T>();
        let name = T::name();
        let methods = T::functions();
        let ptr = Box::into_raw(Box::new(self)) as *mut c_void;
        LuaValue::UserData(ptr, size, name, methods)
    }
}
