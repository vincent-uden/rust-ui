# Cad Frontend

## Roadmap
- [x] Dynamic panes
- [ ] Editing sketches
- [ ] 3D Operations

### Towards editing sketches
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
    - [x] Line
    - [x] Point
    - [x] Circle
    - [ ] Constrain
- [ ] Constraint rendering
    - [ ] Non-parametrized constraints
    - [ ] Dimensions
- [/] Wires/Loops (series of lines (shapes???))
    - [x] Write tests in `sketch.rs` for `is_inside`
    - [ ] Create loops when appropriate, for example when placing lines
    - [ ] Split line into line-point-line
    - [ ] Split circle into arc-point-arc
- [ ] Loops (closed wire)
    - [ ] Is the mouse INSIDE or OUTSIDE a given loop?

#### Inside/outside loops
I can already get the mouse position in the plane. *How would I go about determining if that point is insdie or outside a general (possibly non-convex) polygon?* For a convex polygon I could just determine if I am to the left or right of every side, store them in a consistent clockwise or anti-clockwise order and check if I'm to the inner side of all lines.

*Is that possible for a non-convex polygon as well?* No. Imagine a U-shape. If the point is inside the right arm of the "U" it is outside the left arm and would be classified incorrectly.

Thus, we need to triangulate or at least split non-convex polygons in to convex sub-shapes.

What about line-arc combinations. I guess those must also be split into convex shapes. But I'm not entirely sure how.

[Containment test for polygons containing circular arcs](https://ieeexplore.ieee.org/document/1011280) contains the exact algorithm needed.

#### Creating loops
The creation of loops should be handled in the `cad` crate, not in the frontend. It is a common CAD operation to draw a bunch of lines and ask about what sort of interior shapes that have been created.

### Towards extrude
- [ ] Extrude polygon
    - [ ] Extrude mode (in base)
    - [ ] Determine which wire 
    - [ ] Mesh shader
- [ ] Extrude circle
- [ ] 3D Boolean join
    - [ ] Intersection of 3D bodies
- [ ] 3D Boolean cut

