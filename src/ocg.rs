//! Optional Content Groups (OCG) / Layers support for PDF documents
//! 
//! This module provides a high-level API for working with PDF layers,
//! allowing you to organize content into groups that can be toggled on/off
//! in PDF viewers.

use lopdf::{dictionary, Document, Object, ObjectId, Dictionary, content::Operation};
use std::collections::HashMap;

/// Represents a single Optional Content Group (layer) in a PDF
#[derive(Debug, Clone)]
pub struct Layer {
    /// The internal PDF object ID for this layer
    pub id: ObjectId,
    /// The user-visible name of the layer
    pub name: String,
    /// Whether this layer is visible by default
    pub default_visible: bool,
    /// The resource tag used in content streams (e.g., "L0", "L1")
    pub tag: Option<String>,
}

impl Layer {
    /// Creates a new layer with the given name and visibility
    pub fn new(name: impl Into<String>, default_visible: bool) -> Self {
        Layer {
            id: (0, 0),
            name: name.into(),
            default_visible,
            tag: None,
        }
    }
    
    /// Sets the visibility of this layer
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.default_visible = visible;
        self
    }
}

/// Configuration for the OCG system
#[derive(Debug, Clone)]
pub struct OCGConfig {
    /// Base state for all layers when none are specified ("ON" or "OFF")
    pub base_state: String,
    /// Whether to create a layer panel UI in the PDF viewer
    pub create_panel_ui: bool,
    /// Intent of the layers (e.g., ["View", "Design"])
    pub intent: Vec<String>,
}

impl Default for OCGConfig {
    fn default() -> Self {
        OCGConfig {
            base_state: "ON".to_string(),
            create_panel_ui: true,
            intent: vec!["View".to_string()],
        }
    }
}

/// Main manager for Optional Content Groups in a PDF document
pub struct OCGManager {
    /// All layers in the document
    pub(crate) layers: Vec<Layer>,
    /// Configuration for the OCG system
    pub config: OCGConfig,
    /// The object ID of the OCProperties dictionary
    pub(crate) oc_properties_id: Option<ObjectId>,
    /// Mapping from layer names to their index
    layer_index: HashMap<String, usize>,
}

impl OCGManager {
    /// Creates a new OCGManager with default configuration
    pub fn new() -> Self {
        OCGManager {
            layers: Vec::new(),
            config: OCGConfig::default(),
            oc_properties_id: None,
            layer_index: HashMap::new(),
        }
    }
    
    /// Creates a new OCGManager with custom configuration
    pub fn with_config(config: OCGConfig) -> Self {
        OCGManager {
            layers: Vec::new(),
            config,
            oc_properties_id: None,
            layer_index: HashMap::new(),
        }
    }
    
    /// Adds a new layer to the manager
    /// 
    /// # Arguments
    /// * `layer` - The layer to add
    /// 
    /// # Returns
    /// The index of the added layer
    pub fn add_layer(&mut self, layer: Layer) -> usize {
        let index = self.layers.len();
        self.layer_index.insert(layer.name.clone(), index);
        self.layers.push(layer);
        index
    }
    
    /// Gets a layer by name
    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layer_index.get(name).and_then(|&idx| self.layers.get(idx))
    }
    
    /// Gets a mutable layer by name
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        if let Some(&idx) = self.layer_index.get(name) {
            self.layers.get_mut(idx)
        } else {
            None
        }
    }

    /// Get the number of layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if there are no layers
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Check if OCProperties has been initialized
    pub fn has_oc_properties(&self) -> bool {
        self.oc_properties_id.is_some()
    }
    
    /// Initializes all layers in the PDF document
    /// This should be called after all layers have been added but before they are used
    pub fn initialize(&mut self, doc: &mut Document) {
        // Create OCG objects for each layer
        for layer in &mut self.layers {
            let ocg_dict = dictionary! {
                "Type" => "OCG",
                "Name" => Object::string_literal(layer.name.as_bytes().to_vec()),
            };
            layer.id = doc.add_object(ocg_dict);
        }
        
        // Create the OCProperties dictionary
        self.create_oc_properties(doc);
    }
    
    /// Prepares a page's resources dictionary to use layers
    /// 
    /// # Arguments
    /// * `resources` - The page's resources dictionary
    /// 
    /// # Returns
    /// A map from layer names to their resource tags
    pub fn setup_page_resources(&mut self, resources: &mut Dictionary) -> HashMap<String, String> {
        let mut properties = Dictionary::new();
        let mut layer_map = HashMap::new();
        
        for (i, layer) in self.layers.iter_mut().enumerate() {
            let tag = format!("L{}", i);
            properties.set(tag.clone(), Object::Reference(layer.id));
            layer.tag = Some(tag.clone());
            layer_map.insert(layer.name.clone(), tag);
        }
        
        resources.set("Properties", properties);
        layer_map
    }
    
    /// Updates the document catalog to include OCProperties
    pub fn update_catalog(&self, doc: &mut Document) {
        if let Some(oc_props_id) = self.oc_properties_id {
            // Find the catalog by checking the trailer root
            if let Ok(Object::Reference(catalog_id)) = doc.trailer.get(b"Root") {
                if let Ok(Object::Dictionary(ref mut catalog)) = doc.get_object_mut(*catalog_id) {
                    catalog.set("OCProperties", Object::Reference(oc_props_id));
                }
            }
        }
    }
    
    /// Creates the OCProperties dictionary in the document
    fn create_oc_properties(&mut self, doc: &mut Document) {
        let ocg_refs: Vec<Object> = self.layers.iter()
            .map(|layer| Object::Reference(layer.id))
            .collect();
        
        let on_refs: Vec<Object> = self.layers.iter()
            .filter(|layer| layer.default_visible)
            .map(|layer| Object::Reference(layer.id))
            .collect();
        
        let off_refs: Vec<Object> = self.layers.iter()
            .filter(|layer| !layer.default_visible)
            .map(|layer| Object::Reference(layer.id))
            .collect();
        
        let mut default_dict = dictionary! {
            "Order" => ocg_refs.clone(),
        };
        
        if !self.config.base_state.is_empty() {
            default_dict.set("BaseState", Object::Name(self.config.base_state.as_bytes().to_vec()));
        }
        
        if !on_refs.is_empty() {
            default_dict.set("ON", on_refs);
        }
        
        if !off_refs.is_empty() {
            default_dict.set("OFF", off_refs);
        }
        
        if self.config.create_panel_ui {
            default_dict.set("ListMode", "AllPages");
        }
        
        let mut oc_properties = dictionary! {
            "OCGs" => ocg_refs,
            "D" => default_dict,
        };
        
        if !self.config.intent.is_empty() {
            let intents: Vec<Object> = self.config.intent.iter()
                .map(|s| Object::Name(s.as_bytes().to_vec()))
                .collect();
            oc_properties.set("Intent", intents);
        }
        
        self.oc_properties_id = Some(doc.add_object(oc_properties));
    }
}

