//! Block System Integration Tests
//!
//! Tests for the block registration and instancing system.

use hipdf::blocks::{Block, BlockInstance, BlockManager, Transform, merge_blocks};
use hipdf::lopdf::{content::{Content, Operation}, dictionary, Dictionary, Document, Object, Stream};

use std::fs;
use std::path::Path;

/// Directory for test outputs
const TEST_OUTPUT_DIR: &str = "tests/outputs";

fn ensure_output_dir() {
    if !Path::new(TEST_OUTPUT_DIR).exists() {
        fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create test output directory");
    }
}

#[test]
fn test_transform() {
    let t1 = Transform::default();
    assert_eq!(t1.scale_x, 1.0);
    assert_eq!(t1.scale_y, 1.0);
    assert_eq!(t1.rotation, 0.0);

    let t2 = Transform::translate(100.0, 200.0);
    assert_eq!(t2.translate_x, 100.0);
    assert_eq!(t2.translate_y, 200.0);

    let t3 = Transform::translate_scale(50.0, 50.0, 2.0);
    assert_eq!(t3.scale_x, 2.0);
    assert_eq!(t3.scale_y, 2.0);

    let matrix = t2.to_matrix();
    assert_eq!(matrix[4], 100.0);
    assert_eq!(matrix[5], 200.0);
}

#[test]
fn test_block_creation() {
    let ops = vec![
        Operation::new("rg", vec![1.0.into(), 0.0.into(), 0.0.into()]),
        Operation::new("re", vec![0.0.into(), 0.0.into(), 50.0.into(), 50.0.into()]),
        Operation::new("f", vec![]),
    ];

    let block = Block::new("test_block", ops.clone());
    assert_eq!(block.id, "test_block");
    assert_eq!(block.operations.len(), 3);
    
    let block_with_bbox = Block::new("test_block2", ops)
        .with_bbox(0.0, 0.0, 50.0, 50.0);
    assert_eq!(block_with_bbox.bbox, Some((0.0, 0.0, 50.0, 50.0)));
}

