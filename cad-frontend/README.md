# Cad Frontend

## Roadmap
- [x] Dynamic panes
- [ ] Editing sketches
  - [x] Fix Text rendering grainy renders
  - [x] Make sure Point mode is enabled
  - [x] Line mode
  - [x] Circle mode
  - [x] Show pending shapes

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
    - [x] Line
    - [x] Point
    - [x] Circle
    - [ ] Constrain
- [ ] Constraint rendering
    - [ ] Non-parametrized constraints
    - [ ] Dimensions
- [ ] Wires (series of lines (shapes???))
    - [ ] Split line into line-point-line
    - [ ] Split circle into arc-point-arc
- [ ] Loops (closed wire)
    - [ ] Is the mouse INSIDE or OUTSIDE a given loop?

## Towards extrude
- [ ] Extrude polygon
    - [ ] Extrude mode (in base)
    - [ ] Determine which wire 
    - [ ] Mesh shader
- [ ] Extrude circle
- [ ] 3D Boolean join
    - [ ] Intersection of 3D bodies
- [ ] 3D Boolean cut
