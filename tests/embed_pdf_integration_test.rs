//! PDF Embedding Integration Tests - Comprehensive Suite
//!
//! These tests validate and showcase all PDF embedding features

use hipdf::embed_pdf::{
    CustomLayoutStrategy, EmbedOptions, GridFillOrder, MultiPageLayout, PageRange, PdfEmbedder,
};
use hipdf::lopdf::{content::Content, dictionary, Dictionary, Document, Object, Stream};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Directory for test outputs
const TEST_OUTPUT_DIR: &str = "tests/outputs/embed_pdf_integration_test";

fn ensure_output_dir() {
    if !Path::new(TEST_OUTPUT_DIR).exists() {
        fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create test output directory");
    }
}

/// Helper function to create a basic page structure
fn create_page_with_title(
    doc: &mut Document,
    title: &str,
) -> (
    Object,
    Vec<lopdf::content::Operation>,
    HashMap<String, Object>,
) {
    // Add font for labels
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let mut page_ops = Vec::new();

    // Title
    page_ops.extend(vec![
        lopdf::content::Operation::new("BT", vec![]),
        lopdf::content::Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 20.into()]),
        lopdf::content::Operation::new("Td", vec![50.into(), 800.into()]),
        lopdf::content::Operation::new("Tj", vec![Object::string_literal(title)]),
        lopdf::content::Operation::new("ET", vec![]),
    ]);

    let xobjects = HashMap::new();

    (lopdf::Object::Reference(font_id), page_ops, xobjects)
}

/// Helper to add a section label
fn add_section_label(ops: &mut Vec<lopdf::content::Operation>, x: f32, y: f32, text: &str) {
    ops.extend(vec![
        lopdf::content::Operation::new("BT", vec![]),
        lopdf::content::Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 10.into()]),
        lopdf::content::Operation::new("Td", vec![x.into(), y.into()]),
        lopdf::content::Operation::new("Tj", vec![Object::string_literal(text)]),
        lopdf::content::Operation::new("ET", vec![]),
    ]);
}

/// Helper to draw a border around an area
fn add_border(ops: &mut Vec<lopdf::content::Operation>, x: f32, y: f32, width: f32, height: f32) {
    ops.extend(vec![
        lopdf::content::Operation::new("q", vec![]),
        lopdf::content::Operation::new("RG", vec![0.5.into(), 0.5.into(), 0.5.into()]),
        lopdf::content::Operation::new("w", vec![0.5.into()]),
        lopdf::content::Operation::new("re", vec![x.into(), y.into(), width.into(), height.into()]),
        lopdf::content::Operation::new("S", vec![]),
        lopdf::content::Operation::new("Q", vec![]),
    ]);
}

