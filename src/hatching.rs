//! Hatching and pattern support for PDF documents
//!
//! This module provides a high-level API for creating various hatching patterns,
//! crosshatching, and other fill patterns for shapes in PDF documents.

use lopdf::{
    content::{Content, Operation},
    dictionary, Dictionary, Document, Object, ObjectId, Stream,
};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;

/// Represents a hatching pattern style
#[derive(Debug, Clone)]
pub enum HatchStyle {
    /// Single diagonal lines (/)
    DiagonalRight,
    /// Single diagonal lines (KATEX_INLINE_CLOSE
    DiagonalLeft,
    /// Horizontal lines (-)
    Horizontal,
    /// Vertical lines (|)
    Vertical,
    /// Cross hatch (+)
    Cross,
    /// Diagonal cross hatch (X)
    DiagonalCross,
    /// Dots pattern
    Dots,
    /// Checkerboard pattern
    Checkerboard,
    /// Brick pattern
    Brick,
    /// Hexagonal pattern
    Hexagonal,
    /// Wave pattern
    Wave,
    /// Zigzag pattern
    Zigzag,
    /// Circles pattern
    Circles,
    /// Triangles pattern
    Triangles,
    /// Diamond pattern
    Diamond,
    /// Scales pattern
    Scales,
    /// Spiral pattern
    Spiral,
    /// Grid with dots at intersections
    DottedGrid,
    /// Concentric circles
    ConcentricCircles,
    /// Wood grain pattern
    WoodGrain,
    /// Custom user-defined pattern
    Custom(CustomPattern),
}

/// Represents a custom pattern defined by user-provided drawing commands
#[derive(Clone)]
pub enum CustomPattern {
    /// A simple function that generates operations
    Simple(Arc<dyn Fn(f32, f32) -> Vec<Operation> + Send + Sync>),
    /// A parameterized pattern with custom data
    Parametric(
        Arc<dyn Fn(f32, f32, &PatternParams) -> Vec<Operation> + Send + Sync>,
        PatternParams,
    ),
    /// A procedural pattern based on mathematical functions
    Procedural(ProceduralPattern),
    /// A composite pattern that combines multiple patterns
    Composite(Vec<PatternElement>),
}

impl std::fmt::Debug for CustomPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomPattern::Simple(_) => f.debug_tuple("Simple").field(&"<function>").finish(),
            CustomPattern::Parametric(_, params) => f
                .debug_tuple("Parametric")
                .field(&"<function>")
                .field(params)
                .finish(),
            CustomPattern::Procedural(proc) => f.debug_tuple("Procedural").field(proc).finish(),
            CustomPattern::Composite(elements) => {
                f.debug_tuple("Composite").field(elements).finish()
            }
        }
    }
}

/// Parameters for custom patterns
#[derive(Debug, Clone)]
pub struct PatternParams {
    pub data: HashMap<String, f32>,
    pub colors: Vec<(f32, f32, f32)>,
    pub strings: HashMap<String, String>,
}

impl Default for PatternParams {
    fn default() -> Self {
        PatternParams {
            data: HashMap::new(),
            colors: vec![(0.0, 0.0, 0.0)],
            strings: HashMap::new(),
        }
    }
}

impl PatternParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_param(mut self, key: &str, value: f32) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.colors.push((r, g, b));
        self
    }

    pub fn get(&self, key: &str) -> f32 {
        *self.data.get(key).unwrap_or(&0.0)
    }
}

/// Procedural pattern generator using mathematical functions
#[derive(Clone)]
pub struct ProceduralPattern {
    pub sampler: Arc<dyn Fn(f32, f32, f32) -> bool + Send + Sync>,
    pub resolution: usize,
    pub fill: bool,
}

impl std::fmt::Debug for ProceduralPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProceduralPattern")
            .field("resolution", &self.resolution)
            .field("fill", &self.fill)
            .field("sampler", &"<function>")
            .finish()
    }
}

/// Element in a composite pattern
#[derive(Debug, Clone)]
pub struct PatternElement {
    pub operations: Vec<Operation>,
    pub transform: Option<Transform>,
    pub opacity: f32,
}

