# Benchmarking of text rendering log

```nu
./bench.nu text-rendering 10 200
```

**Initial results:**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.536 s ±  0.006 s    [User: 2.684 s, System: 0.111 s]
  Range (min … max):    3.528 s …  3.546 s    10 runs
```

**Move atlases from HashMaps to Vec:**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.529 s ±  0.008 s    [User: 1.648 s, System: 0.122 s]
  Range (min … max):    3.514 s …  3.538 s    10 runs
```

**Move characters in atlas from Hashmap to Vec:**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.525 s ±  0.007 s    [User: 1.435 s, System: 0.117 s]
  Range (min … max):    3.517 s …  3.535 s    10 runs
```

Why does user time go down but the actual time doesnt? Probably because CPU time is greatly reduced but bandwidth to GPU is the bottleneck.

**Post line and measurement cache:**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.524 s ±  0.008 s    [User: 0.383 s, System: 0.123 s]
  Range (min … max):    3.515 s …  3.538 s    10 runs
```

**After bug fixes:**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):      3.518 s ±  0.006 s    [User: 0.477 s, System: 0.133 s]
  Range (min … max):    3.510 s …  3.527 s    10 runs
```

**After disabling vsync and batching draw calls per paragraph (on laptop):**
```
Benchmark 1: ./target/release/benchmarks --iters 200 text-rendering
  Time (mean ± σ):     868.3 ms ±  16.3 ms    [User: 439.8 ms, System: 348.5 ms]
  Range (min … max):   846.0 ms … 896.3 ms    10 runs
```

It turns out the vsync was enabled even though I thought it wasnt. With an option to disable it, things sped up by a lot.

After all these changes, we're comfortable back to being CPU limited and are way below the frame budget on release build. On debug builds we exceed the frame budget if the debug window is open.

## Resources

Render infinte amounts of text with one draw call: https://github.com/Samson-Mano/opengl_textrendering
