# Cad Frontend

## Roadmap
- [x] Dynamic panes
- [ ] Editing sketches
  - [x] Fix Text rendering grainy renders
  - [x] Make sure Point mode is enabled
  - [ ] Line mode
  - [ ] Circle mode
  - [ ] Show pending shapes

## Towards editing sketches
- [x] Sprite renderer
    - [x] Sprite atlas for icons
- [x] Moving the camera to face the sketch
    - [x] Cross product of x and y for normal vector
- [x] Entity picker shader
    - [x] Framebuffer inspection tools
- [x] Render all sketch entities
    - [x] Point renderer
    - [x] Circle renderer
- [ ] Tools?
    - [ ] Select
    - [ ] Line
    - [/] Point
    - [ ] Circle
    - [ ] Constrain
- [ ] Constraint rendering
    - [ ] Non-parametrized constraints
    - [ ] Dimensions

## Is it time for a proper mode system?

On one hand it is a pain to implement a proper mode stack in Rust. But problems are starting to arise now that I am implmenting pending state for drawing shapes. Some pending state can be quite complex, such as when creating a `BiConstraint` like distance which requires two entities and a float.

This could be stored in a separate variable. However if I want to make bad states impossible, this does require some special datatypes. We can't actually use a `BiConstraint` since it doesn't have `Option`s for the entities and `ConstraintType`.

### How could a mode system work?

It needs a stack. If all the modes are known at compile time we can store them in an enum. Is that desireable? That makes is harder or impossible to use as a library. Or the enum would have to implement a trait so that it has some known methods. **Lets try some dynamic dispatch and see how it goes**
```rust
pub struct ModeStack {
    modes: Vec<Box<dyn Mode>>,
}
```

What does a mode need to implement? Input handling. It also needs some way to modify the stack itself. By popping itself off the (and possible more modes upwards) and adding new, inner modes. This can happen on any input. The problem is that the mode can't have a reference to the stack it is in since it would need a mutable and immutable borrow at the same time to accomplish that.
```rust
pub trait Mode<'a>: Any {
    /// This can be put into a box for storage in a Vec
    fn new(mode_events: &'a ModeEventQueue) -> Self where Self: Sized;

    fn handle_key(
        &mut self,
        _key: Key,
        _scancode: Scancode,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }
    fn handle_mouse_button(
        &mut self,
        _button: MouseButton,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }
    fn handle_mouse_position(&mut self, _position: Vector<f32>, _delta: Vector<f32>) {}
    fn handle_mouse_scroll(&mut self, _scroll_delta: Vector<f32>) {}
}
```
We need to decouple the actual stack length changes from the moment they happen. Either all the handlers need to return a possible `ModeStackMessage`, or they get to borrow an event queue. They also need to return wether they used an event or not.

If I'm going to use the borrowing alternative, the borrowed list needs to be passed in for each handler since the trait can't store data. We can enforce a constructor which borrows a list. This is more powerful since one input event is allowed to queue multiple modifications of the `ModeStack`.

They will also need a mutable reference to the `App` but that can be passed into each handler.

I think we can't get around having message enums. How else could we do keybindings? Each action needs a name, then a mapping from that name to some action. Should each mode have its own message? They need to in order to avoid crazy overhead in dispatch.


### Where to store mode state

In the modestack I want the Modes to contain no state since they are used as keys in the HashMap. What if we hand
```rust
enum ModeId {
    Base,
    Sketch,
    Point,
    Line,
}

struct SketchModeData {
    sketch_id: u16,
}

struct LineModeData {
    p1: Option<Vector<f32>>,
    p2: Option<Vector<f32>>,
}

enum ModeData {
    Base,
    Sketch(SketchData),
    Point,
    Line(LineData),
}

struct AppMode {
    id: ModeId,
    data: ModeData,
}
```

If we assume that every mode can only be active once at a time (which is probably reasonable) all mode data can just be fields on the `App` struct instead. This should be more performant and ergonomic as opposed to search through the modestack on every access;