/// Transform for pattern elements
#[derive(Debug, Clone)]
pub struct Transform {
    pub translate: (f32, f32),
    pub rotate: f32,
    pub scale: (f32, f32),
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            translate: (0.0, 0.0),
            rotate: 0.0,
            scale: (1.0, 1.0),
        }
    }

    pub fn to_operations(&self) -> Vec<Operation> {
        let mut ops = vec![];
        let (tx, ty) = self.translate;
        let angle_rad = self.rotate * PI / 180.0;
        let (sx, sy) = self.scale;

        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        ops.push(Operation::new(
            "cm",
            vec![
                (sx * cos).into(),
                (sx * sin).into(),
                (-sy * sin).into(),
                (sy * cos).into(),
                tx.into(),
                ty.into(),
            ],
        ));

        ops
    }
}

/// Builder for creating custom patterns with a fluent API
pub struct CustomPatternBuilder {
    operations: Vec<Operation>,
    current_path: Vec<(String, Vec<Object>)>,
    transform_stack: Vec<Transform>,
}

impl Default for CustomPatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPatternBuilder {
    pub fn new() -> Self {
        CustomPatternBuilder {
            operations: Vec::new(),
            current_path: Vec::new(),
            transform_stack: Vec::new(),
        }
    }

    // Path building methods
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.current_path
            .push(("m".to_string(), vec![x.into(), y.into()]));
        self
    }

    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.current_path
            .push(("l".to_string(), vec![x.into(), y.into()]));
        self
    }

    pub fn curve_to(
        &mut self,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.current_path.push((
            "c".to_string(),
            vec![
                cx1.into(),
                cy1.into(),
                cx2.into(),
                cy2.into(),
                x.into(),
                y.into(),
            ],
        ));
        self
    }

    pub fn close_path(&mut self) -> &mut Self {
        self.current_path.push(("h".to_string(), vec![]));
        self
    }

    // Drawing methods
    pub fn stroke(&mut self) -> &mut Self {
        self.flush_path();
        self.operations.push(Operation::new("S", vec![]));
        self
    }

    pub fn fill(&mut self) -> &mut Self {
        self.flush_path();
        self.operations.push(Operation::new("f", vec![]));
        self
    }

    pub fn fill_stroke(&mut self) -> &mut Self {
        self.flush_path();
        self.operations.push(Operation::new("B", vec![]));
        self
    }

    // Style methods
    pub fn set_line_width(&mut self, width: f32) -> &mut Self {
        self.operations
            .push(Operation::new("w", vec![width.into()]));
        self
    }

    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.operations
            .push(Operation::new("RG", vec![r.into(), g.into(), b.into()]));
        self
    }

    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.operations
            .push(Operation::new("rg", vec![r.into(), g.into(), b.into()]));
        self
    }

    pub fn set_dash_pattern(&mut self, pattern: Vec<f32>, phase: f32) -> &mut Self {
        let array: Vec<Object> = pattern.iter().map(|&v| v.into()).collect();
        self.operations.push(Operation::new(
            "d",
            vec![Object::Array(array), phase.into()],
        ));
        self
    }

    // Transform methods
    pub fn push_transform(&mut self, transform: Transform) -> &mut Self {
        self.operations.push(Operation::new("q", vec![])); // Save graphics state
        self.operations.extend(transform.to_operations());
        self.transform_stack.push(transform);
        self
    }

    pub fn pop_transform(&mut self) -> &mut Self {
        if !self.transform_stack.is_empty() {
            self.transform_stack.pop();
            self.operations.push(Operation::new("Q", vec![])); // Restore graphics state
        }
        self
    }

    // Convenience shape methods
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.operations.push(Operation::new(
            "re",
            vec![x.into(), y.into(), width.into(), height.into()],
        ));
        self
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) -> &mut Self {
        let k = 0.552_284_8;
        self.move_to(cx + r, cy)
            .curve_to(cx + r, cy + k * r, cx + k * r, cy + r, cx, cy + r)
            .curve_to(cx - k * r, cy + r, cx - r, cy + k * r, cx - r, cy)
            .curve_to(cx - r, cy - k * r, cx - k * r, cy - r, cx, cy - r)
            .curve_to(cx + k * r, cy - r, cx + r, cy - k * r, cx + r, cy)
            .close_path()
    }

    pub fn polygon(&mut self, points: &[(f32, f32)]) -> &mut Self {
        if let Some((first, rest)) = points.split_first() {
            self.move_to(first.0, first.1);
            for point in rest {
                self.line_to(point.0, point.1);
            }
            self.close_path();
        }
        self
    }

    // Utility methods
    fn flush_path(&mut self) {
        for (op, args) in &self.current_path {
            self.operations.push(Operation::new(op, args.clone()));
        }
        self.current_path.clear();
    }

    pub fn add_operation(&mut self, op: Operation) -> &mut Self {
        self.operations.push(op);
        self
    }

    pub fn add_operations(&mut self, ops: Vec<Operation>) -> &mut Self {
        self.operations.extend(ops);
        self
    }

    pub fn build(mut self) -> Vec<Operation> {
        self.flush_path();
        self.operations
    }
}