/// Builder for creating layered content in a PDF content stream
pub struct LayerContentBuilder {
    operations: Vec<Operation>,
    current_layer: Option<String>,
}

impl LayerContentBuilder {
    /// Creates a new LayerContentBuilder
    pub fn new() -> Self {
        LayerContentBuilder {
            operations: Vec::new(),
            current_layer: None,
        }
    }
    
    /// Begins content for a specific layer
    /// 
    /// # Arguments
    /// * `layer_tag` - The resource tag for the layer (from setup_page_resources)
    pub fn begin_layer(&mut self, layer_tag: &str) -> &mut Self {
        if self.current_layer.is_some() {
            self.end_layer();
        }
        
        self.operations.push(Operation::new("BDC", vec![
            Object::Name(b"OC".to_vec()),
            Object::Name(layer_tag.as_bytes().to_vec())
        ]));
        self.current_layer = Some(layer_tag.to_string());
        self
    }
    
    /// Ends the current layer
    pub fn end_layer(&mut self) -> &mut Self {
        if self.current_layer.is_some() {
            self.operations.push(Operation::new("EMC", vec![]));
            self.current_layer = None;
        }
        self
    }
    
    /// Adds a custom operation
    pub fn add_operation(&mut self, op: Operation) -> &mut Self {
        self.operations.push(op);
        self
    }
    
    /// Adds multiple operations
    pub fn add_operations(&mut self, ops: Vec<Operation>) -> &mut Self {
        self.operations.extend(ops);
        self
    }
    
    /// Builds the final operations list, ensuring all layers are closed
    pub fn build(mut self) -> Vec<Operation> {
        if self.current_layer.is_some() {
            self.end_layer();
        }
        self.operations
    }
}

/// Helper functions for common PDF operations within layers
pub struct LayerOperations;

impl LayerOperations {
    /// Creates a rectangle operation
    pub fn rectangle(x: f32, y: f32, width: f32, height: f32) -> Operation {
        Operation::new("re", vec![x.into(), y.into(), width.into(), height.into()])
    }
    
    /// Creates a fill operation
    pub fn fill() -> Operation {
        Operation::new("f", vec![])
    }
    
    /// Creates a stroke operation
    pub fn stroke() -> Operation {
        Operation::new("S", vec![])
    }
    
    /// Sets RGB fill color
    pub fn set_fill_color_rgb(r: f32, g: f32, b: f32) -> Operation {
        Operation::new("rg", vec![r.into(), g.into(), b.into()])
    }
    
    /// Sets RGB stroke color
    pub fn set_stroke_color_rgb(r: f32, g: f32, b: f32) -> Operation {
        Operation::new("RG", vec![r.into(), g.into(), b.into()])
    }
    
    /// Sets gray fill color
    pub fn set_fill_color_gray(gray: f32) -> Operation {
        Operation::new("g", vec![gray.into()])
    }
    
    /// Begins text
    pub fn begin_text() -> Operation {
        Operation::new("BT", vec![])
    }
    
    /// Ends text
    pub fn end_text() -> Operation {
        Operation::new("ET", vec![])
    }
    
    /// Sets font and size
    pub fn set_font(font_name: &str, size: f32) -> Operation {
        Operation::new("Tf", vec![
            Object::Name(font_name.as_bytes().to_vec()),
            size.into()
        ])
    }
    
    /// Positions text
    pub fn text_position(x: f32, y: f32) -> Operation {
        Operation::new("Td", vec![x.into(), y.into()])
    }
    
    /// Shows text
    pub fn show_text(text: &str) -> Operation {
        Operation::new("Tj", vec![Object::string_literal(text)])
    }
}

/// Convenience macro for creating layers
#[macro_export]
macro_rules! layer {
    ($name:expr) => {
        Layer::new($name, true)
    };
    ($name:expr, visible: $vis:expr) => {
        Layer::new($name, $vis)
    };
}

/// Convenience macro for building layer content
#[macro_export]
macro_rules! layer_content {
    ($builder:expr, in layer $tag:expr => { $($ops:expr),* }) => {
        $builder.begin_layer($tag)
            $(.add_operation($ops))*
            .end_layer()
    };
}
