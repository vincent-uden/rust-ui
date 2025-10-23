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