/// Configuration for a hatching pattern
#[derive(Debug, Clone)]
pub struct HatchConfig {
    /// The style of hatching
    pub style: HatchStyle,
    /// Spacing between lines/elements (in points)
    pub spacing: f32,
    /// Line width (in points)
    pub line_width: f32,
    /// Primary color (RGB)
    pub color: (f32, f32, f32),
    /// Background color (RGB), None for transparent
    pub background: Option<(f32, f32, f32)>,
    /// Angle offset in degrees (for rotating patterns)
    pub angle: f32,
    /// Scale factor for the pattern
    pub scale: f32,
}

impl Default for HatchConfig {
    fn default() -> Self {
        HatchConfig {
            style: HatchStyle::DiagonalRight,
            spacing: 5.0,
            line_width: 0.5,
            color: (0.0, 0.0, 0.0),
            background: None,
            angle: 0.0,
            scale: 1.0,
        }
    }
}

impl HatchConfig {
    /// Creates a new HatchConfig with the specified style
    pub fn new(style: HatchStyle) -> Self {
        HatchConfig {
            style,
            ..Default::default()
        }
    }

    /// Builder method to set spacing
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Builder method to set line width
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Builder method to set color
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = (r, g, b);
        self
    }

    /// Builder method to set background color
    pub fn with_background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background = Some((r, g, b));
        self
    }

    /// Builder method to set angle
    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    /// Builder method to set scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// Manager for creating and managing hatching patterns in a PDF
pub struct HatchingManager {
    /// Counter for generating unique pattern names
    pattern_counter: usize,
}

impl Default for HatchingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HatchingManager {
    /// Creates a new HatchingManager
    pub fn new() -> Self {
        HatchingManager { pattern_counter: 0 }
    }

    /// Creates a pattern object in the PDF document
    /// Returns the pattern ID and name to use in content streams
    pub fn create_pattern(
        &mut self,
        doc: &mut Document,
        config: &HatchConfig,
    ) -> (ObjectId, String) {
        self.pattern_counter += 1;
        let pattern_name = format!("P{}", self.pattern_counter);

        // Calculate pattern bounds based on style and config
        let (width, height) = self.calculate_pattern_bounds(config);

        // Generate pattern content
        let operations = self.generate_pattern_operations(config, width, height);
        let content = Content { operations };

        // Create pattern stream
        let pattern_dict = dictionary! {
            "Type" => "Pattern",
            "PatternType" => 1i32,  // Tiling pattern
            "PaintType" => 1i32,    // Colored pattern
            "TilingType" => 1i32,    // Constant spacing
            "BBox" => vec![0.into(), 0.into(), width.into(), height.into()],
            "XStep" => Object::Real(width),
            "YStep" => Object::Real(height),
            "Resources" => dictionary!{},
        };

        let pattern_stream = Stream::new(pattern_dict, content.encode().unwrap());
        let pattern_id = doc.add_object(pattern_stream);

        // Return the pattern ID and name
        (pattern_id, pattern_name)
    }