#[test]
fn test_vertical_layouts() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Vertical Layout Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Vertical stack with small gap
    add_section_label(
        &mut page_ops,
        50.0,
        750.0,
        "1. Vertical stack - First 3 pages (small gap)",
    );
    add_border(&mut page_ops, 45.0, 450.0, 110.0, 280.0);

    let vertical_small_gap = EmbedOptions::new()
        .at_position(50.0, 720.0)
        .with_max_size(100.0, 80.0)
        .with_layout(MultiPageLayout::Vertical { gap: 5.0 })
        .with_page_range(PageRange::Range(0, 2));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &vertical_small_gap)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Vertical stack with large gap
    add_section_label(
        &mut page_ops,
        200.0,
        750.0,
        "2. Vertical stack - Pages 2-5 (large gap)",
    );
    add_border(&mut page_ops, 195.0, 420.0, 110.0, 310.0);

    let vertical_large_gap = EmbedOptions::new()
        .at_position(200.0, 720.0)
        .with_max_size(100.0, 60.0)
        .with_layout(MultiPageLayout::Vertical { gap: 20.0 })
        .with_page_range(PageRange::Range(1, 4));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &vertical_large_gap)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Vertical stack with different scales
    add_section_label(&mut page_ops, 350.0, 750.0, "3. Vertical - Different sizes");
    add_border(&mut page_ops, 345.0, 500.0, 160.0, 230.0);

    let vertical_varied = EmbedOptions::new()
        .at_position(350.0, 720.0)
        .with_scale(0.25)
        .with_layout(MultiPageLayout::Vertical { gap: 10.0 })
        .with_page_range(PageRange::Pages(vec![0, 2, 4]));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &vertical_varied)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/vertical_layouts_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Vertical layouts test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_horizontal_layouts() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Horizontal Layout Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Horizontal spread - small pages
    add_section_label(
        &mut page_ops,
        50.0,
        700.0,
        "1. Horizontal spread - First 4 pages (small)",
    );
    add_border(&mut page_ops, 45.0, 580.0, 500.0, 110.0);

    let horizontal_small = EmbedOptions::new()
        .at_position(50.0, 680.0)
        .with_max_size(100.0, 100.0)
        .with_layout(MultiPageLayout::Horizontal { gap: 20.0 })
        .with_page_range(PageRange::Range(0, 3));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &horizontal_small)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Horizontal spread - medium pages with tight spacing
    add_section_label(
        &mut page_ops,
        50.0,
        550.0,
        "2. Horizontal spread - Pages 2-4 (medium, tight)",
    );
    add_border(&mut page_ops, 45.0, 400.0, 500.0, 140.0);

    let horizontal_medium = EmbedOptions::new()
        .at_position(50.0, 530.0)
        .with_max_size(150.0, 130.0)
        .with_layout(MultiPageLayout::Horizontal { gap: 5.0 })
        .with_page_range(PageRange::Range(1, 3));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &horizontal_medium)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Horizontal spread - varying widths
    add_section_label(
        &mut page_ops,
        50.0,
        370.0,
        "3. Horizontal spread - Custom selection",
    );
    add_border(&mut page_ops, 45.0, 250.0, 500.0, 110.0);

    let horizontal_custom = EmbedOptions::new()
        .at_position(50.0, 350.0)
        .with_scale_xy(0.15, 0.12)
        .with_layout(MultiPageLayout::Horizontal { gap: 30.0 })
        .with_page_range(PageRange::Pages(vec![0, 3, 5, 7]));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &horizontal_custom)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/horizontal_layouts_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Horizontal layouts test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_grid_layouts() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Grid Layout Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: 2x3 Grid (Row First)
    add_section_label(&mut page_ops, 50.0, 700.0, "1. 2x3 Grid - Row First Fill");
    add_border(&mut page_ops, 45.0, 480.0, 230.0, 210.0);

    let grid_2x3_row = EmbedOptions::new()
        .at_position(50.0, 680.0)
        .with_max_size(70.0, 90.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 2,
            gap_x: 15.0,
            gap_y: 10.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 5));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &grid_2x3_row)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: 3x2 Grid (Column First)
    add_section_label(
        &mut page_ops,
        300.0,
        700.0,
        "2. 3x2 Grid - Column First Fill",
    );
    add_border(&mut page_ops, 295.0, 480.0, 260.0, 210.0);

    let grid_3x2_col = EmbedOptions::new()
        .at_position(300.0, 680.0)
        .with_max_size(70.0, 90.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 3,
            gap_x: 10.0,
            gap_y: 15.0,
            fill_order: GridFillOrder::ColumnFirst,
        })
        .with_page_range(PageRange::Range(0, 5));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &grid_3x2_col)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: 4x4 Grid - Thumbnail gallery
    add_section_label(&mut page_ops, 50.0, 450.0, "3. 4x4 Thumbnail Gallery");
    add_border(&mut page_ops, 45.0, 140.0, 300.0, 300.0);

    let grid_4x4_thumb = EmbedOptions::new()
        .at_position(50.0, 430.0)
        .with_max_size(60.0, 60.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 4,
            gap_x: 10.0,
            gap_y: 10.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 11))
        .preserve_aspect_ratio(true);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &grid_4x4_thumb)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 4: 5x1 Grid (essentially horizontal with grid control)
    add_section_label(&mut page_ops, 50.0, 110.0, "4. 5x1 Grid - Single row");
    add_border(&mut page_ops, 45.0, 30.0, 500.0, 70.0);

    let grid_5x1 = EmbedOptions::new()
        .at_position(50.0, 95.0)
        .with_max_size(80.0, 60.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 5,
            gap_x: 15.0,
            gap_y: 0.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 4));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &grid_5x1).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/grid_layouts_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Grid layouts test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_scaling_and_sizing() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Scaling and Sizing Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Different scale factors
    add_section_label(&mut page_ops, 50.0, 700.0, "1. Scale: 0.1x");
    let scale_tiny = EmbedOptions::new()
        .at_position(50.0, 620.0)
        .with_scale(0.1)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &scale_tiny)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(&mut page_ops, 150.0, 700.0, "2. Scale: 0.2x");
    let scale_small = EmbedOptions::new()
        .at_position(150.0, 620.0)
        .with_scale(0.2)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &scale_small)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(&mut page_ops, 320.0, 700.0, "3. Scale: 0.3x");
    let scale_medium = EmbedOptions::new()
        .at_position(320.0, 620.0)
        .with_scale(0.3)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &scale_medium)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Non-uniform scaling
    add_section_label(
        &mut page_ops,
        50.0,
        450.0,
        "4. Non-uniform: Wide (0.3x, 0.1x)",
    );
    let scale_wide = EmbedOptions::new()
        .at_position(50.0, 400.0)
        .with_scale_xy(0.3, 0.1)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &scale_wide)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(
        &mut page_ops,
        300.0,
        450.0,
        "5. Non-uniform: Tall (0.1x, 0.3x)",
    );
    let scale_tall = EmbedOptions::new()
        .at_position(300.0, 400.0)
        .with_scale_xy(0.1, 0.3)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &scale_tall)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Max size constraints
    add_section_label(&mut page_ops, 50.0, 250.0, "6. Max size: 150x100");
    add_border(&mut page_ops, 45.0, 140.0, 160.0, 110.0);

    let max_size_1 = EmbedOptions::new()
        .at_position(50.0, 240.0)
        .with_max_size(150.0, 100.0)
        .with_layout(MultiPageLayout::FirstPageOnly)
        .preserve_aspect_ratio(true);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &max_size_1)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(&mut page_ops, 250.0, 250.0, "7. Max size: 100x150");
    add_border(&mut page_ops, 245.0, 85.0, 110.0, 160.0);

    let max_size_2 = EmbedOptions::new()
        .at_position(250.0, 240.0)
        .with_max_size(100.0, 150.0)
        .with_layout(MultiPageLayout::FirstPageOnly)
        .preserve_aspect_ratio(true);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &max_size_2)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(&mut page_ops, 400.0, 250.0, "8. Max: 100x100 (square)");
    add_border(&mut page_ops, 395.0, 140.0, 110.0, 110.0);

    let max_size_square = EmbedOptions::new()
        .at_position(400.0, 240.0)
        .with_max_size(100.0, 100.0)
        .with_layout(MultiPageLayout::FirstPageOnly)
        .preserve_aspect_ratio(true);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &max_size_square)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/scaling_sizing_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Scaling and sizing test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_custom_layouts() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Custom Layout Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Diagonal cascade
    add_section_label(&mut page_ops, 50.0, 700.0, "1. Diagonal Cascade");

    let diagonal_strategy = CustomLayoutStrategy {
        position_fn: |idx, _w, _h| (idx as f32 * 50.0, -(idx as f32 * 50.0)),
        scale_fn: |idx| {
            let scale = 0.2 - (idx as f32 * 0.03);
            (scale, scale)
        },
    };

    let diagonal_cascade = EmbedOptions::new()
        .at_position(50.0, 650.0)
        .with_layout(MultiPageLayout::Custom(diagonal_strategy))
        .with_page_range(PageRange::Range(0, 4));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &diagonal_cascade)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Spiral arrangement
    add_section_label(&mut page_ops, 300.0, 700.0, "2. Spiral Arrangement");

    let spiral_strategy = CustomLayoutStrategy {
        position_fn: |idx, _w, _h| {
            let angle = idx as f32 * 0.8;
            let radius = idx as f32 * 20.0;
            (angle.cos() * radius, angle.sin() * radius)
        },
        scale_fn: |_idx| (0.1, 0.1),
    };

    let spiral = EmbedOptions::new()
        .at_position(400.0, 600.0)
        .with_layout(MultiPageLayout::Custom(spiral_strategy))
        .with_page_range(PageRange::Range(0, 7));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &spiral).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Decreasing size stack
    add_section_label(&mut page_ops, 50.0, 350.0, "3. Decreasing Size Stack");

    let decreasing_strategy = CustomLayoutStrategy {
        position_fn: |idx, _w, _h| (idx as f32 * 10.0, 0.0),
        scale_fn: |idx| {
            let scale = 0.25 / (idx as f32 + 1.0);
            (scale, scale)
        },
    };

    let decreasing = EmbedOptions::new()
        .at_position(50.0, 300.0)
        .with_layout(MultiPageLayout::Custom(decreasing_strategy))
        .with_page_range(PageRange::Range(0, 3));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &decreasing)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 4: Wave pattern
    add_section_label(&mut page_ops, 200.0, 180.0, "4. Wave Pattern");

    let wave_strategy = CustomLayoutStrategy {
        position_fn: |idx, _w, _h| {
            let x = idx as f32 * 60.0;
            let y = (idx as f32 * 1.5).sin() * 30.0;
            (x, y)
        },
        scale_fn: |_idx| (0.08, 0.08),
    };

    let wave = EmbedOptions::new()
        .at_position(200.0, 100.0)
        .with_layout(MultiPageLayout::Custom(wave_strategy))
        .with_page_range(PageRange::Range(0, 6));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &wave).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/custom_layouts_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Custom layouts test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_page_ranges() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Page Range Selection Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Single page
    add_section_label(&mut page_ops, 50.0, 700.0, "1. Single page (page 3)");

    let single_page = EmbedOptions::new()
        .at_position(50.0, 600.0)
        .with_max_size(100.0, 100.0)
        .with_page_range(PageRange::Single(2));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &single_page)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Range of pages
    add_section_label(&mut page_ops, 200.0, 700.0, "2. Range (pages 2-5)");

    let range = EmbedOptions::new()
        .at_position(200.0, 680.0)
        .with_max_size(60.0, 60.0)
        .with_layout(MultiPageLayout::Vertical { gap: 5.0 })
        .with_page_range(PageRange::Range(1, 4));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &range).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Specific pages
    add_section_label(&mut page_ops, 300.0, 700.0, "3. Specific (1,3,5,7)");

    let specific = EmbedOptions::new()
        .at_position(300.0, 680.0)
        .with_max_size(60.0, 60.0)
        .with_layout(MultiPageLayout::Vertical { gap: 5.0 })
        .with_page_range(PageRange::Pages(vec![0, 2, 4, 6]));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &specific).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 4: All pages in grid
    add_section_label(&mut page_ops, 50.0, 350.0, "4. All pages (first 12)");
    add_border(&mut page_ops, 45.0, 80.0, 500.0, 260.0);

    let all_pages = EmbedOptions::new()
        .at_position(50.0, 330.0)
        .with_max_size(40.0, 50.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 6,
            gap_x: 40.0,
            gap_y: 10.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 11));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &all_pages)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/page_ranges_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Page ranges test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_rotation_and_transformations() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Rotation and Transformation Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example: Different rotation angles
    let angles = vec![0.0, 15.0, 30.0, 45.0, 60.0, 90.0];
    let mut x_pos = 50.0;

    for angle in angles {
        add_section_label(&mut page_ops, x_pos, 700.0, &format!("{}°", angle));

        let rotated = EmbedOptions::new()
            .at_position(x_pos + 30.0, 600.0)
            .with_scale(0.1)
            .with_rotation(angle)
            .with_layout(MultiPageLayout::FirstPageOnly);

        let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &rotated).unwrap();
        page_ops.extend(result.operations);
        all_xobjects.extend(result.xobject_resources);

        x_pos += 80.0;
    }

    // Example: Rotated multi-page layout
    add_section_label(&mut page_ops, 50.0, 400.0, "Rotated Grid Layout (45°)");

    let rotated_grid = EmbedOptions::new()
        .at_position(200.0, 300.0)
        .with_max_size(50.0, 50.0)
        .with_rotation(45.0)
        .with_layout(MultiPageLayout::Grid {
            columns: 3,
            gap_x: 10.0,
            gap_y: 10.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 5));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &rotated_grid)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/rotation_transformations_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Rotation and transformations test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_mixed_pdfs() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Mixed PDF Embedding");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();
    let lines_pdf = embedder.load_pdf("tests/assets/lines.pdf").unwrap();

    // Example: Alternating PDFs in a grid
    add_section_label(&mut page_ops, 50.0, 700.0, "Alternating PDFs in Grid");

    let mut y_pos = 650.0;
    for row in 0..3 {
        let mut x_pos = 50.0;
        for col in 0..4 {
            let pdf_to_use = if (row + col) % 2 == 0 {
                &arxiv_pdf
            } else {
                &lines_pdf
            };
            let page_num = if pdf_to_use == &arxiv_pdf { row } else { 0 };

            let options = EmbedOptions::new()
                .at_position(x_pos, y_pos)
                .with_max_size(80.0, 80.0)
                .with_layout(MultiPageLayout::SpecificPage(page_num));

            let result = embedder.embed_pdf(&mut doc, pdf_to_use, &options).unwrap();
            page_ops.extend(result.operations);
            all_xobjects.extend(result.xobject_resources);

            x_pos += 100.0;
        }
        y_pos -= 100.0;
    }

    // Example: Side by side comparison
    add_section_label(&mut page_ops, 50.0, 320.0, "Side-by-side Comparison");

    let comparison_left = EmbedOptions::new()
        .at_position(50.0, 200.0)
        .with_max_size(200.0, 100.0)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &comparison_left)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    let comparison_right = EmbedOptions::new()
        .at_position(280.0, 200.0)
        .with_max_size(200.0, 100.0)
        .with_layout(MultiPageLayout::FirstPageOnly);

    let result = embedder
        .embed_pdf(&mut doc, &lines_pdf, &comparison_right)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/mixed_pdfs_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Mixed PDFs test completed");
    println!("📄 PDF created: {}", output_path);
}

