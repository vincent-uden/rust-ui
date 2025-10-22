use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Copy, Subcommand)]
enum Benchmark {
    TextRendering,
}

#[derive(Parser, Debug, Clone, Copy)]
struct Args {
    #[command(subcommand)]
    benchmark: Benchmark,
}

mod text_rendering;

fn main() {
    let args = Args::parse();

    match args.benchmark {
        Benchmark::TextRendering => text_rendering::render_text(100),
    }

    println!("Hello, world!");
}