#[test]
fn test_block_manager() {
    let mut manager = BlockManager::new();

    // Create some test operations
    let rect_ops = vec![
        Operation::new("re", vec![0.0.into(), 0.0.into(), 30.0.into(), 30.0.into()]),
        Operation::new("f", vec![]),
    ];

    let circle_ops = vec![
        Operation::new("m", vec![25.0.into(), 0.0.into()]),
        Operation::new("c", vec![
            25.0.into(), 13.807.into(),
            13.807.into(), 25.0.into(),
            0.0.into(), 25.0.into(),
        ]),
        Operation::new("f", vec![]),
    ];

    // Register blocks
    manager.register(Block::new("rect", rect_ops));
    manager.register(Block::new("circle", circle_ops));

    assert_eq!(manager.count(), 2);
    assert!(manager.has("rect"));
    assert!(manager.has("circle"));
    assert!(!manager.has("nonexistent"));

    // Get block
    let rect_block = manager.get("rect");
    assert!(rect_block.is_some());
    assert_eq!(rect_block.unwrap().id, "rect");

    // Remove block
    let removed = manager.remove("circle");
    assert!(removed.is_some());
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_block_instance() {
    let instance1 = BlockInstance::at("block1", 100.0, 200.0);
    assert_eq!(instance1.block_id, "block1");
    assert_eq!(instance1.transform.translate_x, 100.0);
    assert_eq!(instance1.transform.translate_y, 200.0);

    let instance2 = BlockInstance::at_scaled("block2", 50.0, 50.0, 1.5);
    assert_eq!(instance2.transform.scale_x, 1.5);
    assert_eq!(instance2.transform.scale_y, 1.5);

    let custom_transform = Transform::full(30.0, 40.0, 2.0, 3.0, 45.0);
    let instance3 = BlockInstance::new("block3", custom_transform);
    assert_eq!(instance3.transform.rotation, 45.0);
}

#[test]
fn test_render_instance() {
    let mut manager = BlockManager::new();

    let ops = vec![
        Operation::new("rg", vec![0.0.into(), 1.0.into(), 0.0.into()]),
        Operation::new("re", vec![0.0.into(), 0.0.into(), 20.0.into(), 20.0.into()]),
        Operation::new("f", vec![]),
    ];

    manager.register(Block::new("green_square", ops));

    let instance = BlockInstance::at("green_square", 100.0, 100.0);
    let rendered_ops = manager.render_instance(&instance);

    // Should have: q, cm, original ops, Q
    assert_eq!(rendered_ops.len(), 6); // q, cm, 3 original ops, Q
}

#[test]
fn test_render_multiple_instances() {
    let mut manager = BlockManager::new();

    let ops = vec![
        Operation::new("re", vec![0.0.into(), 0.0.into(), 10.0.into(), 10.0.into()]),
        Operation::new("S", vec![]),
    ];

    manager.register(Block::new("small_rect", ops));

    let instances = vec![
        BlockInstance::at("small_rect", 10.0, 10.0),
        BlockInstance::at("small_rect", 30.0, 10.0),
        BlockInstance::at("small_rect", 50.0, 10.0),
        BlockInstance::at_scaled("small_rect", 10.0, 30.0, 2.0),
    ];

    let rendered_ops = manager.render_instances(&instances);
    
    // Each instance: q, cm, 2 ops, Q = 5 ops per instance
    assert_eq!(rendered_ops.len(), 20);
}

#[test]
fn test_merge_blocks() {
    let block1 = Block::new("b1", vec![
        Operation::new("q", vec![]),
        Operation::new("Q", vec![]),
    ]);

    let block2 = Block::new("b2", vec![
        Operation::new("f", vec![]),
    ]);

    let merged = merge_blocks(&[&block1, &block2]);
    assert_eq!(merged.len(), 3);
}

/// Integration test with a complete PDF
#[test]
fn test_blocks_integration() {
    ensure_output_dir();

    // Create PDF document
    let mut doc = Document::with_version("1.7");

    // Setup document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // Add fonts
    let helvetica = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => helvetica,
        },
    };

    // Create block manager
    let mut block_manager = BlockManager::new();

    // Define a logo block (complex shape with multiple operations)
    let logo_ops = vec![
        // Blue background circle
        Operation::new("rg", vec![0.2.into(), 0.4.into(), 0.8.into()]),
        Operation::new("m", vec![50.0.into(), 5.0.into()]),
        Operation::new("c", vec![
            72.0.into(), 5.0.into(),
            90.0.into(), 23.0.into(),
            90.0.into(), 45.0.into(),
        ]),
        Operation::new("c", vec![
            90.0.into(), 67.0.into(),
            72.0.into(), 85.0.into(),
            50.0.into(), 85.0.into(),
        ]),
        Operation::new("c", vec![
            28.0.into(), 85.0.into(),
            10.0.into(), 67.0.into(),
            10.0.into(), 45.0.into(),
        ]),
        Operation::new("c", vec![
            10.0.into(), 23.0.into(),
            28.0.into(), 5.0.into(),
            50.0.into(), 5.0.into(),
        ]),
        Operation::new("f", vec![]),
        // White inner shape
        Operation::new("rg", vec![1.0.into(), 1.0.into(), 1.0.into()]),
        Operation::new("m", vec![50.0.into(), 20.0.into()]),
        Operation::new("l", vec![70.0.into(), 45.0.into()]),
        Operation::new("l", vec![50.0.into(), 70.0.into()]),
        Operation::new("l", vec![30.0.into(), 45.0.into()]),
        Operation::new("h", vec![]),
        Operation::new("f", vec![]),
    ];

    // Define an arrow block
    let arrow_ops = vec![
        Operation::new("rg", vec![0.3.into(), 0.7.into(), 0.3.into()]),
        Operation::new("m", vec![0.0.into(), 15.0.into()]),
        Operation::new("l", vec![30.0.into(), 15.0.into()]),
        Operation::new("l", vec![30.0.into(), 5.0.into()]),
        Operation::new("l", vec![45.0.into(), 20.0.into()]),
        Operation::new("l", vec![30.0.into(), 35.0.into()]),
        Operation::new("l", vec![30.0.into(), 25.0.into()]),
        Operation::new("l", vec![0.0.into(), 25.0.into()]),
        Operation::new("h", vec![]),
        Operation::new("f", vec![]),
    ];

    // Define a star block
    let star_ops = vec![
        Operation::new("rg", vec![1.0.into(), 0.843.into(), 0.0.into()]),
        Operation::new("m", vec![25.0.into(), 0.0.into()]),
        Operation::new("l", vec![31.0.into(), 17.0.into()]),
        Operation::new("l", vec![49.0.into(), 17.0.into()]),
        Operation::new("l", vec![34.0.into(), 28.0.into()]),
        Operation::new("l", vec![40.0.into(), 45.0.into()]),
        Operation::new("l", vec![25.0.into(), 35.0.into()]),
        Operation::new("l", vec![10.0.into(), 45.0.into()]),
        Operation::new("l", vec![16.0.into(), 28.0.into()]),
        Operation::new("l", vec![1.0.into(), 17.0.into()]),
        Operation::new("l", vec![19.0.into(), 17.0.into()]),
        Operation::new("h", vec![]),
        Operation::new("f", vec![]),
    ];

    // Define a text label block
    let label_ops = vec![
        // Background
        Operation::new("rg", vec![0.9.into(), 0.9.into(), 0.9.into()]),
        Operation::new("re", vec![0.0.into(), 0.0.into(), 80.0.into(), 25.0.into()]),
        Operation::new("f", vec![]),
        // Border
        Operation::new("RG", vec![0.0.into(), 0.0.into(), 0.0.into()]),
        Operation::new("w", vec![1.0.into()]),
        Operation::new("re", vec![0.0.into(), 0.0.into(), 80.0.into(), 25.0.into()]),
        Operation::new("S", vec![]),
        // Text
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 10.0.into()]),
        Operation::new("Td", vec![10.0.into(), 8.0.into()]),
        Operation::new("Tj", vec![Object::string_literal("LABEL")]),
        Operation::new("ET", vec![]),
    ];

    // Register all blocks
    block_manager.register(Block::new("logo", logo_ops).with_bbox(0.0, 0.0, 100.0, 90.0));
    block_manager.register(Block::new("arrow", arrow_ops).with_bbox(0.0, 0.0, 45.0, 40.0));
    block_manager.register(Block::new("star", star_ops).with_bbox(0.0, 0.0, 50.0, 45.0));
    block_manager.register(Block::new("label", label_ops).with_bbox(0.0, 0.0, 80.0, 25.0));

    // Create XObjects for efficient reuse
    block_manager.create_xobjects(&mut doc);

    // Define instances with various transformations
    let instances = vec![
        // Row of logos at different scales
        BlockInstance::at("logo", 50.0, 700.0),
        BlockInstance::at_scaled("logo", 180.0, 700.0, 0.7),
        BlockInstance::at_scaled("logo", 280.0, 700.0, 0.5),
        BlockInstance::at_scaled("logo", 350.0, 700.0, 1.2),
        
        // Arrows pointing different directions
        BlockInstance::new("arrow", Transform::translate(50.0, 600.0)),
        BlockInstance::new("arrow", Transform::full(150.0, 600.0, 1.0, 1.0, 45.0)),
        BlockInstance::new("arrow", Transform::full(250.0, 600.0, 1.0, 1.0, 90.0)),
        BlockInstance::new("arrow", Transform::full(350.0, 600.0, 1.0, 1.0, 180.0)),
        BlockInstance::new("arrow", Transform::full(450.0, 600.0, 1.0, 1.0, 270.0)),
        
        // Grid of stars
        BlockInstance::at("star", 50.0, 500.0),
        BlockInstance::at("star", 110.0, 500.0),
        BlockInstance::at("star", 170.0, 500.0),
        BlockInstance::at("star", 230.0, 500.0),
        BlockInstance::at("star", 290.0, 500.0),
        
        // Stars with different scales
        BlockInstance::at_scaled("star", 50.0, 420.0, 0.5),
        BlockInstance::at_scaled("star", 110.0, 420.0, 0.75),
        BlockInstance::at_scaled("star", 170.0, 420.0, 1.0),
        BlockInstance::at_scaled("star", 230.0, 420.0, 1.25),
        BlockInstance::at_scaled("star", 290.0, 420.0, 1.5),
        
        // Labels at different positions and scales
        BlockInstance::at("label", 50.0, 350.0),
        BlockInstance::at_scaled("label", 150.0, 350.0, 1.5),
        BlockInstance::new("label", Transform::translate_scale_xy(280.0, 350.0, 2.0, 1.0)),
        
        // Complex transformations
        BlockInstance::new("logo", Transform::full(100.0, 200.0, 0.8, 1.2, 30.0)),
        BlockInstance::new("star", Transform::full(250.0, 200.0, 2.0, 2.0, 15.0)),
        BlockInstance::new("arrow", Transform::full(400.0, 200.0, 1.5, 0.8, -30.0)),
    ];

    // Build page content
    let mut all_operations = Vec::new();

    // Add title
    all_operations.push(Operation::new("BT", vec![]));
    all_operations.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 24.0.into()]));
    all_operations.push(Operation::new("Td", vec![50.0.into(), 800.0.into()]));
    all_operations.push(Operation::new("Tj", vec![Object::string_literal("PDF Block System Demo")]));
    all_operations.push(Operation::new("ET", vec![]));

    // Add subtitle
    all_operations.push(Operation::new("BT", vec![]));
    all_operations.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.0.into()]));
    all_operations.push(Operation::new("Td", vec![50.0.into(), 780.0.into()]));
    all_operations.push(Operation::new("Tj", vec![Object::string_literal("Reusable PDF content blocks with multiple instances")]));
    all_operations.push(Operation::new("ET", vec![]));

    // Render instances using XObjects
    let instance_ops = block_manager.render_instances_as_xobjects(&instances, &mut resources);
    all_operations.extend(instance_ops);

    // Also demonstrate direct rendering (without XObjects)
    let direct_instances = vec![
        BlockInstance::new("logo", Transform::full(450.0, 100.0, 0.5, 0.5, 0.0)),
        BlockInstance::new("star", Transform::full(500.0, 100.0, 0.5, 0.5, 0.0)),
    ];
    all_operations.extend(block_manager.render_instances(&direct_instances));

    // Create content stream
    let content = Content { operations: all_operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Create page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Update pages
    doc.get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap()
        .set("Kids", vec![Object::Reference(page_id)]);

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save PDF
    let output_path = format!("{}/blocks_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    assert!(Path::new(&output_path).exists());

    println!("✅ Block system test completed successfully");
    println!("📄 PDF created: {}", output_path);
    println!("\n📦 Blocks registered:");
    println!("  - logo: Complex shape with 90x90 bbox");
    println!("  - arrow: Directional arrow with 45x40 bbox");
    println!("  - star: Star shape with 50x45 bbox");
    println!("  - label: Text label with background");
    println!("\n🔄 Instances created:");
    println!("  - 4 logo instances (various scales)");
    println!("  - 5 arrow instances (various rotations)");
    println!("  - 10 star instances (grid and scaled)");
    println!("  - 3 label instances (various scales)");
    println!("  - 3 complex transformed instances");
    println!("  - 2 directly rendered instances");
    println!("\n💡 Total: 27 instances from 4 unique blocks");
}

#[test]
fn test_xobject_creation() {
    let mut doc = Document::with_version("1.7");
    let mut manager = BlockManager::new();

    let ops = vec![
        Operation::new("rg", vec![1.0.into(), 0.0.into(), 1.0.into()]),
        Operation::new("re", vec![0.0.into(), 0.0.into(), 50.0.into(), 50.0.into()]),
        Operation::new("f", vec![]),
    ];

    manager.register(Block::new("magenta_square", ops).with_bbox(0.0, 0.0, 50.0, 50.0));
    
    // Create XObjects
    manager.create_xobjects(&mut doc);

    // Create instances and render as XObjects
    let instances = vec![
        BlockInstance::at("magenta_square", 10.0, 10.0),
        BlockInstance::at_scaled("magenta_square", 70.0, 10.0, 2.0),
    ];

    let mut resources = Dictionary::new();
    let ops = manager.render_instances_as_xobjects(&instances, &mut resources);

    // Should have created XObject references
    assert!(resources.has(b"XObject"));
    // Each instance: q, cm, Do, Q = 4 ops
    assert_eq!(ops.len(), 8);
}

#[test]
fn test_block_with_resources() {
    let font_dict = dictionary! {
        "F1" => Object::Name(b"Helvetica".to_vec()),
    };

    let resources = dictionary! {
        "Font" => font_dict,
    };

    let ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.0.into()]),
        Operation::new("Tj", vec![Object::string_literal("Text")]),
        Operation::new("ET", vec![]),
    ];

    let block = Block::new("text_block", ops)
        .with_resources(resources.clone());

    assert!(block.resources.is_some());
    assert_eq!(block.resources.unwrap(), resources);
}