#[test]
fn test_clipping_and_bounds() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "Clipping and Bounds Examples");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Example 1: Clipped to small rectangle
    add_section_label(&mut page_ops, 50.0, 700.0, "1. Clipped to small rectangle");
    add_border(&mut page_ops, 50.0, 600.0, 100.0, 80.0);

    let clipped_small = EmbedOptions {
        page_range: Some(PageRange::Single(1)),
        position: (50.0, 600.0),
        clip_bounds: Some((0.1, 0.8, 0.4, 0.9)),
        max_width: Some(100.0),
        max_height: Some(80.0),
        ..Default::default()
    };

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &clipped_small)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 2: Clipped grid layout
    add_section_label(&mut page_ops, 200.0, 700.0, "2. Clipped grid");
    add_border(&mut page_ops, 200.0, 550.0, 150.0, 130.0);

    let clipped_grid = EmbedOptions {
        page_range: Some(PageRange::Pages(vec![1, 2, 3, 4, 5, 6])),
        layout: MultiPageLayout::Grid {
            columns: 3,
            gap_x: 5.0,
            gap_y: 5.0,
            fill_order: GridFillOrder::RowFirst,
        },
        position: (200.0, 550.0),
        clip_bounds: Some((0.1, 0.1, 0.9, 0.9)),
        max_width: Some(150.0),
        max_height: Some(130.0),
        ..Default::default()
    };

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &clipped_grid)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Example 3: Aspect ratio preservation vs stretching
    add_section_label(&mut page_ops, 50.0, 400.0, "3. Preserve aspect ratio");
    add_border(&mut page_ops, 50.0, 300.0, 150.0, 80.0);

    let preserve_aspect = EmbedOptions {
        page_range: Some(PageRange::Single(1)),
        position: (50.0, 300.0),
        max_width: Some(150.0),
        max_height: Some(80.0),
        preserve_aspect_ratio: true,
        ..Default::default()
    };

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &preserve_aspect)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    add_section_label(&mut page_ops, 250.0, 400.0, "4. Stretched to fit");
    add_border(&mut page_ops, 250.0, 300.0, 150.0, 80.0);

    let stretched = EmbedOptions {
        page_range: Some(PageRange::Single(1)),
        position: (250.0, 300.0),
        max_width: Some(150.0),
        max_height: Some(80.0),
        preserve_aspect_ratio: false,
        ..Default::default()
    };

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &stretched)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/clipping_bounds_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Clipping and bounds test completed");
    println!("📄 PDF created: {}", output_path);
}