    /// Adds a pattern to a page's resources
    pub fn add_pattern_to_resources(
        &self,
        resources: &mut Dictionary,
        pattern_name: &str,
        pattern_id: ObjectId,
    ) {
        // Check if Pattern dictionary exists, create it if not
        if !resources.has(b"Pattern") {
            resources.set("Pattern", Dictionary::new());
        }

        if let Ok(Object::Dictionary(ref mut patterns)) = resources.get_mut(b"Pattern") {
            patterns.set(pattern_name, Object::Reference(pattern_id));
        }
    }

    /// Calculate pattern bounds based on style
    fn calculate_pattern_bounds(&self, config: &HatchConfig) -> (f32, f32) {
        let base_size = config.spacing * config.scale;

        match config.style {
            HatchStyle::Checkerboard => (base_size * 2.0, base_size * 2.0),
            HatchStyle::Brick => (base_size * 4.0, base_size * 2.0),
            HatchStyle::Hexagonal => (base_size * 3.0, base_size * 2.6),
            HatchStyle::Circles | HatchStyle::ConcentricCircles => {
                (base_size * 2.0, base_size * 2.0)
            }
            HatchStyle::Diamond => (base_size * 2.0, base_size * 2.0),
            HatchStyle::Scales => (base_size * 2.0, base_size * 2.0),
            HatchStyle::Triangles => (base_size * 2.0, base_size * 1.73),
            HatchStyle::Wave => (base_size * 4.0, base_size * 2.0),
            HatchStyle::Zigzag => (base_size * 4.0, base_size),
            HatchStyle::Spiral => (base_size * 4.0, base_size * 4.0),
            HatchStyle::WoodGrain => (base_size * 8.0, base_size * 2.0),
            HatchStyle::Custom(_) => (base_size, base_size), // Default size for custom patterns
            _ => (base_size, base_size),
        }
    }

    /// Generate pattern operations based on style
    fn generate_pattern_operations(
        &self,
        config: &HatchConfig,
        width: f32,
        height: f32,
    ) -> Vec<Operation> {
        let mut ops = Vec::new();

        // Add background if specified
        if let Some((r, g, b)) = config.background {
            ops.push(Operation::new("rg", vec![r.into(), g.into(), b.into()]));
            ops.push(Operation::new(
                "re",
                vec![0.into(), 0.into(), width.into(), height.into()],
            ));
            ops.push(Operation::new("f", vec![]));
        }

        // Handle custom patterns
        if let HatchStyle::Custom(ref custom) = config.style {
            match custom {
                CustomPattern::Simple(func) => {
                    ops.extend(func(width, height));
                }
                CustomPattern::Parametric(func, params) => {
                    ops.extend(func(width, height, params));
                }
                CustomPattern::Procedural(proc) => {
                    ops.extend(self.generate_procedural_pattern(proc, width, height));
                }
                CustomPattern::Composite(elements) => {
                    for element in elements {
                        if let Some(ref transform) = element.transform {
                            ops.push(Operation::new("q", vec![]));
                            ops.extend(transform.to_operations());
                        }
                        ops.extend(element.operations.clone());
                        if element.transform.is_some() {
                            ops.push(Operation::new("Q", vec![]));
                        }
                    }
                }
            }
            return ops;
        }

        // Set line width and color
        ops.push(Operation::new("w", vec![config.line_width.into()]));
        let (r, g, b) = config.color;
        ops.push(Operation::new("RG", vec![r.into(), g.into(), b.into()]));
        ops.push(Operation::new("rg", vec![r.into(), g.into(), b.into()]));

        // Apply rotation if specified
        if config.angle != 0.0 {
            let angle_rad = config.angle * PI / 180.0;
            let cos = angle_rad.cos();
            let sin = angle_rad.sin();
            ops.push(Operation::new(
                "cm",
                vec![
                    cos.into(),
                    sin.into(),
                    (-sin).into(),
                    cos.into(),
                    0.into(),
                    0.into(),
                ],
            ));
        }

        // Generate pattern-specific operations
        match config.style {
            HatchStyle::DiagonalRight => self.diagonal_right_ops(&mut ops, width, height),
            HatchStyle::DiagonalLeft => self.diagonal_left_ops(&mut ops, width, height),
            HatchStyle::Horizontal => self.horizontal_ops(&mut ops, width, height),
            HatchStyle::Vertical => self.vertical_ops(&mut ops, width, height),
            HatchStyle::Cross => self.cross_ops(&mut ops, width, height),
            HatchStyle::DiagonalCross => self.diagonal_cross_ops(&mut ops, width, height),
            HatchStyle::Dots => self.dots_ops(&mut ops, width, height, config.spacing),
            HatchStyle::Checkerboard => self.checkerboard_ops(&mut ops, width, height),
            HatchStyle::Brick => self.brick_ops(&mut ops, width, height),
            HatchStyle::Hexagonal => self.hexagonal_ops(&mut ops, width, height),
            HatchStyle::Wave => self.wave_ops(&mut ops, width, height),
            HatchStyle::Zigzag => self.zigzag_ops(&mut ops, width, height),
            HatchStyle::Circles => self.circles_ops(&mut ops, width, height),
            HatchStyle::Triangles => self.triangles_ops(&mut ops, width, height),
            HatchStyle::Diamond => self.diamond_ops(&mut ops, width, height),
            HatchStyle::Scales => self.scales_ops(&mut ops, width, height),
            HatchStyle::Spiral => self.spiral_ops(&mut ops, width, height),
            HatchStyle::DottedGrid => self.dotted_grid_ops(&mut ops, width, height),
            HatchStyle::ConcentricCircles => self.concentric_circles_ops(&mut ops, width, height),
            HatchStyle::WoodGrain => self.wood_grain_ops(&mut ops, width, height),
            HatchStyle::Custom(_) => {} // Custom patterns are handled above
        }

        ops
    }

