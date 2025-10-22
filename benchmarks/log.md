# Benchmarking of text rendering log

```nu
./bench.nu text-rendering 10 200
```

Initial results:
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.536 s ±  0.006 s    [User: 2.684 s, System: 0.111 s]
  Range (min … max):    3.528 s …  3.546 s    10 runs
```

Move atlases from HashMaps to Vec:
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.529 s ±  0.008 s    [User: 1.648 s, System: 0.122 s]
  Range (min … max):    3.514 s …  3.538 s    10 runs
```

Move characters in atlas from Hashmap to Vec:
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.525 s ±  0.007 s    [User: 1.435 s, System: 0.117 s]
  Range (min … max):    3.517 s …  3.535 s    10 runs
```

Why does user time go down but the actual time doesnt? Probably because CPU time is greatly reduced but bandwidth to GPU is the bottleneck.

Post line and measurement cache:
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.524 s ±  0.008 s    [User: 0.383 s, System: 0.123 s]
  Range (min … max):    3.515 s …  3.538 s    10 runs
```

## Resources

Render infinte amounts of text with one draw call: https://github.com/Samson-Mano/opengl_textrendering