/// Test to generate a comprehensive showcase document
#[test]
fn test_comprehensive_showcase() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let (font_id, mut page_ops, mut all_xobjects) =
        create_page_with_title(&mut doc, "PDF Embedding - Comprehensive Showcase");

    let mut embedder = PdfEmbedder::new();
    let arxiv_pdf = embedder.load_pdf("tests/assets/2412.07377v3.pdf").unwrap();

    // Row 1: Different scales
    add_section_label(
        &mut page_ops,
        50.0,
        720.0,
        "Scales: 0.05, 0.1, 0.15, 0.2, 0.25",
    );
    let scales = [0.05, 0.1, 0.15, 0.2, 0.25];
    for (i, scale) in scales.iter().enumerate() {
        let options = EmbedOptions::new()
            .at_position(50.0 + i as f32 * 100.0, 700.0)
            .with_scale(*scale)
            .with_layout(MultiPageLayout::FirstPageOnly);

        let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &options).unwrap();
        page_ops.extend(result.operations);
        all_xobjects.extend(result.xobject_resources);
    }

    // Row 2: Grid variations
    add_section_label(&mut page_ops, 50.0, 570.0, "Grid Layouts: 2x2, 3x2, 4x2");
    let grid_configs = [(2, 2), (3, 2), (4, 2)];
    for (i, (cols, rows)) in grid_configs.iter().enumerate() {
        let options = EmbedOptions::new()
            .at_position(50.0 + i as f32 * 180.0, 550.0)
            .with_max_size(30.0, 30.0)
            .with_layout(MultiPageLayout::Grid {
                columns: *cols,
                gap_x: 5.0,
                gap_y: 5.0,
                fill_order: GridFillOrder::RowFirst,
            })
            .with_page_range(PageRange::Range(0, cols * rows - 1));

        let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &options).unwrap();
        page_ops.extend(result.operations);
        all_xobjects.extend(result.xobject_resources);
    }

    // Row 3: Page selection patterns
    add_section_label(
        &mut page_ops,
        50.0,
        430.0,
        "Page Selections: All Even, All Odd, First 3, Last 3",
    );

    // Even pages
    let even_pages = EmbedOptions::new()
        .at_position(50.0, 400.0)
        .with_max_size(40.0, 40.0)
        .with_layout(MultiPageLayout::Horizontal { gap: 5.0 })
        .with_page_range(PageRange::Pages(vec![0, 2, 4, 6]));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &even_pages)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Odd pages
    let odd_pages = EmbedOptions::new()
        .at_position(250.0, 400.0)
        .with_max_size(40.0, 40.0)
        .with_layout(MultiPageLayout::Horizontal { gap: 5.0 })
        .with_page_range(PageRange::Pages(vec![1, 3, 5, 7]));

    let result = embedder
        .embed_pdf(&mut doc, &arxiv_pdf, &odd_pages)
        .unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Row 4: Rotations
    add_section_label(
        &mut page_ops,
        50.0,
        250.0,
        "Rotation angles: 0°, 30°, 60°, 90°, 120°, 150°",
    );
    let angles = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0];
    for (i, angle) in angles.iter().enumerate() {
        let options = EmbedOptions::new()
            .at_position(80.0 + i as f32 * 80.0, 180.0)
            .with_scale(0.08)
            .with_rotation(*angle)
            .with_layout(MultiPageLayout::FirstPageOnly);

        let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &options).unwrap();
        page_ops.extend(result.operations);
        all_xobjects.extend(result.xobject_resources);
    }

    // Row 5: Custom artistic layout
    add_section_label(&mut page_ops, 50.0, 100.0, "Artistic Arrangement");

    let artistic_strategy = CustomLayoutStrategy {
        position_fn: |idx, _w, _h| {
            let angle = idx as f32 * std::f32::consts::PI / 4.0;
            let radius = 50.0;
            (200.0 + angle.cos() * radius, -angle.sin() * radius)
        },
        scale_fn: |idx| {
            let scale = 0.06 + (idx as f32 * 0.01);
            (scale, scale)
        },
    };

    let artistic = EmbedOptions::new()
        .at_position(50.0, 50.0)
        .with_layout(MultiPageLayout::Custom(artistic_strategy))
        .with_page_range(PageRange::Range(0, 7));

    let result = embedder.embed_pdf(&mut doc, &arxiv_pdf, &artistic).unwrap();
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create the page
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => xobject_dict,
    };

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/comprehensive_showcase_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).unwrap();

    assert!(Path::new(&output_path).exists());
    println!("✅ Comprehensive showcase test completed");
    println!("📄 PDF created: {}", output_path);
}