    fn diagonal_right_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![0.into(), 0.into()]));
        ops.push(Operation::new("l", vec![width.into(), height.into()]));
        ops.push(Operation::new("S", vec![]));
    }

    fn diagonal_left_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![0.into(), height.into()]));
        ops.push(Operation::new("l", vec![width.into(), 0.into()]));
        ops.push(Operation::new("S", vec![]));
    }

    fn horizontal_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![0.into(), (height / 2.0).into()]));
        ops.push(Operation::new(
            "l",
            vec![width.into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    fn vertical_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![(width / 2.0).into(), 0.into()]));
        ops.push(Operation::new(
            "l",
            vec![(width / 2.0).into(), height.into()],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    fn cross_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        self.horizontal_ops(ops, width, height);
        self.vertical_ops(ops, width, height);
    }

    fn diagonal_cross_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        self.diagonal_right_ops(ops, width, height);
        self.diagonal_left_ops(ops, width, height);
    }

    fn dots_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32, spacing: f32) {
        let radius = spacing * 0.2;
        self.circle_at(ops, width / 2.0, height / 2.0, radius);
        ops.push(Operation::new("f", vec![]));
    }

    fn checkerboard_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new(
            "re",
            vec![
                0.into(),
                0.into(),
                (width / 2.0).into(),
                (height / 2.0).into(),
            ],
        ));
        ops.push(Operation::new(
            "re",
            vec![
                (width / 2.0).into(),
                (height / 2.0).into(),
                (width / 2.0).into(),
                (height / 2.0).into(),
            ],
        ));
        ops.push(Operation::new("f", vec![]));
    }

    fn brick_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        // Horizontal lines
        ops.push(Operation::new("m", vec![0.into(), (height / 2.0).into()]));
        ops.push(Operation::new(
            "l",
            vec![width.into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new("S", vec![]));

        // Vertical lines (staggered)
        ops.push(Operation::new("m", vec![(width / 4.0).into(), 0.into()]));
        ops.push(Operation::new(
            "l",
            vec![(width / 4.0).into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new("S", vec![]));

        ops.push(Operation::new(
            "m",
            vec![(width * 3.0 / 4.0).into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new(
            "l",
            vec![(width * 3.0 / 4.0).into(), height.into()],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    fn hexagonal_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        let cx = width / 2.0;
        let cy = height / 2.0;
        let r = width / 3.0;

        ops.push(Operation::new("m", vec![(cx + r).into(), cy.into()]));
        for i in 1..7 {
            let angle = i as f32 * PI / 3.0;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            ops.push(Operation::new("l", vec![x.into(), y.into()]));
        }
        ops.push(Operation::new("S", vec![]));
    }

    fn wave_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![0.into(), (height / 2.0).into()]));
        ops.push(Operation::new(
            "c",
            vec![
                (width / 4.0).into(),
                0.into(),
                (width * 3.0 / 4.0).into(),
                height.into(),
                width.into(),
                (height / 2.0).into(),
            ],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    fn zigzag_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![0.into(), (height / 2.0).into()]));
        ops.push(Operation::new("l", vec![(width / 4.0).into(), 0.into()]));
        ops.push(Operation::new(
            "l",
            vec![(width / 2.0).into(), height.into()],
        ));
        ops.push(Operation::new(
            "l",
            vec![(width * 3.0 / 4.0).into(), 0.into()],
        ));
        ops.push(Operation::new(
            "l",
            vec![width.into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    fn circles_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        let radius = width.min(height) * 0.3;
        self.circle_at(ops, width / 2.0, height / 2.0, radius);
        ops.push(Operation::new("S", vec![]));
    }

    fn triangles_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![(width / 2.0).into(), 0.into()]));
        ops.push(Operation::new("l", vec![0.into(), height.into()]));
        ops.push(Operation::new("l", vec![width.into(), height.into()]));
        ops.push(Operation::new("h", vec![]));
        ops.push(Operation::new("S", vec![]));
    }

    fn diamond_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        ops.push(Operation::new("m", vec![(width / 2.0).into(), 0.into()]));
        ops.push(Operation::new(
            "l",
            vec![width.into(), (height / 2.0).into()],
        ));
        ops.push(Operation::new(
            "l",
            vec![(width / 2.0).into(), height.into()],
        ));
        ops.push(Operation::new("l", vec![0.into(), (height / 2.0).into()]));
        ops.push(Operation::new("h", vec![]));
        ops.push(Operation::new("S", vec![]));
    }

    fn scales_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        let r = width / 2.0;
        self.arc_at(ops, width / 2.0, height, r, 0.0, PI);
        ops.push(Operation::new("S", vec![]));
    }

    fn spiral_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        let cx = width / 2.0;
        let cy = height / 2.0;
        let steps = 20;
        let max_r = width.min(height) / 2.0;

        ops.push(Operation::new("m", vec![cx.into(), cy.into()]));
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let angle = t * 2.0 * PI;
            let r = t * max_r;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            ops.push(Operation::new("l", vec![x.into(), y.into()]));
        }
        ops.push(Operation::new("S", vec![]));
    }

    fn dotted_grid_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        // Grid lines
        self.cross_ops(ops, width, height);
        // Dot at intersection
        let radius = width.min(height) * 0.1;
        self.circle_at(ops, width / 2.0, height / 2.0, radius);
        ops.push(Operation::new("f", vec![]));
    }

    fn concentric_circles_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        let cx = width / 2.0;
        let cy = height / 2.0;
        let max_r = width.min(height) / 2.0;

        for i in 1..=3 {
            let r = max_r * i as f32 / 3.0;
            self.circle_at(ops, cx, cy, r);
            ops.push(Operation::new("S", vec![]));
        }
    }

    fn wood_grain_ops(&self, ops: &mut Vec<Operation>, width: f32, height: f32) {
        for i in 0..3 {
            let y = height * (i as f32 + 0.5) / 3.0;
            ops.push(Operation::new("m", vec![0.into(), y.into()]));
            ops.push(Operation::new(
                "c",
                vec![
                    (width * 0.2).into(),
                    (y - height * 0.1).into(),
                    (width * 0.8).into(),
                    (y + height * 0.1).into(),
                    width.into(),
                    y.into(),
                ],
            ));
            ops.push(Operation::new("S", vec![]));
        }
    }

    // Helper function to draw a circle
    fn circle_at(&self, ops: &mut Vec<Operation>, cx: f32, cy: f32, r: f32) {
        let k = 0.552_284_8;
        ops.push(Operation::new("m", vec![(cx + r).into(), cy.into()]));
        ops.push(Operation::new(
            "c",
            vec![
                (cx + r).into(),
                (cy + k * r).into(),
                (cx + k * r).into(),
                (cy + r).into(),
                cx.into(),
                (cy + r).into(),
            ],
        ));
        ops.push(Operation::new(
            "c",
            vec![
                (cx - k * r).into(),
                (cy + r).into(),
                (cx - r).into(),
                (cy + k * r).into(),
                (cx - r).into(),
                cy.into(),
            ],
        ));
        ops.push(Operation::new(
            "c",
            vec![
                (cx - r).into(),
                (cy - k * r).into(),
                (cx - k * r).into(),
                (cy - r).into(),
                cx.into(),
                (cy - r).into(),
            ],
        ));
        ops.push(Operation::new(
            "c",
            vec![
                (cx + k * r).into(),
                (cy - r).into(),
                (cx + r).into(),
                (cy - k * r).into(),
                (cx + r).into(),
                cy.into(),
            ],
        ));
    }

    // Helper function to draw an arc
    fn arc_at(
        &self,
        ops: &mut Vec<Operation>,
        cx: f32,
        cy: f32,
        r: f32,
        start_angle: f32,
        end_angle: f32,
    ) {
        let start_x = cx + r * start_angle.cos();
        let start_y = cy + r * start_angle.sin();
        let end_x = cx + r * end_angle.cos();
        let end_y = cy + r * end_angle.sin();

        ops.push(Operation::new("m", vec![start_x.into(), start_y.into()]));

        // Simplified arc using cubic bezier
        let control_distance = r * 0.552_284_8;
        let mid_angle = (start_angle + end_angle) / 2.0;
        let _mid_x = cx + r * mid_angle.cos();
        let _mid_y = cy + r * mid_angle.sin();

        ops.push(Operation::new(
            "c",
            vec![
                (start_x + control_distance * (mid_angle - PI / 2.0).cos()).into(),
                (start_y + control_distance * (mid_angle - PI / 2.0).sin()).into(),
                (end_x + control_distance * (mid_angle + PI / 2.0).cos()).into(),
                (end_y + control_distance * (mid_angle + PI / 2.0).sin()).into(),
                end_x.into(),
                end_y.into(),
            ],
        ));
    }

    /// Creates a custom pattern from a builder function
    pub fn create_custom_pattern(
        &mut self,
        doc: &mut Document,
        width: f32,
        height: f32,
        builder_fn: impl FnOnce(&mut CustomPatternBuilder) -> &mut CustomPatternBuilder,
    ) -> (ObjectId, String) {
        self.pattern_counter += 1;
        let pattern_name = format!("P{}", self.pattern_counter);

        let mut builder = CustomPatternBuilder::new();
        builder_fn(&mut builder);
        let operations = builder.build();

        let content = Content { operations };
        let pattern_dict = dictionary! {
            "Type" => "Pattern",
            "PatternType" => 1i32,
            "PaintType" => 1i32,
            "TilingType" => 1i32,
            "BBox" => vec![0.into(), 0.into(), width.into(), height.into()],
            "XStep" => Object::Real(width),
            "YStep" => Object::Real(height),
            "Resources" => dictionary!{},
        };

        let pattern_stream = Stream::new(pattern_dict, content.encode().unwrap());
        let pattern_id = doc.add_object(pattern_stream);

        (pattern_id, pattern_name)
    }

    /// Generate procedural pattern operations
    fn generate_procedural_pattern(
        &self,
        proc: &ProceduralPattern,
        width: f32,
        height: f32,
    ) -> Vec<Operation> {
        let mut ops = Vec::new();
        let step = width.min(height) / proc.resolution as f32;

        for i in 0..proc.resolution {
            for j in 0..proc.resolution {
                let x = i as f32 * step;
                let y = j as f32 * step;
                let t =
                    (i as f32 / proc.resolution as f32 + j as f32 / proc.resolution as f32) / 2.0;

                if (proc.sampler)(x, y, t) {
                    if proc.fill {
                        ops.push(Operation::new(
                            "re",
                            vec![x.into(), y.into(), step.into(), step.into()],
                        ));
                        ops.push(Operation::new("f", vec![]));
                    } else {
                        let cx = x + step / 2.0;
                        let cy = y + step / 2.0;
                        let r = step * 0.3;
                        self.circle_at(&mut ops, cx, cy, r);
                        ops.push(Operation::new("f", vec![]));
                    }
                }
            }
        }

        ops
    }
}

