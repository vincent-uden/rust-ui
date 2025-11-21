# Rust Ui

## Roadmap
- Multiple fonts

## Installing tracy profiler
```nu
git clone https://github.com/wolfpld/tracy
git checkout v0.12.2
cmake -B profiler/build -S profiler
cd profiler/build
make
cp ./tracy-profiler ~/.local/bin
```

## Code generation
`scripts/codegen.py` is supposed to be run on every change to the sprites in order to create the atlas. It also generates a rust module in cad-frontend containing an enum of all the available sprite keys.

## Dependencies

### System packages
- `glfw`