/// Helper to create a minimal PDF in bytes with custom MediaBox, UserUnit, Rotate, and Content
fn create_custom_test_pdf(
    media_box: (f32, f32, f32, f32),
    user_unit: Option<f32>,
    rotate: Option<i64>,
    user_unit_on_parent: bool,
    content_ops: Vec<lopdf::content::Operation>,
) -> Vec<u8> {
    let mut doc = Document::with_version("1.6");
    let mut pages_dict = dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    };
    if user_unit_on_parent {
        if let Some(uu) = user_unit {
            pages_dict.set("UserUnit", Object::Real(uu));
        }
    }
    let pages_id = doc.add_object(pages_dict);

    let content = Content {
        operations: content_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut page_dict = dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![
            media_box.0.into(),
            media_box.1.into(),
            media_box.2.into(),
            media_box.3.into(),
        ],
        "Contents" => content_id,
    };

    if !user_unit_on_parent {
        if let Some(uu) = user_unit {
            page_dict.set("UserUnit", Object::Real(uu));
        }
    }
    if let Some(rot) = rotate {
        page_dict.set("Rotate", Object::Integer(rot));
    }

    let page_id = doc.add_object(page_dict);

    let pages_obj = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_obj.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}

#[test]
fn test_user_unit_basic() {
    ensure_output_dir();

    // Create a 306x396 PDF with UserUnit 2.0 (physically US Letter 612x792 pt)
    let pdf_bytes = create_custom_test_pdf(
        (0.0, 0.0, 306.0, 396.0),
        Some(2.0),
        None,
        false,
        vec![
            lopdf::content::Operation::new("q", vec![]),
            lopdf::content::Operation::new("re", vec![0.into(), 0.into(), 306.into(), 396.into()]),
            lopdf::content::Operation::new("f", vec![]),
            lopdf::content::Operation::new("Q", vec![]),
        ],
    );

    let mut embedder = PdfEmbedder::new();
    let id = embedder
        .load_pdf_from_bytes(&pdf_bytes, "user_unit_pdf")
        .unwrap();

    // Verify info reports scaled physical dimensions
    let info = embedder.get_pdf_info(&id).unwrap();
    assert_eq!(info.page_count, 1);
    assert_eq!(info.page_dimensions[0], (612.0, 792.0));

    // Embed into target doc
    let mut target_doc = Document::with_version("1.6");
    let options = EmbedOptions::default();
    let result = embedder.embed_pdf(&mut target_doc, &id, &options).unwrap();

    // Check XObject was created with scaled BBox [0, 0, 612, 792]
    assert_eq!(result.xobject_resources.len(), 1);
    let xobj_ref = result.xobject_resources.values().next().unwrap();
    if let Object::Reference(xobj_id) = xobj_ref {
        let xobj = target_doc.get_object(*xobj_id).unwrap();
        let stream = xobj.as_stream().unwrap();
        let bbox = stream.dict.get(b"BBox").unwrap().as_array().unwrap();
        let to_f32 = |o: &Object| match o {
            Object::Real(v) => *v as f32,
            Object::Integer(v) => *v as f32,
            _ => 0.0,
        };
        assert_eq!(to_f32(&bbox[0]), 0.0);
        assert_eq!(to_f32(&bbox[1]), 0.0);
        assert_eq!(to_f32(&bbox[2]), 612.0);
        assert_eq!(to_f32(&bbox[3]), 792.0);

        // Decompress content stream and verify "2 0 0 2 0 0 cm" is prepended
        let decompressed = stream.decompressed_content().unwrap();
        let content_str = String::from_utf8_lossy(&decompressed);
        assert!(
            content_str.starts_with("2 0 0 2 0 0 cm"),
            "Expected content to start with UserUnit cm transform, got: {}",
            content_str
        );
    } else {
        panic!("Expected XObject reference");
    }
}

