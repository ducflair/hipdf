# hipdf

[![Crates.io](https://img.shields.io/crates/v/hipdf.svg)](https://crates.io/crates/hipdf)
[![Documentation](https://docs.rs/hipdf/badge.svg)](https://docs.rs/hipdf)
[![Build Status](https://github.com/jorgesoares/hipdf/workflows/CI/badge.svg)](https://github.com/jorgesoares/hipdf/actions)

A high-level PDF manipulation library built on [lopdf](https://github.com/j-f-liu/lopdf), focusing on ease of use and powerful abstractions for common PDF operations.

## Features

- **🖼️ OCG (Optional Content Groups) Support**: Easy creation and management of PDF layers
- **🔧 Layer Management**: High-level API for organizing content into toggleable layers
- **📦 Content Building**: Fluent API for building layered PDF content
- **🛡️ Type Safety**: Strongly typed interfaces with compile-time guarantees
- **⚡ Performance**: Efficient operations with minimal allocations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hipdf = "0.1.0"
```

## Quick Start

```rust
use hipdf::{OCGManager, Layer, LayerContentBuilder, LayerOperations as Ops};
use lopdf::{dictionary, Document, Object, content::Content, Stream};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    // Configure the OCG manager
    let config = hipdf::OCGConfig::default();
    let mut ocg_manager = OCGManager::with_config(config);

    // Add layers
    ocg_manager.add_layer(Layer::new("Main Content", true));
    ocg_manager.add_layer(Layer::new("Annotations", false));
    ocg_manager.add_layer(Layer::new("Watermark", false));

    // Initialize layers in the document
    ocg_manager.initialize(&mut doc);

    // Build your PDF...
    // (See examples for complete implementation)

    Ok(())
}
```

## Project Structure

```
hipdf/
├── src/
│   ├── lib.rs           # Main library module
│   ├── ocg.rs           # OCG implementation
│   └── examples/
│       └── ocg_example.rs  # Complete example
├── tests/
│   └── ocg_integration_test.rs  # Integration tests
├── Makefile             # Development commands
├── Cargo.toml           # Dependencies and project config
└── README.md            # This file
```

## Examples

Run the included example:

```bash
# Clone the repository
git clone https://github.com/jorgesoares/hipdf.git
cd hipdf

# Run the example
make example

# Or using cargo directly
cargo run --bin hipdf-example
```

This creates a PDF with multiple layers that can be toggled in compatible PDF viewers.

## Usage

### Creating Layers

```rust
use hipdf::{OCGManager, Layer};

// Create a layer manager
let mut manager = OCGManager::new();

// Add individual layers
let main_layer = Layer::new("Main Content", true);
let annotations_layer = Layer::new("Annotations", false);

// Add layers to manager
manager.add_layer(main_layer);
manager.add_layer(annotations_layer);
```

### Building Content

```rust
use hipdf::{LayerContentBuilder, LayerOperations as Ops};

// Create a content builder
let mut builder = LayerContentBuilder::new();

// Add content to specific layers
builder.begin_layer("L0")
    .add_operation(Ops::rectangle(50.0, 50.0, 200.0, 100.0))
    .add_operation(Ops::fill())
    .end_layer();

// Get the operations
let content = builder.build();
```

### Document Integration

```rust
// Initialize in document
manager.initialize(&mut doc);

// Setup page resources
let resources = dictionary! { "Font" => font_refs };
let layer_tags = manager.setup_page_resources(&mut resources);

// Update catalog
manager.update_catalog(&mut doc);
```

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Quick build
make build
# or
cargo build

# Build in release mode
make build-release
# or
cargo build --release
```

### Testing

```bash
# Run all tests
make test
# or
cargo test

# Run with verbose output
make test-verbose
# or
cargo test -- --nocapture

# Run specific test
cargo test test_ocg_integration

# Run performance test
cargo test test_ocg_performance
```

### Development Workflow

Use the provided Makefile for common tasks:

```bash
# Format code
make fmt

# Run linter
make clippy

# Full development check (format + lint + check + test)
make dev-check

# Generate documentation
make doc

# Clean build artifacts
make clean
```

### Testing Output

Tests that generate PDF files will place them in `tests/outputs/` directory. The `.gitignore` file ensures these are not committed to version control.

## API Reference

### Core Types

- **`OCGManager`**: Main struct for managing Optional Content Groups
  - `new()` - Create with default configuration
  - `with_config(config)` - Create with custom configuration
  - `add_layer(layer)` - Add a layer
  - `initialize(document)` - Set up layers in document
  - `setup_page_resources(resources)` - Configure page for layers
  - `update_catalog(document)` - Update document catalog

- **`Layer`**: Represents a single PDF layer
  - `new(name, visible)` - Create a new layer
  - `with_visibility(visible)` - Set visibility (fluent API)

- **`LayerContentBuilder`**: Fluent API for building layer content
  - `new()` - Create a new builder
  - `begin_layer(tag)` - Start content for a layer
  - `add_operation(op)` - Add a PDF operation
  - `end_layer()` - End current layer
  - `build()` - Get final operations

- **`OCGConfig`**: Configuration for OCG system
  - `base_state` - Default ON/OFF state for layers
  - `create_panel_ui` - Whether to show layer panel
  - `intent` - Layer purposes (View, Design, etc.)

### OCG Implementation Details

HiPDF implements the PDF Optional Content Groups (OCG) specification:

- **OCG Objects**: Each layer becomes an OCG object with a name and visibility settings
- **OCProperties**: Contains layer ordering and default visibility
- **BDC/EMC Operators**: Content streams use these operators to mark layer boundaries
- **Resource Properties**: Layers are registered in page resource dictionaries

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make sure tests pass (`make dev-check`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built on top of the excellent [lopdf](https://github.com/j-f-liu/lopdf) library
- PDF OCG specification (ISO 32000-2)
- Rust community for the fantastic ecosystem

## Roadmap

- [ ] Additional PDF manipulation features (text, images, forms)
- [ ] More OCG extensions (usage application, viewer preferences)
- [ ] PDF/A compliance checking
- [ ] Performance optimizations
- [ ] More comprehensive test coverage
- [ ] Documentation improvements

---

For more information, see the [API documentation](https://docs.rs/hipdf) or explore the examples in `src/examples/`.
