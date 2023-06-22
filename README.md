# lofy
High level LuaJIT wrapper for Rust. Compile time magic to bridge Lua <-> Rust interactions with no additional runtime cost.

## API changes
`lua_to*` -> `state.cast_to::<T>(idx)`
`lua_is*` -> `state.is::<T>(idx)`
`lua_pcall` -> `state.protected_call::<T: ToLua, B: FromLua>(args: A)

## Working with multiple values
Tuples implement `ToLua` and `FromLua`. So you can use it to represent multiple values in lua.
```rust
// this
state.push(10);
state.push(20);
state.push(30);

// can be written like this
state.push((10, 20, 30));

// you can also use different types and even UserData
state.push((10, false, "foo", user_data));
```
You can also return multiple values from a native function:
```rust
struct Math;

#[user_data]
impl Math {
	pub fn sum(&self, a: i32, b: i32) -> i32 {
		a + b
	}

	pub fn mul_both(&self, a: i32, b: i32) -> (i32, i32) {
		(a * 2, b * 2)
	}
}

let mut state = State::new();
state.push(Math {});
state.get_field(-1, "mul_both");

let result = state.protected_call::<_, (i32, i32)>((10, 20));
assert!(result.is_ok());

let (a, b) = result.unwrap();
assert_eq!(a, 20);
assert_eq!(b, 40);
```

## UserData
There is a `#[user_data]` macro available, it will do all the necessary magic for you. You can write four kinds of functions:
- raw instance: `(&self, &mut State) -> i32` or `(&mut self, &mut State) -> i32`
- raw static: `(&mut State) -> i32`
- instance: `(&self, a: i32, b: i32) -> bool`, function args and return values are converted automatically. You can use any value that implements `ToLua` and `FromLua`.

```rust
struct Test { ratio: f32 }

#[user_data]
impl Test {
	// lua: test.static_function(10, 20) == 30
	pub fn static_function(a: i32, b: i32) -> i32 {
		a + b
	}

	// lua: test:instance_method(3, 9) == 3
	pub fn instance_method(&self, a: f32, b: f32) -> f32 {
		b / a * sellf.ratio
	}

	// lua: test.raw_static(false)
	pub fn raw_static(state: &mut State) -> i32 {
		let value = state.cast_to::<bool>(-1).unwrap();
		state.push(!value);
		1
	}

	// lua: test.raw_mut_instance
	pub fn raw_mut_instance(&mut self, state: &mut State) -> i32 {
		todo!()
	}

	// private functions are ignored
	fn useless_private_func(&self) {}

	// async functions not supported yet
	async fn this_will_panic(&self) {}
}
```
