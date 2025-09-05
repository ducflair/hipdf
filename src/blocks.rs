//! Block System for PDF - Reusable Content Components
//!
//! This module provides a block system for registering and reusing PDF content operations.
//! Blocks contain raw lopdf operations that can be instantiated multiple times with
//! different transformations.

use lopdf::{content::{Content, Operation}, Dictionary, Document, Object, ObjectId, Stream, dictionary};
use std::collections::HashMap;
use std::f32::consts::PI;

/// Represents a transformation for block instances
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Scale in X direction
    pub scale_x: f32,
    /// Scale in Y direction
    pub scale_y: f32,
    /// Rotation angle in degrees
    pub rotation: f32,
    /// Translation in X direction
    pub translate_x: f32,
    /// Translation in Y direction
    pub translate_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
        }
    }
}

impl Transform {
    /// Creates a new transform with position only
    pub fn translate(x: f32, y: f32) -> Self {
        Transform {
            translate_x: x,
            translate_y: y,
            ..Default::default()
        }
    }

    /// Creates a new transform with position and uniform scale
    pub fn translate_scale(x: f32, y: f32, scale: f32) -> Self {
        Transform {
            translate_x: x,
            translate_y: y,
            scale_x: scale,
            scale_y: scale,
            ..Default::default()
        }
    }

    /// Creates a new transform with position and non-uniform scale
    pub fn translate_scale_xy(x: f32, y: f32, scale_x: f32, scale_y: f32) -> Self {
        Transform {
            translate_x: x,
            translate_y: y,
            scale_x,
            scale_y,
            ..Default::default()
        }
    }

    /// Creates a new transform with position, scale, and rotation
    pub fn full(x: f32, y: f32, scale_x: f32, scale_y: f32, rotation: f32) -> Self {
        Transform {
            translate_x: x,
            translate_y: y,
            scale_x,
            scale_y,
            rotation,
        }
    }

    /// Converts the transform to a PDF transformation matrix
    pub fn to_matrix(&self) -> [f32; 6] {
        let angle_rad = self.rotation * PI / 180.0;
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // Combine scale and rotation
        let a = self.scale_x * cos_angle;
        let b = self.scale_x * sin_angle;
        let c = -self.scale_y * sin_angle;
        let d = self.scale_y * cos_angle;
        let e = self.translate_x;
        let f = self.translate_y;

        [a, b, c, d, e, f]
    }

    /// Creates a PDF concatenate matrix operation
    pub fn to_operation(&self) -> Operation {
        let matrix = self.to_matrix();
        Operation::new(
            "cm",
            vec![
                matrix[0].into(),
                matrix[1].into(),
                matrix[2].into(),
                matrix[3].into(),
                matrix[4].into(),
                matrix[5].into(),
            ],
        )
    }
}

/// Represents a reusable block of PDF content
#[derive(Debug, Clone)]
pub struct Block {
    /// Unique identifier for this block
    pub id: String,
    /// The PDF operations that make up this block
    pub operations: Vec<Operation>,
    /// Optional bounding box (x, y, width, height) for Form XObject creation
    pub bbox: Option<(f32, f32, f32, f32)>,
    /// Optional resources required by this block
    pub resources: Option<Dictionary>,
}

impl Block {
    /// Creates a new block with the given ID and operations
    pub fn new(id: impl Into<String>, operations: Vec<Operation>) -> Self {
        Block {
            id: id.into(),
            operations,
            bbox: None,
            resources: None,
        }
    }

    /// Sets the bounding box for this block
    pub fn with_bbox(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.bbox = Some((x, y, width, height));
        self
    }

    /// Sets the resources for this block
    pub fn with_resources(mut self, resources: Dictionary) -> Self {
        self.resources = Some(resources);
        self
    }

    /// Adds an operation to the block
    pub fn add_operation(&mut self, op: Operation) {
        self.operations.push(op);
    }

    /// Adds multiple operations to the block
    pub fn add_operations(&mut self, ops: Vec<Operation>) {
        self.operations.extend(ops);
    }
}

/// Represents an instance of a block
#[derive(Debug, Clone)]
pub struct BlockInstance {
    /// The ID of the block to instance
    pub block_id: String,
    /// The transformation to apply
    pub transform: Transform,
}

impl BlockInstance {
    /// Creates a new block instance
    pub fn new(block_id: impl Into<String>, transform: Transform) -> Self {
        BlockInstance {
            block_id: block_id.into(),
            transform,
        }
    }

    /// Creates a new block instance at a specific position
    pub fn at(block_id: impl Into<String>, x: f32, y: f32) -> Self {
        BlockInstance {
            block_id: block_id.into(),
            transform: Transform::translate(x, y),
        }
    }

    /// Creates a new block instance with position and scale
    pub fn at_scaled(block_id: impl Into<String>, x: f32, y: f32, scale: f32) -> Self {
        BlockInstance {
            block_id: block_id.into(),
            transform: Transform::translate_scale(x, y, scale),
        }
    }
}