#[test]
fn test_user_unit_inheritance() {
    ensure_output_dir();

    // Create a PDF where UserUnit 3.0 is inherited from the parent /Pages node
    let pdf_bytes = create_custom_test_pdf(
        (0.0, 0.0, 100.0, 200.0),
        Some(3.0),
        None,
        true, // on parent /Pages
        vec![
            lopdf::content::Operation::new("q", vec![]),
            lopdf::content::Operation::new("re", vec![0.into(), 0.into(), 100.into(), 200.into()]),
            lopdf::content::Operation::new("f", vec![]),
            lopdf::content::Operation::new("Q", vec![]),
        ],
    );

    let mut embedder = PdfEmbedder::new();
    let id = embedder
        .load_pdf_from_bytes(&pdf_bytes, "inherited_user_unit_pdf")
        .unwrap();

    // Verify info reports scaled physical dimensions (100*3, 200*3) = (300, 600)
    let info = embedder.get_pdf_info(&id).unwrap();
    assert_eq!(info.page_dimensions[0], (300.0, 600.0));

    // Embed into target doc
    let mut target_doc = Document::with_version("1.6");
    let options = EmbedOptions::default();
    let result = embedder.embed_pdf(&mut target_doc, &id, &options).unwrap();

    let xobj_ref = result.xobject_resources.values().next().unwrap();
    if let Object::Reference(xobj_id) = xobj_ref {
        let xobj = target_doc.get_object(*xobj_id).unwrap();
        let stream = xobj.as_stream().unwrap();
        let bbox = stream.dict.get(b"BBox").unwrap().as_array().unwrap();
        let to_f32 = |o: &Object| match o {
            Object::Real(v) => *v as f32,
            Object::Integer(v) => *v as f32,
            _ => 0.0,
        };
        assert_eq!(to_f32(&bbox[2]), 300.0);
        assert_eq!(to_f32(&bbox[3]), 600.0);

        let decompressed = stream.decompressed_content().unwrap();
        let content_str = String::from_utf8_lossy(&decompressed);
        assert!(
            content_str.starts_with("3 0 0 3 0 0 cm"),
            "Expected content to start with UserUnit cm transform, got: {}",
            content_str
        );
    } else {
        panic!("Expected XObject reference");
    }
}

