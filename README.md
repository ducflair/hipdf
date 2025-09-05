# hipdf

[![Crates.io](https://img.shields.io/crates/v/hipdf.svg)](https://crates.io/crates/hipdf)
[![Documentation](https://docs.rs/hipdf/badge.svg)](https://docs.rs/hipdf)

A high-level PDF manipulation library built on [lopdf](https://github.com/j-f-liu/lopdf), focusing on ease of use and powerful abstractions for common PDF operations following the **ISO 32000-2** standard.

## Features

- **Optional Content Groups (OCG)**: Easy creation and management of PDF layers
- **Layer Management**: High-level API for organizing content into toggleable layers
- **Hatching Patterns**: Support for various fill patterns including crosshatching, dots, and custom patterns
- **PDF Embedding**: Embed other PDF documents with various layout strategies
- **Block System**: Reusable PDF content components with transformations and efficient rendering
- **Type Safety**: Strongly typed interfaces with compile-time guarantees

# Showcase
You can see the results of our test suit in [this](https://github.com/ducflair/hipdf/tree/main/tests/outputs) folder.


## Quick Start

### Creating PDF Layers

```rust
use hipdf::ocg::{OCGManager, Layer, LayerContentBuilder, LayerOperations as Ops};
use hipdf::lopdf::{Document, Object};

// Create a new PDF with layers
let mut doc = Document::with_version("1.5");
let mut ocg_manager = OCGManager::with_config(Default::default());

// Add layers
ocg_manager.add_layer(Layer::new("Background", true));
ocg_manager.add_layer(Layer::new("Main Content", true));
ocg_manager.add_layer(Layer::new("Annotations", false));

// Initialize layers in document
ocg_manager.initialize(&mut doc);

// Build content for specific layers
let mut builder = LayerContentBuilder::new();
builder.begin_layer("L0")
    .add_operation(Ops::rectangle(50.0, 50.0, 200.0, 100.0))
    .add_operation(Ops::fill())
    .end_layer();
```

### Adding Hatching Patterns

```rust
use hipdf::hatching::{HatchingManager, HatchStyle, PatternedShapeBuilder};

// Create a hatching manager
let mut manager = HatchingManager::new();

// Add a diagonal pattern
let pattern_id = manager.add_pattern(HatchStyle::DiagonalRight, 5.0, 1.0);

// Create a shape with the pattern
let mut builder = PatternedShapeBuilder::new();
builder.rectangle(100.0, 100.0, 200.0, 150.0, &pattern_id);
```

### Embedding PDFs

```rust
use hipdf::embed_pdf::{PdfEmbedder, EmbedOptions, MultiPageLayout};

// Create an embedder
let mut embedder = PdfEmbedder::new();

// Load a PDF
embedder.load_pdf("source.pdf", "doc1")?;

let options = EmbedOptions {
    layout: MultiPageLayout::Vertical { gap: 10.0 },
    scale: 1.0,
    ..Default::default()
};

// Embed into target document
embedder.embed_pdf(&mut target_doc, "doc1", &options)?;
```

### Creating Reusable Blocks

```rust
use hipdf::blocks::{BlockManager, Block, BlockInstance, Transform};
use hipdf::lopdf::content::Operation;

// Create a block manager
let mut manager = BlockManager::new();

// Create a reusable block
let mut block = Block::new("my_shape", vec![
    Operation::new("re", vec![0.into(), 0.into(), 100.into(), 50.into()]),
    Operation::new("f", vec![]),
]);
manager.register(block);

// Create instances with different transformations
let instances = vec![
    BlockInstance::at("my_shape", 50.0, 100.0),
    BlockInstance::at_scaled("my_shape", 200.0, 100.0, 0.5),
    BlockInstance::new("my_shape", Transform::full(350.0, 100.0, 1.5, 1.5, 45.0)),
];

// Render all instances
let operations = manager.render_instances(&instances);
```

## Modules

- [`ocg`] - Optional Content Groups (layers) functionality
- [`hatching`] - Hatching and pattern support for PDF documents
- [`embed_pdf`] - PDF embedding and composition support
- [`blocks`] - Reusable PDF content components with transformations

## Usage Examples

### Layer Management

```rust
use hipdf::ocg::{OCGManager, Layer};

// Create layer manager
let mut manager = OCGManager::new();

// Add layers with different visibility settings
manager.add_layer(Layer::new("Background", true));
manager.add_layer(Layer::new("Content", true));
manager.add_layer(Layer::new("Debug", false));

// Initialize in PDF document
manager.initialize(&mut doc);
```

### Custom Hatching Patterns

```rust
use hipdf::hatching::{CustomPatternBuilder, HatchStyle};

// Create custom pattern
let mut pattern_builder = CustomPatternBuilder::new();
pattern_builder
    .move_to(0.0, 0.0)
    .line_to(10.0, 10.0)
    .line_to(20.0, 0.0);

// Register the pattern
let custom_pattern_id = manager.add_custom_pattern(pattern_builder.build());
```

### Advanced PDF Embedding

```rust
use hipdf::embed_pdf::{EmbedLayoutBuilder, LayoutStrategy};

// Create layout builder
let mut layout_builder = EmbedLayoutBuilder::new();

// Add multiple PDFs with different layouts
layout_builder
    .add_pdf("doc1.pdf", LayoutStrategy::SinglePage { x: 0.0, y: 0.0 })
    .add_pdf("doc2.pdf", LayoutStrategy::Grid {
        columns: 2,
        spacing: 10.0
    });

// Generate the final document
let final_doc = layout_builder.build()?;
```

### Block System

```rust
use hipdf::blocks::{BlockManager, Block, BlockInstance, Transform};

// Create block manager and register reusable content
let mut manager = BlockManager::new();
let star_block = Block::new("star", vec![
    // Operations to draw a star shape
]).with_bbox(0.0, 0.0, 50.0, 50.0);
manager.register(star_block);

// Create multiple instances with different positions and transformations
let instances = vec![
    BlockInstance::at("star", 100.0, 200.0),
    BlockInstance::new("star", Transform::translate_scale(200.0, 200.0, 1.5)),
    BlockInstance::new("star", Transform::full(300.0, 200.0, 0.8, 0.8, 30.0)),
];

// Render instances efficiently using Form XObjects
manager.create_xobjects(&mut doc);
let operations = manager.render_instances_as_xobjects(&instances, &mut resources);
```

## Requirements

- Rust 1.70+

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## API Documentation

For complete API documentation, visit [docs.rs/hipdf](https://docs.rs/hipdf).
