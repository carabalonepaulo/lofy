use std::marker::PhantomData;

mod from_lua;
mod is_type;
pub mod state;
mod to_lua;

pub type RawFunction = unsafe extern "C" fn(state: *mut luajit2_sys::lua_State) -> std::ffi::c_int;

pub struct NativeFunction;

pub struct LuaFunction;

pub struct AnyUserData;

pub struct LightUserData;

pub struct Coroutine;

pub struct Table;

pub struct RelativeValue<T>(i32, PhantomData<T>);

impl<T> RelativeValue<T> {
    pub fn new(idx: i32) -> Self {
        Self(idx, PhantomData::<T>)
    }
}

pub trait UserData {
    fn name() -> *const i8;
    fn functions() -> Vec<luajit2_sys::luaL_Reg>;
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