/// Helper functions for using patterns in content streams
pub struct PatternOperations;

impl PatternOperations {
    /// Sets the fill color space to Pattern
    pub fn set_pattern_fill_colorspace() -> Operation {
        Operation::new("cs", vec![Object::Name(b"Pattern".to_vec())])
    }

    /// Sets the stroke color space to Pattern
    pub fn set_pattern_stroke_colorspace() -> Operation {
        Operation::new("CS", vec![Object::Name(b"Pattern".to_vec())])
    }

    /// Sets the current fill pattern
    pub fn set_fill_pattern(pattern_name: &str) -> Operation {
        Operation::new("scn", vec![Object::Name(pattern_name.as_bytes().to_vec())])
    }

    /// Sets the current stroke pattern
    pub fn set_stroke_pattern(pattern_name: &str) -> Operation {
        Operation::new("SCN", vec![Object::Name(pattern_name.as_bytes().to_vec())])
    }
}

/// Convenience builder for creating shapes with patterns
pub struct PatternedShapeBuilder {
    operations: Vec<Operation>,
}

impl Default for PatternedShapeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternedShapeBuilder {
    pub fn new() -> Self {
        PatternedShapeBuilder {
            operations: Vec::new(),
        }
    }

    /// Rectangle with pattern fill
    pub fn rectangle(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        pattern_name: &str,
    ) -> &mut Self {
        self.operations
            .push(PatternOperations::set_pattern_fill_colorspace());
        self.operations
            .push(PatternOperations::set_fill_pattern(pattern_name));
        self.operations.push(Operation::new(
            "re",
            vec![x.into(), y.into(), width.into(), height.into()],
        ));
        self.operations.push(Operation::new("f", vec![]));
        self
    }

