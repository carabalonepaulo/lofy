use lofy::state::State;

fn main() {
    let state = State::new();
    state.open_libs();
    state.do_string("print 'hello world'").unwrap();
}
