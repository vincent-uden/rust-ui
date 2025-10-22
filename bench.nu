#!/usr/bin/env nu

def main [benchmark: string, hyperfine_iters: int, internal_iters: int] {
  cargo build --profile bench -p benchmarks;
  hyperfine --warmup 2 --runs $hyperfine_iters ($"./target/release/benchmarks --iters ($internal_iters) ($benchmark)")
}