    /// Circle with pattern fill
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, pattern_name: &str) -> &mut Self {
        self.operations
            .push(PatternOperations::set_pattern_fill_colorspace());
        self.operations
            .push(PatternOperations::set_fill_pattern(pattern_name));

        let k = 0.552_284_8;
        self.operations
            .push(Operation::new("m", vec![(cx + r).into(), cy.into()]));
        self.operations.push(Operation::new(
            "c",
            vec![
                (cx + r).into(),
                (cy + k * r).into(),
                (cx + k * r).into(),
                (cy + r).into(),
                cx.into(),
                (cy + r).into(),
            ],
        ));
        self.operations.push(Operation::new(
            "c",
            vec![
                (cx - k * r).into(),
                (cy + r).into(),
                (cx - r).into(),
                (cy + k * r).into(),
                (cx - r).into(),
                cy.into(),
            ],
        ));
        self.operations.push(Operation::new(
            "c",
            vec![
                (cx - r).into(),
                (cy - k * r).into(),
                (cx - k * r).into(),
                (cy - r).into(),
                cx.into(),
                (cy - r).into(),
            ],
        ));
        self.operations.push(Operation::new(
            "c",
            vec![
                (cx + k * r).into(),
                (cy - r).into(),
                (cx + r).into(),
                (cy - k * r).into(),
                (cx + r).into(),
                cy.into(),
            ],
        ));
        self.operations.push(Operation::new("f", vec![]));
        self
    }

    /// Triangle with pattern fill
    pub fn triangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        pattern_name: &str,
    ) -> &mut Self {
        self.operations
            .push(PatternOperations::set_pattern_fill_colorspace());
        self.operations
            .push(PatternOperations::set_fill_pattern(pattern_name));
        self.operations
            .push(Operation::new("m", vec![x1.into(), y1.into()]));
        self.operations
            .push(Operation::new("l", vec![x2.into(), y2.into()]));
        self.operations
            .push(Operation::new("l", vec![x3.into(), y3.into()]));
        self.operations.push(Operation::new("h", vec![]));
        self.operations.push(Operation::new("f", vec![]));
        self
    }

    /// Builds the operations
    pub fn build(self) -> Vec<Operation> {
        self.operations
    }
}
