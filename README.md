
# spyglass

a small scoped timer helper library.

```
[dependencies]
spyglass = { git = "https://github.com/zphixon/spyglass" }
```

e.g.

```rust
impl Drop for MyTimer {
    fn drop(&mut self) {
        let timing = self.make_timing();
        std::thread::spawn(move || GLOBAL_TIMER.queue(timing));
    }
}

spyglass::lazy_static! {
    static ref GLOBAL_TIMER: Timer = Timer::new();
}
```

for a more complete example, see `examples/my_timer.rs`

## caveats

when creating a scoped timer, you need to make sure that it doesn't get dropped
before the end of whatever scope you're actually trying to benchmark.

```rust
// !!! these will get dropped immediately !!!
MyTimer::new(t!());
let _ = MyTimer::new(t!());

// this is OK
let _timer = MyTimer::new(t!());
```