/// Manager for blocks and their instances
pub struct BlockManager {
    /// Registered blocks
    blocks: HashMap<String, Block>,
    /// Form XObjects created for blocks (for efficient reuse)
    xobjects: HashMap<String, ObjectId>,
    /// Counter for generating unique XObject names
    xobject_counter: usize,
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockManager {
    /// Creates a new block manager
    pub fn new() -> Self {
        BlockManager {
            blocks: HashMap::new(),
            xobjects: HashMap::new(),
            xobject_counter: 0,
        }
    }

    /// Registers a block
    pub fn register(&mut self, block: Block) {
        self.blocks.insert(block.id.clone(), block);
    }

    /// Registers multiple blocks
    pub fn register_blocks(&mut self, blocks: Vec<Block>) {
        for block in blocks {
            self.register(block);
        }
    }

    /// Gets a block by ID
    pub fn get(&self, id: &str) -> Option<&Block> {
        self.blocks.get(id)
    }

    /// Gets a mutable block by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Block> {
        self.blocks.get_mut(id)
    }

    /// Removes a block
    pub fn remove(&mut self, id: &str) -> Option<Block> {
        self.xobjects.remove(id);
        self.blocks.remove(id)
    }

    /// Checks if a block exists
    pub fn has(&self, id: &str) -> bool {
        self.blocks.contains_key(id)
    }

    /// Gets the number of registered blocks
    pub fn count(&self) -> usize {
        self.blocks.len()
    }

    /// Renders a block instance directly as operations
    /// This method includes the block's operations wrapped with transformation
    pub fn render_instance(&self, instance: &BlockInstance) -> Vec<Operation> {
        if let Some(block) = self.blocks.get(&instance.block_id) {
            let mut ops = Vec::new();
            
            // Save graphics state
            ops.push(Operation::new("q", vec![]));
            
            // Apply transformation
            ops.push(instance.transform.to_operation());
            
            // Add block operations
            ops.extend(block.operations.clone());
            
            // Restore graphics state
            ops.push(Operation::new("Q", vec![]));
            
            ops
        } else {
            Vec::new()
        }
    }

    /// Renders multiple block instances
    pub fn render_instances(&self, instances: &[BlockInstance]) -> Vec<Operation> {
        let mut operations = Vec::new();
        for instance in instances {
            operations.extend(self.render_instance(instance));
        }
        operations
    }

    /// Creates Form XObjects for all registered blocks
    /// This allows for more efficient reuse in the PDF
    pub fn create_xobjects(&mut self, doc: &mut Document) {
        for (id, block) in &self.blocks {
            if !self.xobjects.contains_key(id) {
                let xobject_id = self.create_xobject_for_block(doc, block);
                self.xobjects.insert(id.clone(), xobject_id);
            }
        }
    }

    /// Creates a Form XObject for a specific block
    fn create_xobject_for_block(&self, doc: &mut Document, block: &Block) -> ObjectId {
        let mut dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Form",
        };

        // Set bounding box
        if let Some((x, y, w, h)) = block.bbox {
            dict.set("BBox", vec![x.into(), y.into(), (x + w).into(), (y + h).into()]);
        } else {
            dict.set("BBox", vec![0.into(), 0.into(), 100.into(), 100.into()]);
        }

        // Add resources if provided
        if let Some(ref resources) = block.resources {
            dict.set("Resources", resources.clone());
        }

        // Create content from operations
        let content = Content { operations: block.operations.clone() };
        let stream = Stream::new(dict, content.encode().unwrap());
        doc.add_object(stream)
    }

    /// Renders instances using Form XObjects (more efficient for repeated content)
    /// Returns the operations and modifies the resources dictionary to include XObjects
    pub fn render_instances_as_xobjects(
        &mut self,
        instances: &[BlockInstance],
        resources: &mut Dictionary,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();
        let mut xobject_dict = Dictionary::new();

        for instance in instances {
            if let Some(&xobject_id) = self.xobjects.get(&instance.block_id) {
                // Generate unique name for this XObject reference
                let name = format!("Blk{}", self.xobject_counter);
                self.xobject_counter += 1;

                // Add to XObject dictionary
                xobject_dict.set(name.clone(), Object::Reference(xobject_id));

                // Save graphics state
                operations.push(Operation::new("q", vec![]));

                // Apply transformation
                operations.push(instance.transform.to_operation());

                // Draw the XObject
                operations.push(Operation::new(
                    "Do",
                    vec![Object::Name(name.as_bytes().to_vec())],
                ));

                // Restore graphics state
                operations.push(Operation::new("Q", vec![]));
            }
        }

        // Add XObjects to resources if any were used
        if !xobject_dict.is_empty() {
            resources.set("XObject", xobject_dict);
        }

        operations
    }

    /// Clears all registered blocks and XObjects
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.xobjects.clear();
        self.xobject_counter = 0;
    }
}

/// Utility to merge operations from multiple blocks into one
pub fn merge_blocks(blocks: &[&Block]) -> Vec<Operation> {
    let mut operations = Vec::new();
    for block in blocks {
        operations.extend(block.operations.clone());
    }
    operations
}

/// Convenience macro for creating blocks
#[macro_export]
macro_rules! block {
    ($id:expr, $ops:expr) => {
        Block::new($id, $ops)
    };
    ($id:expr, $ops:expr, bbox: ($x:expr, $y:expr, $w:expr, $h:expr)) => {
        Block::new($id, $ops).with_bbox($x, $y, $w, $h)
    };
}

/// Convenience macro for creating instances
#[macro_export]
macro_rules! instance {
    ($block_id:expr, at: ($x:expr, $y:expr)) => {
        BlockInstance::at($block_id, $x, $y)
    };
    ($block_id:expr, at: ($x:expr, $y:expr), scale: $scale:expr) => {
        BlockInstance::at_scaled($block_id, $x, $y, $scale)
    };
    ($block_id:expr, transform: $transform:expr) => {
        BlockInstance::new($block_id, $transform)
    };
}