#[test]
fn test_user_unit_with_rotation() {
    ensure_output_dir();

    // Test 90° CW with UserUnit 2.0 (raw MediaBox 300x400)
    // Physical unrotated: 600x800. After 90° CW: width = 800, height = 600.
    let pdf_bytes_90 = create_custom_test_pdf(
        (0.0, 0.0, 300.0, 400.0),
        Some(2.0),
        Some(90),
        false,
        vec![],
    );

    let mut embedder = PdfEmbedder::new();
    let id_90 = embedder
        .load_pdf_from_bytes(&pdf_bytes_90, "rot90_uu_pdf")
        .unwrap();

    let info = embedder.get_pdf_info(&id_90).unwrap();
    assert_eq!(info.page_dimensions[0], (800.0, 600.0));

    let mut target_doc = Document::with_version("1.6");
    let result_90 = embedder
        .embed_pdf(&mut target_doc, &id_90, &EmbedOptions::default())
        .unwrap();
    let xobj_ref = result_90.xobject_resources.values().next().unwrap();
    if let Object::Reference(xobj_id) = xobj_ref {
        let xobj = target_doc.get_object(*xobj_id).unwrap();
        let stream = xobj.as_stream().unwrap();
        let bbox = stream.dict.get(b"BBox").unwrap().as_array().unwrap();
        let to_f32 = |o: &Object| match o {
            Object::Real(v) => *v as f32,
            Object::Integer(v) => *v as f32,
            _ => 0.0,
        };
        assert_eq!(to_f32(&bbox[2]), 800.0);
        assert_eq!(to_f32(&bbox[3]), 600.0);

        let decompressed = stream.decompressed_content().unwrap();
        let content_str = String::from_utf8_lossy(&decompressed);
        assert!(
            content_str.starts_with("0 -2 2 0 0 600 cm"),
            "Expected 90 CW + UserUnit transform, got: {}",
            content_str
        );
    }

    // Test 180° with UserUnit 2.0 (raw MediaBox 300x400)
    let pdf_bytes_180 = create_custom_test_pdf(
        (0.0, 0.0, 300.0, 400.0),
        Some(2.0),
        Some(180),
        false,
        vec![],
    );
    let id_180 = embedder
        .load_pdf_from_bytes(&pdf_bytes_180, "rot180_uu_pdf")
        .unwrap();
    let info = embedder.get_pdf_info(&id_180).unwrap();
    assert_eq!(info.page_dimensions[0], (600.0, 800.0));

    let result_180 = embedder
        .embed_pdf(&mut target_doc, &id_180, &EmbedOptions::default())
        .unwrap();
    let xobj_ref = result_180.xobject_resources.values().next().unwrap();
    if let Object::Reference(xobj_id) = xobj_ref {
        let xobj = target_doc.get_object(*xobj_id).unwrap();
        let stream = xobj.as_stream().unwrap();
        let decompressed = stream.decompressed_content().unwrap();
        let content_str = String::from_utf8_lossy(&decompressed);
        assert!(
            content_str.starts_with("-2 0 0 -2 600 800 cm"),
            "Expected 180 + UserUnit transform, got: {}",
            content_str
        );
    }

    // Test 270° with UserUnit 2.0 (raw MediaBox 300x400)
    let pdf_bytes_270 = create_custom_test_pdf(
        (0.0, 0.0, 300.0, 400.0),
        Some(2.0),
        Some(270),
        false,
        vec![],
    );
    let id_270 = embedder
        .load_pdf_from_bytes(&pdf_bytes_270, "rot270_uu_pdf")
        .unwrap();
    let info = embedder.get_pdf_info(&id_270).unwrap();
    assert_eq!(info.page_dimensions[0], (800.0, 600.0));

    let result_270 = embedder
        .embed_pdf(&mut target_doc, &id_270, &EmbedOptions::default())
        .unwrap();
    let xobj_ref = result_270.xobject_resources.values().next().unwrap();
    if let Object::Reference(xobj_id) = xobj_ref {
        let xobj = target_doc.get_object(*xobj_id).unwrap();
        let stream = xobj.as_stream().unwrap();
        let decompressed = stream.decompressed_content().unwrap();
        let content_str = String::from_utf8_lossy(&decompressed);
        assert!(
            content_str.starts_with("0 2 -2 0 800 0 cm"),
            "Expected 270 + UserUnit transform, got: {}",
            content_str
        );
    }
}

