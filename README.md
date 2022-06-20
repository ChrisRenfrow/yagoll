# YAGoLL (Yet Another Game of Life Library)

![crates.io version shield](https://img.shields.io/crates/v/yagoll.svg)

## What?

`yagoll` is a simple Game of Life library designed to be consumed by other Rust programs for the purposes of initializing and interacting with configurable Game of Life instances.
	
## Why?

I wanted to learn how to make and publish a Rust library and this seemed like an easy way to accomplish that.

## Examples

To see the library in action, follow the steps below:

1. Clone the repo: 
   `git clone https://github.com/ChrisRenfrow/yagoll.git && cd yagoll`
2. Run the provided example:
   `cargo run --example play-game-of-life ./tests/test-boards/four-circles.txt 20 500`

## Roadmap

- [x] Basic Functionality
- [ ] Extended
  - [ ] Provide a method that returns a range of pre-computed cycles
  - [ ] Implement `Iterable` for `Board`
  - [ ] Write board state to file at path
