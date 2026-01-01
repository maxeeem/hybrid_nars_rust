# Hybrid NARS Rust

A Rust implementation of a Hybrid Non-Axiomatic Reasoning System (NARS).

## Overview

This project implements a NARS reasoning system with hybrid capabilities, integrating symbolic reasoning with continuous representations (embeddings).

## Features

- **NARS Logic**: Implements core NARS logic including deduction, induction, abduction, and revision.
- **Hybrid Representation**: Utilizes GloVe embeddings for grounding concepts.
- **Advanced Operators**: Support for equivalence operators (`<|>`, `</>`, `<\>`) and interval processing.
- **Inference Control**: Mechanisms for concept drift detection and answer filtering based on confidence.
- **Interactive REPL**: A command-line interface for interacting with the system.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- GloVe embeddings file (expected at `assets/glove.txt`)

### Building

```bash
cargo build --release
```

### Running

To start the REPL:

```bash
cargo run --bin repl
```

To run the test runner:

```bash
cargo run --bin test_runner
```

## Project Structure

- `src/nars`: Core NARS implementation (logic, memory, control).
- `src/bin`: Executables (REPL, test runner).
- `assets`: Resource files (embeddings).
- `tests`: NAL test files.

## License

[License Information]
