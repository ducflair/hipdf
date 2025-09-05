use lopdf::{content::Content, dictionary, Dictionary, Document, Object, Stream};
use std::collections::HashMap;
use std::io::Result;
use std::sync::Arc;

use hipdf::hatching::{
    CustomPattern, HatchConfig, HatchStyle, HatchingManager, PatternedShapeBuilder,
    ProceduralPattern, Transform,
};

#[test]
fn test_hatching_patterns_showcase() -> Result<()> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    // Setup basic document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // Add font for labels
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Initialize the hatching manager
    let mut hatching_manager = HatchingManager::new();

    // Create page resources dictionary
    let mut resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "Pattern" => Dictionary::new(),
    };

    // Define all 20 different patterns with various configurations
    let patterns: Vec<(String, HatchConfig)> = vec![
        // 1. Basic diagonal right hatching
        (
            "diagonal_right".to_string(),
            HatchConfig::new(HatchStyle::DiagonalRight)
                .with_spacing(8.0)
                .with_line_width(0.5)
                .with_color(0.0, 0.0, 0.0),
        ),
        // 2. Diagonal left with blue color
        (
            "diagonal_left_blue".to_string(),
            HatchConfig::new(HatchStyle::DiagonalLeft)
                .with_spacing(10.0)
                .with_line_width(1.0)
                .with_color(0.0, 0.0, 1.0),
        ),
        // 3. Horizontal lines with background
        (
            "horizontal_bg".to_string(),
            HatchConfig::new(HatchStyle::Horizontal)
                .with_spacing(6.0)
                .with_line_width(0.3)
                .with_color(0.5, 0.0, 0.0)
                .with_background(1.0, 0.95, 0.9),
        ),
        // 4. Vertical thick lines
        (
            "vertical_thick".to_string(),
            HatchConfig::new(HatchStyle::Vertical)
                .with_spacing(12.0)
                .with_line_width(2.0)
                .with_color(0.2, 0.2, 0.2),
        ),
        // 5. Cross hatch (grid)
        (
            "cross_grid".to_string(),
            HatchConfig::new(HatchStyle::Cross)
                .with_spacing(8.0)
                .with_line_width(0.5)
                .with_color(0.3, 0.3, 0.3),
        ),
        // 6. Diagonal cross (X pattern)
        (
            "diagonal_cross".to_string(),
            HatchConfig::new(HatchStyle::DiagonalCross)
                .with_spacing(10.0)
                .with_line_width(0.7)
                .with_color(0.0, 0.5, 0.0),
        ),
        // 7. Dots pattern
        (
            "dots".to_string(),
            HatchConfig::new(HatchStyle::Dots)
                .with_spacing(5.0)
                .with_color(0.0, 0.0, 0.0),
        ),
        // 8. Checkerboard
        (
            "checkerboard".to_string(),
            HatchConfig::new(HatchStyle::Checkerboard)
                .with_spacing(10.0)
                .with_color(0.0, 0.0, 0.0)
                .with_background(1.0, 1.0, 1.0),
        ),
        // 9. Brick pattern
        (
            "brick".to_string(),
            HatchConfig::new(HatchStyle::Brick)
                .with_spacing(15.0)
                .with_line_width(1.0)
                .with_color(0.6, 0.3, 0.1),
        ),
        // 10. Hexagonal pattern
        (
            "hexagonal".to_string(),
            HatchConfig::new(HatchStyle::Hexagonal)
                .with_spacing(20.0)
                .with_line_width(0.8)
                .with_color(0.0, 0.3, 0.6),
        ),
        // 11. Wave pattern
        (
            "wave".to_string(),
            HatchConfig::new(HatchStyle::Wave)
                .with_spacing(15.0)
                .with_line_width(1.5)
                .with_color(0.0, 0.5, 0.8),
        ),
        // 12. Zigzag pattern
        (
            "zigzag".to_string(),
            HatchConfig::new(HatchStyle::Zigzag)
                .with_spacing(12.0)
                .with_line_width(1.0)
                .with_color(0.8, 0.0, 0.8),
        ),
        // 13. Circles pattern
        (
            "circles".to_string(),
            HatchConfig::new(HatchStyle::Circles)
                .with_spacing(15.0)
                .with_line_width(0.5)
                .with_color(0.5, 0.0, 0.5),
        ),
        // 14. Triangles pattern
        (
            "triangles".to_string(),
            HatchConfig::new(HatchStyle::Triangles)
                .with_spacing(18.0)
                .with_line_width(0.7)
                .with_color(0.0, 0.6, 0.3),
        ),
        // 15. Diamond pattern
        (
            "diamond".to_string(),
            HatchConfig::new(HatchStyle::Diamond)
                .with_spacing(12.0)
                .with_line_width(0.6)
                .with_color(0.7, 0.0, 0.0),
        ),
        // 16. Scales pattern
        (
            "scales".to_string(),
            HatchConfig::new(HatchStyle::Scales)
                .with_spacing(10.0)
                .with_line_width(0.4)
                .with_color(0.0, 0.4, 0.4),
        ),
        // 17. Spiral pattern
        (
            "spiral".to_string(),
            HatchConfig::new(HatchStyle::Spiral)
                .with_spacing(25.0)
                .with_line_width(0.8)
                .with_color(0.3, 0.3, 0.0),
        ),
        // 18. Dotted grid
        (
            "dotted_grid".to_string(),
            HatchConfig::new(HatchStyle::DottedGrid)
                .with_spacing(8.0)
                .with_line_width(0.3)
                .with_color(0.4, 0.4, 0.4),
        ),
        // 19. Concentric circles
        (
            "concentric".to_string(),
            HatchConfig::new(HatchStyle::ConcentricCircles)
                .with_spacing(20.0)
                .with_line_width(0.5)
                .with_color(0.0, 0.2, 0.6),
        ),
        // 20. Wood grain pattern
        (
            "wood_grain".to_string(),
            HatchConfig::new(HatchStyle::WoodGrain)
                .with_spacing(30.0)
                .with_line_width(0.7)
                .with_color(0.4, 0.2, 0.0),
        ),
    ];

    // Create all patterns and store their IDs
    let mut pattern_map = HashMap::new();
    for (name, config) in &patterns {
        let (pattern_id, pattern_name) = hatching_manager.create_pattern(&mut doc, config);
        hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
        pattern_map.insert(name.clone(), pattern_name);
    }

    // Create page content
    let mut shape_builder = PatternedShapeBuilder::new();
    let mut text_ops = Vec::new();

    // Add title
    text_ops.push(lopdf::content::Operation::new("0 g", vec![])); // Set solid black fill for text
    text_ops.push(lopdf::content::Operation::new("BT", vec![]));
    text_ops.push(lopdf::content::Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Td",
        vec![200.into(), 800.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Tj",
        vec![Object::string_literal("20 Different Hatching Patterns")],
    ));
    text_ops.push(lopdf::content::Operation::new("ET", vec![]));

    // Create a 5x4 grid of pattern samples
    let start_x = 50.0;
    let start_y = 700.0;
    let cell_width = 100.0;
    let cell_height = 100.0;
    let spacing = 10.0;

    for (index, (name, _config)) in patterns.iter().enumerate() {
        let row = index / 5;
        let col = index % 5;

        let x = start_x + col as f32 * (cell_width + spacing);
        let y = start_y - row as f32 * (cell_height + spacing + 20.0); // Extra space for labels

        // Draw rectangle with pattern
        if let Some(pattern_name) = pattern_map.get(name) {
            shape_builder.rectangle(x, y, cell_width, cell_height, pattern_name);
        }

        // Add label below each pattern
        text_ops.push(lopdf::content::Operation::new("0 g", vec![])); // Set solid black fill for text
        text_ops.push(lopdf::content::Operation::new("BT", vec![]));
        text_ops.push(lopdf::content::Operation::new(
            "Tf",
            vec![Object::Name(b"F1".to_vec()), 8.into()],
        ));
        text_ops.push(lopdf::content::Operation::new(
            "Td",
            vec![x.into(), (y - 15.0).into()],
        ));

        // Format the label nicely
        let label = name.replace('_', " ");
        let label = label
            .chars()
            .take(15) // Limit length to fit in cell
            .collect::<String>();

        text_ops.push(lopdf::content::Operation::new(
            "Tj",
            vec![Object::string_literal(label.as_bytes().to_vec())],
        ));
        text_ops.push(lopdf::content::Operation::new("ET", vec![]));
    }

    // Add demonstration shapes with various patterns at the bottom
    let demo_y = 150.0;

    // Circle with spiral pattern
    if let Some(pattern_name) = pattern_map.get("spiral") {
        shape_builder.circle(100.0, demo_y, 30.0, pattern_name);
    }

    // Triangle with hexagonal pattern
    if let Some(pattern_name) = pattern_map.get("hexagonal") {
        shape_builder.triangle(
            200.0,
            demo_y - 30.0,
            250.0,
            demo_y + 30.0,
            150.0,
            demo_y + 30.0,
            pattern_name,
        );
    }

    // Large rectangle with wood grain
    if let Some(pattern_name) = pattern_map.get("wood_grain") {
        shape_builder.rectangle(300.0, demo_y - 30.0, 150.0, 60.0, pattern_name);
    }

    // Label for demo shapes
    text_ops.push(lopdf::content::Operation::new("0 g", vec![])); // Set solid black fill for text
    text_ops.push(lopdf::content::Operation::new("BT", vec![]));
    text_ops.push(lopdf::content::Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 10.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Td",
        vec![50.into(), 100.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Tj",
        vec![Object::string_literal(
            "Demo: Different shapes with various patterns",
        )],
    ));
    text_ops.push(lopdf::content::Operation::new("ET", vec![]));

    // Combine all operations
    let mut all_operations = shape_builder.build();
    all_operations.extend(text_ops);

    // Create content stream
    let content = Content {
        operations: all_operations,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Create the page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()], // A4
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Update pages dictionary
    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save the PDF
    let output_path = std::path::Path::new("tests/outputs/hatching_patterns_integration_test.pdf");
    let absolute_path = std::env::current_dir()?.join(output_path);
    doc.save(&absolute_path)?;

    println!("Successfully created PDF with 20 different hatching patterns!");
    println!("Output: {}", absolute_path.display());
    println!("\nPatterns included:");
    for (i, (name, _)) in patterns.iter().enumerate() {
        println!("  {}. {}", i + 1, name.replace('_', " "));
    }

    // Verify the file was created
    assert!(absolute_path.exists(), "Output PDF file should exist");

    Ok(())
}

