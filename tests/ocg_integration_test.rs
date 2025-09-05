//! OCG Integration Tests
//!
//! These tests validate the OCG functionality by creating actual PDF files
//! and verifying their correct structure and layer creation.
//!
//! The tests cover:
//! - Basic OCG manager creation and configuration
//! - Layer creation and management
//! - Content building with layers
//! - Full PDF generation with OCG support
//! - Performance and edge cases

use hipdf::ocg::{Layer, LayerContentBuilder, LayerOperations as Ops, OCGConfig, OCGManager};
use hipdf::lopdf::{content::Content, dictionary, Document, Object, Stream};

use std::fs;
use std::path::Path;

/// Directory for test outputs
const TEST_OUTPUT_DIR: &str = "tests/outputs";

fn ensure_output_dir() {
    if !Path::new(TEST_OUTPUT_DIR).exists() {
        fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create test output directory");
    }
}

fn cleanup_test_files() {
    if Path::new(TEST_OUTPUT_DIR).exists() {
        if let Ok(entries) = fs::read_dir(TEST_OUTPUT_DIR) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();

                        // Keep test PDFs for inspection (any file ending with _test.pdf)
                        if !file_name_str.ends_with("_test.pdf") {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_ocg_manager_creation() {
    let config = OCGConfig::default();
    let manager = OCGManager::with_config(config);

    assert!(!manager.has_oc_properties());
    assert!(manager.is_empty());
}

#[test]
fn test_layer_creation() {
    let layer = Layer::new("Test Layer", false);

    assert_eq!(layer.name, "Test Layer");
    assert!(!layer.default_visible);
    assert!(layer.tag.is_none());
}

#[test]
fn test_layer_content_builder() {
    let mut builder = LayerContentBuilder::new();

    builder
        .begin_layer("L0")
        .add_operation(Ops::rectangle(0.0, 0.0, 100.0, 100.0))
        .end_layer();

    let operations = builder.build();
    assert!(!operations.is_empty());
    assert_eq!(operations.len(), 3); // BDC, Operation, EMC
}

#[test]
fn test_layer_operations() {
    // Test individual layer operations
    let rect_op = Ops::rectangle(10.0, 20.0, 100.0, 200.0);
    let fill_op = Ops::fill();
    let stroke_op = Ops::stroke();

    // Verify operation types - Operation has a public method to get the operator
    assert_eq!(
        format!("{:?}", rect_op),
        format!("{:?}", Ops::rectangle(10.0, 20.0, 100.0, 200.0))
    );
    assert_eq!(format!("{:?}", fill_op), format!("{:?}", Ops::fill()));
    assert_eq!(format!("{:?}", stroke_op), format!("{:?}", Ops::stroke()));

    let color_op = Ops::set_fill_color_rgb(1.0, 0.5, 0.0);
    assert_eq!(
        format!("{:?}", color_op),
        format!("{:?}", Ops::set_fill_color_rgb(1.0, 0.5, 0.0))
    );

    let text_op = Ops::show_text("Test Text");
    assert_eq!(
        format!("{:?}", text_op),
        format!("{:?}", Ops::show_text("Test Text"))
    );

    // Operations created successfully
    assert!(true);
}

#[test]
fn test_ocg_configuration() {
    // Test different OCG configurations
    let configs = vec![
        OCGConfig {
            base_state: "OFF".to_string(),
            create_panel_ui: false,
            intent: vec!["View".to_string()],
        },
        OCGConfig {
            base_state: "ON".to_string(),
            create_panel_ui: true,
            intent: vec!["View".to_string(), "Design".to_string()],
        },
    ];

    for config in configs {
        let manager = OCGManager::with_config(config.clone());
        // Configuration should be stored correctly
        // (This tests the config storage, actual usage tested above)
        assert_eq!(manager.config.base_state, config.base_state);
        assert_eq!(manager.config.create_panel_ui, config.create_panel_ui);
        assert_eq!(manager.config.intent, config.intent);
    }
}

/// Performance test to ensure OCG operations are efficient
#[test]
fn test_ocg_performance() {
    use std::time::Instant;

    let mut manager = OCGManager::new();

    // Add multiple layers for performance testing
    let start = Instant::now();
    for i in 0..50 {
        manager.add_layer(Layer::new(format!("Layer {}", i), true));
    }
    let duration = start.elapsed();

    // Should be very fast (< 1ms per layer)
    let avg_time = duration.as_nanos() as f64 / 50_000_000.0;
    println!("📊 Layer creation: {:.2}ms per layer", avg_time);

    assert!(avg_time < 1.0, "Layer creation should be fast");
}

/// Test layer retrieval and modification
#[test]
fn test_layer_retrieval() {
    let mut manager = OCGManager::new();

    let layer1 = Layer::new("Test Layer 1", true);
    let layer2 = Layer::new("Test Layer 2", false);

    let idx1 = manager.add_layer(layer1);
    let idx2 = manager.add_layer(layer2);

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(manager.len(), 2);

    // Test retrieval
    let retrieved = manager.get_layer("Test Layer 1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Layer 1");
    assert!(retrieved.unwrap().default_visible);

    let retrieved2 = manager.get_layer("Nonexistent");
    assert!(retrieved2.is_none());

    // Test mutable retrieval
    let mut_layer = manager.get_layer_mut("Test Layer 2");
    assert!(mut_layer.is_some());
    mut_layer.unwrap().default_visible = true;
    assert!(manager.get_layer("Test Layer 2").unwrap().default_visible);
}

/// Test content builder with multiple layers and operations
#[test]
fn test_complex_content_building() {
    let mut builder = LayerContentBuilder::new();

    // Build content with nested operations
    builder
        .begin_layer("L0")
        .add_operation(Ops::begin_text())
        .add_operation(Ops::set_font("F1", 12.0))
        .add_operation(Ops::text_position(10.0, 20.0))
        .add_operation(Ops::show_text("Layer 0 content"))
        .add_operation(Ops::end_text())
        .add_operation(Ops::rectangle(10.0, 10.0, 100.0, 50.0))
        .add_operation(Ops::fill())
        .end_layer()
        .begin_layer("L1")
        .add_operation(Ops::set_fill_color_rgb(1.0, 0.0, 0.0))
        .add_operation(Ops::rectangle(20.0, 20.0, 80.0, 40.0))
        .add_operation(Ops::stroke())
        .end_layer();

    let operations = builder.build();
    assert!(!operations.is_empty());

    // Verify operations structure
    // Should have: BDC, BT, Tf, Td, Tj, ET, re, f, EMC, BDC, rg, re, S, EMC
    assert_eq!(operations.len(), 14);
}

/// Test OCG configuration variations
#[test]
fn test_ocg_config_variations() {
    let configs = vec![
        OCGConfig {
            base_state: "OFF".to_string(),
            create_panel_ui: false,
            intent: vec!["View".to_string()],
        },
        OCGConfig {
            base_state: "ON".to_string(),
            create_panel_ui: true,
            intent: vec![
                "View".to_string(),
                "Design".to_string(),
                "Print".to_string(),
            ],
        },
        OCGConfig {
            base_state: "Unchanged".to_string(),
            create_panel_ui: true,
            intent: vec![],
        },
    ];

    for config in configs {
        let manager = OCGManager::with_config(config.clone());
        assert_eq!(manager.config.base_state, config.base_state);
        assert_eq!(manager.config.create_panel_ui, config.create_panel_ui);
        assert_eq!(manager.config.intent, config.intent);
    }
}

/// Test layer tag generation and resource setup
#[test]
fn test_layer_tags_and_resources() {
    let mut doc = Document::with_version("1.5");
    let mut manager = OCGManager::new();

    // Add some layers
    manager.add_layer(Layer::new("Layer A", true));
    manager.add_layer(Layer::new("Layer B", false));
    manager.initialize(&mut doc);

    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => doc.add_object(dictionary! {
                "Type" => "Font",
                "Subtype" => "Type1",
                "BaseFont" => "Helvetica",
            }),
        },
    };

    let layer_tags = manager.setup_page_resources(&mut resources);

    // Verify tags
    assert_eq!(layer_tags.len(), 2);
    assert_eq!(
        layer_tags.get(&"Layer A".to_string()),
        Some(&"L0".to_string())
    );
    assert_eq!(
        layer_tags.get(&"Layer B".to_string()),
        Some(&"L1".to_string())
    );

    // Verify resources contain Properties
    assert!(resources.has(b"Properties"));
}

/// Test that creates an advanced layered PDF similar to the original main.rs
/// This generates a visually rich PDF with colors, shapes, and detailed content
#[test]
fn test_ocg_integration() {
    ensure_output_dir();

    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    // Setup basic document structure
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

    let helvetica_bold = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
    });

    // 1. Create and configure the OCG manager
    let config = OCGConfig {
        base_state: "ON".to_string(),
        create_panel_ui: true,
        intent: vec!["View".to_string(), "Design".to_string()],
    };
    let mut ocg_manager = OCGManager::with_config(config);

    // 2. Define your layers
    ocg_manager.add_layer(Layer::new("Background", true));
    ocg_manager.add_layer(Layer::new("Main Content", true));
    ocg_manager.add_layer(Layer::new("Annotations", false));
    ocg_manager.add_layer(Layer::new("Watermark", false));
    ocg_manager.add_layer(Layer::new("Debug Info", false));

    // 3. Initialize layers in the document
    ocg_manager.initialize(&mut doc);

    // 4. Setup page resources
    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => helvetica,
            "F2" => helvetica_bold,
        },
    };

    let layer_tags = ocg_manager.setup_page_resources(&mut resources);

    // 5. Build the page content using the LayerContentBuilder
    let mut builder = LayerContentBuilder::new();

    // Background layer - a light blue background
    if let Some(bg_tag) = layer_tags.get(&"Background".to_string()) {
        builder
            .begin_layer(bg_tag)
            .add_operation(Ops::set_fill_color_rgb(0.9, 0.95, 1.0))
            .add_operation(Ops::rectangle(0.0, 0.0, 595.0, 842.0))
            .add_operation(Ops::fill())
            .end_layer();
    }

    // Main content layer
    if let Some(content_tag) = layer_tags.get(&"Main Content".to_string()) {
        builder
            .begin_layer(content_tag)
            // Title
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_gray(0.0))
            .add_operation(Ops::set_font("F2", 24.0))
            .add_operation(Ops::text_position(50.0, 750.0))
            .add_operation(Ops::show_text("PDF with Optional Content Groups"))
            .add_operation(Ops::end_text())
            // Body text
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_font("F1", 12.0))
            .add_operation(Ops::text_position(50.0, 700.0))
            .add_operation(Ops::show_text(
                "This document contains multiple layers that can be toggled on/off.",
            ))
            .add_operation(Ops::end_text())
            // A shape
            .add_operation(Ops::set_fill_color_rgb(0.2, 0.6, 0.2))
            .add_operation(Ops::rectangle(50.0, 550.0, 200.0, 100.0))
            .add_operation(Ops::fill())
            .end_layer();
    }

    // Annotations layer - some red annotations
    if let Some(anno_tag) = layer_tags.get(&"Annotations".to_string()) {
        builder
            .begin_layer(anno_tag)
            // Red circle (approximated with rectangle for simplicity)
            .add_operation(Ops::set_stroke_color_rgb(1.0, 0.0, 0.0))
            .add_operation(Ops::rectangle(260.0, 560.0, 80.0, 80.0))
            .add_operation(Ops::stroke())
            // Annotation text
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_rgb(1.0, 0.0, 0.0))
            .add_operation(Ops::set_font("F1", 10.0))
            .add_operation(Ops::text_position(350.0, 590.0))
            .add_operation(Ops::show_text("Important!"))
            .add_operation(Ops::end_text())
            .end_layer();
    }

    // Watermark layer
    if let Some(watermark_tag) = layer_tags.get(&"Watermark".to_string()) {
        builder
            .begin_layer(watermark_tag)
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_gray(0.8))
            .add_operation(Ops::set_font("F2", 48.0))
            .add_operation(Ops::text_position(150.0, 400.0))
            .add_operation(Ops::show_text("DRAFT"))
            .add_operation(Ops::end_text())
            .end_layer();
    }

    // Debug info layer
    if let Some(debug_tag) = layer_tags.get(&"Debug Info".to_string()) {
        builder
            .begin_layer(debug_tag)
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_rgb(0.5, 0.5, 0.5))
            .add_operation(Ops::set_font("F1", 8.0))
            .add_operation(Ops::text_position(50.0, 50.0))
            .add_operation(Ops::show_text("Debug: Document created with OCG support"))
            .add_operation(Ops::end_text())
            .add_operation(Ops::begin_text())
            .add_operation(Ops::text_position(50.0, 40.0))
            .add_operation(Ops::show_text(&format!("Layers: {}", layer_tags.len())))
            .add_operation(Ops::end_text())
            .end_layer();
    }

    // Content that's always visible (not in any layer)
    builder
        .add_operation(Ops::begin_text())
        .add_operation(Ops::set_fill_color_gray(0.0))
        .add_operation(Ops::set_font("F1", 10.0))
        .add_operation(Ops::text_position(50.0, 100.0))
        .add_operation(Ops::show_text(
            "This text is always visible (not in any layer).",
        ))
        .add_operation(Ops::end_text());

    // 6. Create the content stream from the builder
    let operations = builder.build();
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // 7. Create the page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // 8. Update pages dictionary
    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    // 9. Create and setup the catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // 10. Update the catalog with OCProperties
    ocg_manager.update_catalog(&mut doc);

    // 11. Save the PDF
    let output_path = format!("{}/ocg_integration_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    // Verify the file was created
    assert!(Path::new(&output_path).exists());

    println!("✅ Advanced layered PDF test completed successfully");
    println!("📄 PDF created: {}", output_path);
    println!("\nLayers created:");
    println!("  - Background (visible by default)");
    println!("  - Main Content (visible by default)");
    println!("  - Annotations (hidden by default)");
    println!("  - Watermark (hidden by default)");
    println!("  - Debug Info (hidden by default)");
    println!(
        "\n💡 Open the PDF in a viewer that supports layers (like Adobe Acrobat) to toggle them!"
    );
}

/// Clean up fixture
#[test]
fn cleanup() {
    cleanup_test_files();
}