#[test]
fn test_user_unit_full_page_options() {
    ensure_output_dir();

    // Create a 306x396 PDF with UserUnit 2.0 (physical 612x792 US Letter)
    let pdf_bytes = create_custom_test_pdf(
        (0.0, 0.0, 306.0, 396.0),
        Some(2.0),
        None,
        false,
        vec![],
    );

    let mut embedder = PdfEmbedder::new();
    let id = embedder
        .load_pdf_from_bytes(&pdf_bytes, "user_unit_full_page")
        .unwrap();

    let mut target_doc = Document::with_version("1.6");
    // full_page_options(612.0, 792.0) should fit 1:1 without shrinking
    let options = hipdf::embed_pdf::EmbedUtils::full_page_options(612.0, 792.0);
    let result = embedder.embed_pdf(&mut target_doc, &id, &options).unwrap();

    // Check that placement operations have scale 1.0 (1 0 0 1 0 0 cm)
    let cm_op = result
        .operations
        .iter()
        .find(|op| op.operator == "cm")
        .expect("cm operator not found");

    let sx = match &cm_op.operands[0] {
        Object::Real(v) => *v as f32,
        Object::Integer(v) => *v as f32,
        _ => 0.0,
    };
    let sy = match &cm_op.operands[3] {
        Object::Real(v) => *v as f32,
        Object::Integer(v) => *v as f32,
        _ => 0.0,
    };

    assert!((sx - 1.0).abs() < 1e-4, "Expected sx=1.0, got {}", sx);
    assert!((sy - 1.0).abs() < 1e-4, "Expected sy=1.0, got {}", sy);
}