#[test]
fn test_custom_patterns_showcase() -> Result<()> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let mut hatching_manager = HatchingManager::new();
    let mut resources = dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "Pattern" => Dictionary::new(),
    };

    let mut pattern_map = HashMap::new();

    // Example 1: Simple custom pattern - Random dots
    let (pattern_id, pattern_name) =
        hatching_manager.create_custom_pattern(&mut doc, 20.0, 20.0, |builder| {
            builder
                .set_fill_color(0.8, 0.2, 0.2)
                .circle(5.0, 5.0, 2.0)
                .fill()
                .circle(15.0, 8.0, 1.5)
                .fill()
                .circle(10.0, 15.0, 2.5)
                .fill()
        });
    hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
    pattern_map.insert("custom_dots", pattern_name);

    // Example 2: Parametric star pattern
    let (pattern_id, pattern_name) =
        hatching_manager.create_custom_pattern(&mut doc, 30.0, 30.0, |builder| {
            let cx = 15.0;
            let cy = 15.0;
            let outer = 12.0;
            let inner = 5.0;
            let points = 5;

            let mut star_points = Vec::new();
            for i in 0..points * 2 {
                let angle = i as f32 * std::f32::consts::PI / points as f32;
                let r = if i % 2 == 0 { outer } else { inner };
                star_points.push((cx + r * angle.cos(), cy + r * angle.sin()));
            }

            builder
                .set_fill_color(1.0, 0.8, 0.0)
                .polygon(&star_points)
                .fill()
        });
    hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
    pattern_map.insert("stars", pattern_name);

    // Example 3: Gradient-like pattern using lines
    let (pattern_id, pattern_name) =
        hatching_manager.create_custom_pattern(&mut doc, 10.0, 10.0, |builder| {
            builder.set_line_width(0.5);
            for i in 0..10 {
                let intensity = i as f32 / 10.0;
                builder
                    .set_stroke_color(intensity, 0.0, 1.0 - intensity)
                    .move_to(i as f32, 0.0)
                    .line_to(i as f32, 10.0)
                    .stroke();
            }
            builder
        });
    hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
    pattern_map.insert("gradient_lines", pattern_name);

    // Example 4: Complex geometric pattern
    let (pattern_id, pattern_name) =
        hatching_manager.create_custom_pattern(&mut doc, 40.0, 40.0, |builder| {
            builder
                .set_stroke_color(0.0, 0.4, 0.6)
                .set_line_width(1.0)
                // Outer square
                .rectangle(5.0, 5.0, 30.0, 30.0)
                .stroke()
                // Inner rotated square
                .push_transform(Transform {
                    translate: (20.0, 20.0),
                    rotate: 45.0,
                    scale: (0.7, 0.7),
                })
                .rectangle(-10.0, -10.0, 20.0, 20.0)
                .stroke()
                .pop_transform()
                // Center circle
                .circle(20.0, 20.0, 5.0)
                .stroke()
        });
    hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
    pattern_map.insert("geometric", pattern_name);

    // Example 6: Procedural pattern - Sierpinski-like
    let sierpinski = HatchConfig::new(HatchStyle::Custom(CustomPattern::Procedural(
        ProceduralPattern {
            sampler: Arc::new(|x, y, _t| {
                let xi = x as i32;
                let yi = y as i32;
                (xi & yi) == 0
            }),
            resolution: 16,
            fill: true,
        },
    )));
    let (pattern_id, pattern_name) = hatching_manager.create_pattern(&mut doc, &sierpinski);
    hatching_manager.add_pattern_to_resources(&mut resources, &pattern_name, pattern_id);
    pattern_map.insert("sierpinski", pattern_name);

    // Create page content
    let mut shape_builder = PatternedShapeBuilder::new();
    let mut text_ops = Vec::new();

    // Title
    text_ops.push(lopdf::content::Operation::new("0 g", vec![])); // Set solid black fill for text
    text_ops.push(lopdf::content::Operation::new("BT", vec![]));
    text_ops.push(lopdf::content::Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 16.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Td",
        vec![150.into(), 800.into()],
    ));
    text_ops.push(lopdf::content::Operation::new(
        "Tj",
        vec![Object::string_literal("Custom Pattern Examples")],
    ));
    text_ops.push(lopdf::content::Operation::new("ET", vec![]));

    // Display all custom patterns
    let patterns = vec![
        ("custom_dots", "Random Dots"),
        ("stars", "Star Pattern"),
        ("gradient_lines", "Gradient Lines"),
        ("geometric", "Geometric"),
        ("sierpinski", "Sierpinski"),
    ];

    let start_x = 50.0;
    let start_y = 700.0;
    let cell_size = 80.0;
    let spacing = 20.0;

    for (index, (key, label)) in patterns.iter().enumerate() {
        let x = start_x + (index % 4) as f32 * (cell_size + spacing);
        let y = start_y - (index / 4) as f32 * (cell_size + spacing + 20.0);

        if let Some(pattern_name) = pattern_map.get(*key) {
            shape_builder.rectangle(x, y, cell_size, cell_size, pattern_name);
        }

        // Add label
        text_ops.push(lopdf::content::Operation::new("0 g", vec![])); // Set solid black fill for text
        text_ops.push(lopdf::content::Operation::new("BT", vec![]));
        text_ops.push(lopdf::content::Operation::new(
            "Tf",
            vec![Object::Name(b"F1".to_vec()), 10.into()],
        ));
        text_ops.push(lopdf::content::Operation::new(
            "Td",
            vec![x.into(), (y - 15.0).into()],
        ));
        text_ops.push(lopdf::content::Operation::new(
            "Tj",
            vec![Object::string_literal(label.as_bytes().to_vec())],
        ));
        text_ops.push(lopdf::content::Operation::new("ET", vec![]));
    }

    // Combine operations
    let mut all_operations = shape_builder.build();
    all_operations.extend(text_ops);

    let content = Content {
        operations: all_operations,
    };
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

    let output_path =
        std::path::Path::new("tests/outputs/hatching_custom_patterns_integration_test.pdf");
    let absolute_path = std::env::current_dir()?.join(output_path);
    doc.save(&absolute_path)?;

    println!("Successfully created PDF with custom patterns!");
    println!("Output: {}", absolute_path.display());
    println!("\nCustom patterns included:");
    for (key, label) in &patterns {
        println!("  - {}: {}", key, label);
    }

    // Verify the file was created
    assert!(absolute_path.exists(), "Output PDF file should exist");

    Ok(())
}
