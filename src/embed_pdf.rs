//! PDF embedding and composition support
//!
//! This module provides functionality to embed other PDF documents within a PDF being created,
//! with support for multi-page documents, various layout strategies, and transformations.

use lopdf::{content::Operation, dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::collections::{BTreeSet, HashMap};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

/// Layout strategy for multi-page embedded PDFs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MultiPageLayout {
    /// Show only the first page
    FirstPageOnly,
    /// Show only a specific page (0-indexed)
    SpecificPage(usize),
    /// Show pages in a vertical stack
    Vertical { gap: f32 },
    /// Show pages in a horizontal line
    Horizontal { gap: f32 },
    /// Show pages in a grid
    Grid {
        columns: usize,
        gap_x: f32,
        gap_y: f32,
        fill_order: GridFillOrder,
    },
    /// Custom layout with specific positions for each page
    Custom(CustomLayoutStrategy),
}

/// Order in which to fill a grid
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridFillOrder {
    RowFirst,    // Left to right, then top to bottom
    ColumnFirst, // Top to bottom, then left to right
}

/// Custom layout strategy for maximum flexibility
#[derive(Debug, Clone, Copy)]
pub struct CustomLayoutStrategy {
    /// Function to calculate position for each page
    pub position_fn: fn(page_index: usize, page_width: f32, page_height: f32) -> (f32, f32),
    /// Function to calculate scale for each page
    pub scale_fn: fn(page_index: usize) -> (f32, f32),
}

impl PartialEq for CustomLayoutStrategy {
    fn eq(&self, other: &Self) -> bool {
        (self.position_fn as usize) == (other.position_fn as usize)
            && (self.scale_fn as usize) == (other.scale_fn as usize)
    }
}

/// Options for embedding a PDF
#[derive(Debug, Clone)]
pub struct EmbedOptions {
    /// Position where to place the embedded content (x, y)
    pub position: (f32, f32),
    /// Scale factor (width_scale, height_scale)
    pub scale: (f32, f32),
    /// Rotation angle in degrees
    pub rotation: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Layout strategy for multi-page PDFs
    pub layout: MultiPageLayout,
    /// Maximum width constraint (None for no constraint)
    pub max_width: Option<f32>,
    /// Maximum height constraint (None for no constraint)
    pub max_height: Option<f32>,
    /// Whether to preserve aspect ratio when scaling
    pub preserve_aspect_ratio: bool,
    /// Clip to bounding box
    pub clip_bounds: Option<(f32, f32, f32, f32)>, // (x, y, width, height)
    /// Page range to include (None means all pages)
    pub page_range: Option<PageRange>,
}

/// Page range specification
#[derive(Debug, Clone, PartialEq)]
pub enum PageRange {
    /// Single page
    Single(usize),
    /// Range of pages (inclusive)
    Range(usize, usize),
    /// Specific pages
    Pages(Vec<usize>),
    /// All pages
    All,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        EmbedOptions {
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
            layout: MultiPageLayout::FirstPageOnly,
            max_width: None,
            max_height: None,
            preserve_aspect_ratio: true,
            clip_bounds: None,
            page_range: None,
        }
    }
}

impl EmbedOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = (scale, scale);
        self
    }

    pub fn with_scale_xy(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.scale = (scale_x, scale_y);
        self
    }

    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_layout(mut self, layout: MultiPageLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_max_size(mut self, width: f32, height: f32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    pub fn with_clip_bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.clip_bounds = Some((x, y, width, height));
        self
    }

    pub fn with_page_range(mut self, range: PageRange) -> Self {
        self.page_range = Some(range);
        self
    }

    pub fn preserve_aspect_ratio(mut self, preserve: bool) -> Self {
        self.preserve_aspect_ratio = preserve;
        self
    }
}

/// Information about an embedded PDF
#[derive(Debug, Clone)]
pub struct EmbeddedPdfInfo {
    /// Number of pages in the source PDF
    pub page_count: usize,
    /// Dimensions of each page (width, height)
    pub page_dimensions: Vec<(f32, f32)>,
    /// The embedded PDF's metadata
    pub metadata: HashMap<String, String>,
}

/// Result of an embed operation containing operations and resources
#[derive(Debug, Clone)]
pub struct EmbedResult {
    /// The content operations to add to the page
    pub operations: Vec<Operation>,
    /// The XObject resources to add to the page's Resources dictionary
    pub xobject_resources: HashMap<String, Object>,
}

/// Manager for embedding PDFs into documents
pub struct PdfEmbedder {
    /// Cache of loaded PDF documents
    loaded_pdfs: HashMap<String, (Document, EmbeddedPdfInfo)>,
    /// Counter for generating unique resource names
    resource_counter: usize,
    /// Cache mapping (source_identifier, source_ObjectId) → target ObjectId so
    /// shared resources are copied only once. Scoped per source document to
    /// prevent cross-contamination when multiple PDFs are embedded.
    copied_objects: HashMap<(String, ObjectId), ObjectId>,
}

impl Default for PdfEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfEmbedder {
    pub fn new() -> Self {
        PdfEmbedder {
            loaded_pdfs: HashMap::new(),
            resource_counter: 0,
            copied_objects: HashMap::new(),
        }
    }

    /// Load a PDF from file, sanitizing it for safe embedding.
    pub fn load_pdf(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        if self.loaded_pdfs.contains_key(&path_str) {
            return Ok(path_str);
        }

        let mut source_doc = Document::load(path.as_ref()).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Failed to load PDF: {}", e))
        })?;

        // Sanitize the document before caching and using it.
        self.sanitize_pdf(&mut source_doc)?;

        let info = self.extract_pdf_info(&source_doc)?;

        self.loaded_pdfs
            .insert(path_str.clone(), (source_doc, info));
        Ok(path_str)
    }

    /// Load a PDF from bytes, sanitizing it for safe embedding.
    pub fn load_pdf_from_bytes(&mut self, bytes: &[u8], identifier: &str) -> Result<String> {
        if self.loaded_pdfs.contains_key(identifier) {
            return Ok(identifier.to_string());
        }

        // lopdf internally uses weezl for LZW-compressed content streams; malformed data can
        // trigger an assertion panic instead of returning an error, so we catch it here.
        let load_result = std::panic::catch_unwind(|| Document::load_from(bytes));
        let mut source_doc = match load_result {
            Ok(Ok(doc)) => doc,
            Ok(Err(e)) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to load PDF from bytes: {}", e),
                ))
            }
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "PDF loading failed: malformed LZW/compressed stream in PDF",
                ))
            }
        };

        // Sanitize the document before caching and using it.
        self.sanitize_pdf(&mut source_doc)?;

        let info = self.extract_pdf_info(&source_doc)?;

        self.loaded_pdfs
            .insert(identifier.to_string(), (source_doc, info));
        Ok(identifier.to_string())
    }

    /// Get information about a loaded PDF
    pub fn get_pdf_info(&self, identifier: &str) -> Option<&EmbeddedPdfInfo> {
        self.loaded_pdfs.get(identifier).map(|(_, info)| info)
    }

    /// Embed a PDF into the target document
    /// Returns both the operations and the XObject resources that need to be added to the page
    pub fn embed_pdf(
        &mut self,
        target_doc: &mut Document,
        source_identifier: &str,
        options: &EmbedOptions,
    ) -> Result<EmbedResult> {
        // Get source document
        let (source_doc, info) = self
            .loaded_pdfs
            .get(source_identifier)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "PDF not loaded"))?;

        // Clone necessary data to avoid borrowing issues
        let info = info.clone();
        let source_doc = source_doc.clone();
        let source_id_key = source_identifier.to_string();

        // Determine which pages to include
        let pages_to_include = self.determine_pages(options, info.page_count);

        // Calculate layout positions for each page
        let page_positions = self.calculate_page_positions(&pages_to_include, &info, options);

        // Generate operations for embedding
        let mut all_operations = Vec::new();
        let mut xobject_resources = HashMap::new();

        // Apply clipping if specified
        if let Some((clip_x, clip_y, clip_w, clip_h)) = options.clip_bounds {
            all_operations.push(Operation::new("q", vec![])); // Save graphics state
            all_operations.push(Operation::new(
                "re",
                vec![clip_x.into(), clip_y.into(), clip_w.into(), clip_h.into()],
            ));
            all_operations.push(Operation::new("W", vec![])); // Clip
            all_operations.push(Operation::new("n", vec![])); // End path without painting
        }

        // Import and embed each page as a Form XObject
        for (page_idx, x, y, scale_x, scale_y) in page_positions {
            self.resource_counter += 1;
            let xobject_name = format!("XO{}", self.resource_counter);

            // Import the page as a Form XObject
            let xobject_ref = self.import_page_as_xobject(target_doc, &source_doc, page_idx, &source_id_key)?;

            // Add to resources map
            xobject_resources.insert(xobject_name.clone(), xobject_ref.clone());

            // Generate operations to place the XObject
            let page_ops = self.place_xobject(
                &xobject_name,
                x,
                y,
                scale_x,
                scale_y,
                options.rotation,
                options.opacity,
            );
            all_operations.extend(page_ops);
        }

        // Restore graphics state if clipping was applied
        if options.clip_bounds.is_some() {
            all_operations.push(Operation::new("Q", vec![]));
        }

        Ok(EmbedResult {
            operations: all_operations,
            xobject_resources,
        })
    }

    // Function to sanitize a PDF document by removing dangerous actions.
    /// Traverses the document and neutralizes dangerous actions like JavaScript or Launch.
    fn sanitize_pdf(&self, doc: &mut Document) -> Result<()> {
        let mut dangerous_action_ids = BTreeSet::new();

        // Pass 1: Identify all dangerous action objects.
        // We check all objects, but also specifically look in common places like
        // the document catalog (/OpenAction) and page annotations (/Annots).

        // Check the document catalog for an /OpenAction
        if let Ok(catalog) = doc.catalog() {
            if let Ok(Object::Reference(id)) = catalog.get(b"OpenAction") {
                if self.is_action_dangerous(doc, *id)? {
                    dangerous_action_ids.insert(*id);
                }
            }
        }

        // Check all page objects for annotations with dangerous actions
        for (_page_num, page_id) in doc.get_pages() {
            if let Ok(page_dict) = doc.get_object(page_id).and_then(|o| o.as_dict()) {
                if let Ok(annots) = page_dict.get(b"Annots") {
                    let annots_obj = match annots {
                        Object::Reference(ref_id) => {
                            doc.get_object(*ref_id)
                        }
                        _ => Ok(annots),
                    };
                    if let Ok(Object::Array(annots_arr)) = annots_obj {
                        for annot_ref in annots_arr {
                            if let Object::Reference(id) = annot_ref {
                                if let Ok(annot_dict) = doc.get_object(*id).and_then(|o| o.as_dict()) {
                                    if let Ok(action_ref) = annot_dict.get(b"A") {
                                        if let Object::Reference(action_id) = action_ref {
                                            if self.is_action_dangerous(doc, *action_id)? {
                                                dangerous_action_ids.insert(*action_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pass 2: Neuter the identified dangerous actions.
        // We replace the action dictionary with an empty one, which is harmless
        // but preserves the object reference, preventing the document from breaking.
        for id in dangerous_action_ids {
            doc.objects.insert(id, Object::Dictionary(Dictionary::new()));
        }

        Ok(())
    }

    // Helper function to check if an action object is dangerous.
    /// Checks if the object at a given ID is a dangerous action dictionary.
    fn is_action_dangerous(&self, doc: &Document, action_id: ObjectId) -> Result<bool> {
        if let Ok(action_obj) = doc.get_object(action_id) {
            if let Ok(dict) = action_obj.as_dict() {
                if let Ok(Object::Name(action_type)) = dict.get(b"S") {
                    let action_type_str = String::from_utf8_lossy(action_type);
                    // Add other action types to this list if needed (e.g., "URI", "GoToR")
                    if action_type_str == "JavaScript" || action_type_str == "Launch" {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Import a page from source document as a Form XObject
    fn import_page_as_xobject(
        &mut self,
        target_doc: &mut Document,
        source_doc: &Document,
        page_index: usize,
        source_id: &str,
    ) -> Result<Object> {
        // Get the page from source document
        let pages = source_doc.get_pages();
        let page_id = pages
            .get(&(page_index as u32 + 1))
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Page not found in source PDF"))?;

        let page_obj = source_doc.get_object(*page_id).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to get page object: {}", e),
            )
        })?;

        let page_dict = page_obj.as_dict().map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Page object is not a dictionary: {}", e),
            )
        })?;

        // Get page dimensions (raw MediaBox)
        let (raw_w, raw_h) = self.get_raw_media_box_dims(page_dict, source_doc);

        // Read the /UserUnit attribute (PDF 1.6+, may be inherited).
        let user_unit = self.resolve_page_user_unit(page_dict, source_doc);

        // Read the /Rotate attribute (may be inherited from parent /Pages nodes).
        // Form XObjects don't support /Rotate or /UserUnit, so we must bake both into
        // the BBox and prepend a correcting `cm` transform to the content stream.
        let rotate = self.resolve_page_rotate(page_dict, source_doc);

        let phys_w = raw_w * user_unit;
        let phys_h = raw_h * user_unit;

        // After rotation and user unit scaling, compute visual (BBox) dimensions and CTM.
        // /Rotate specifies CW degrees the viewer applies on display.
        let (bbox_w, bbox_h, transform_cm) = match rotate {
            90 => (
                phys_h,
                phys_w,
                // 90° CW: x' = user_unit * y,  y' = phys_w - user_unit * x  →  [0 -user_unit user_unit 0 0 phys_w]
                Some(format!("0 -{} {} 0 0 {} cm\n", user_unit, user_unit, phys_w)),
            ),
            180 => (
                phys_w,
                phys_h,
                // 180°: x' = phys_w - user_unit * x,  y' = phys_h - user_unit * y  →  [-user_unit 0 0 -user_unit phys_w phys_h]
                Some(format!("-{} 0 0 -{} {} {} cm\n", user_unit, user_unit, phys_w, phys_h)),
            ),
            270 => (
                phys_h,
                phys_w,
                // 270° CW (= 90° CCW): x' = phys_h - user_unit * y,  y' = user_unit * x  →  [0 user_unit -user_unit 0 phys_h 0]
                Some(format!("0 {} -{} 0 {} 0 cm\n", user_unit, user_unit, phys_h)),
            ),
            _ => {
                if (user_unit - 1.0).abs() > f32::EPSILON {
                    (
                        phys_w,
                        phys_h,
                        Some(format!("{} 0 0 {} 0 0 cm\n", user_unit, user_unit)),
                    )
                } else {
                    (phys_w, phys_h, None)
                }
            }
        };

        let bbox = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(bbox_w),
            Object::Real(bbox_h),
        ]);

        // Build the Form XObject content stream.
        // Always decompress and re-compress to ensure consistency — lopdf may
        // transparently decode stream.content during loading while leaving the
        // Filter key in the dict, so trusting raw bytes + dict is unreliable.
        let raw_content = self.get_page_content_stream(source_doc, page_dict)?;
        let final_content = if let Some(cm_str) = transform_cm {
            let mut buf = cm_str.into_bytes();
            buf.extend_from_slice(&raw_content);
            buf
        } else {
            raw_content
        };
        let content_bytes = miniz_oxide::deflate::compress_to_vec_zlib(&final_content, 6);

        // Get page resources
        let resources = if let Ok(res_obj) = page_dict.get(b"Resources") {
            self.copy_object_to_target(source_doc, target_doc, res_obj, source_id)?
        } else {
            Object::Dictionary(Dictionary::new())
        };

        // Create Form XObject dictionary
        let xobject_dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Form",
            "BBox" => bbox,
            "Resources" => resources,
            "Matrix" => vec![1.into(), 0.into(), 0.into(), 1.into(), 0.into(), 0.into()],
            "Filter" => "FlateDecode",
        };

        // Create the Form XObject stream
        let xobject_stream = Stream::new(xobject_dict, content_bytes);
        let xobject_id = target_doc.add_object(xobject_stream);

        Ok(Object::Reference(xobject_id))
    }

    /// Get the content stream of a page
    fn get_page_content_stream(
        &self,
        source_doc: &Document,
        page_dict: &Dictionary,
    ) -> Result<Vec<u8>> {
        if let Ok(contents_obj) = page_dict.get(b"Contents") {
            self.extract_content_bytes(source_doc, contents_obj)
        } else {
            Ok(Vec::new())
        }
    }

    /// Extract content bytes from a content object
    fn extract_content_bytes(&self, doc: &Document, contents: &Object) -> Result<Vec<u8>> {
        match contents {
            Object::Reference(ref_id) => {
                if let Ok(content_obj) = doc.get_object(*ref_id) {
                    self.extract_content_bytes(doc, content_obj)
                } else {
                    Ok(Vec::new())
                }
            }
            Object::Stream(stream) => {
                // Decode the stream content — weezl (LZW decoder used by lopdf) can panic on
                // malformed streams instead of returning an error, so we catch_unwind it.
                let result = std::panic::catch_unwind(|| stream.decompressed_content());
                match result {
                    Ok(Ok(bytes)) => Ok(bytes),
                    Ok(Err(_)) => Ok(stream.content.clone()),
                    Err(_) => Ok(stream.content.clone()),
                }
            }
            Object::Array(array) => {
                let mut all_content = Vec::new();
                for item in array {
                    let content = self.extract_content_bytes(doc, item)?;
                    all_content.extend(content);
                    all_content.push(b'\n'); // Add newline between content streams
                }
                Ok(all_content)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Copy an object from source to target document, deduplicating shared objects.
    /// `source_id` scopes the dedup cache to the source document so different
    /// PDFs with colliding ObjectIds never cross-contaminate.
    fn copy_object_to_target(
        &mut self,
        source_doc: &Document,
        target_doc: &mut Document,
        obj: &Object,
        source_id: &str,
    ) -> Result<Object> {
        match obj {
            Object::Reference(ref_id) => {
                let cache_key = (source_id.to_string(), *ref_id);
                // Check if we already copied this object
                if let Some(&target_id) = self.copied_objects.get(&cache_key) {
                    return Ok(Object::Reference(target_id));
                }
                // Dereference and copy the actual object
                if let Ok(actual_obj) = source_doc.get_object(*ref_id) {
                    let copied = self.copy_object_to_target(source_doc, target_doc, actual_obj, source_id)?;
                    // If the copied result is a reference (stream), cache it directly.
                    // Otherwise wrap in a new object and cache.
                    match &copied {
                        Object::Reference(new_id) => {
                            self.copied_objects.insert(cache_key, *new_id);
                        }
                        _ => {
                            let new_id = target_doc.add_object(copied.clone());
                            self.copied_objects.insert(cache_key, new_id);
                            return Ok(Object::Reference(new_id));
                        }
                    }
                    Ok(copied)
                } else {
                    Ok(Object::Null)
                }
            }
            Object::Dictionary(dict) => {
                let mut new_dict = Dictionary::new();
                for (key, value) in dict.iter() {
                    let new_value = self.copy_object_to_target(source_doc, target_doc, value, source_id)?;
                    new_dict.set(key.clone(), new_value);
                }
                Ok(Object::Dictionary(new_dict))
            }
            Object::Array(array) => {
                let mut new_array = Vec::new();
                for item in array {
                    let new_item = self.copy_object_to_target(source_doc, target_doc, item, source_id)?;
                    new_array.push(new_item);
                }
                Ok(Object::Array(new_array))
            }
            Object::Stream(stream) => {
                let new_dict = if let Object::Dictionary(dict) = self.copy_object_to_target(
                    source_doc,
                    target_doc,
                    &Object::Dictionary(stream.dict.clone()),
                    source_id,
                )? {
                    dict
                } else {
                    Dictionary::new()
                };
                let new_stream = Stream::new(new_dict, stream.content.clone());
                let stream_id = target_doc.add_object(new_stream);
                Ok(Object::Reference(stream_id))
            }
            // For simple types, just clone
            _ => Ok(obj.clone()),
        }
    }

    /// Get the raw MediaBox dimensions (width, height) without applying /Rotate or /UserUnit.
    fn get_raw_media_box_dims(&self, page_dict: &Dictionary, source_doc: &Document) -> (f32, f32) {
        let extract_box = |dict: &Dictionary| -> Option<(f32, f32)> {
            let obj = dict
                .get(b"MediaBox")
                .ok()
                .and_then(|o| match o {
                    Object::Reference(id) => source_doc.get_object(*id).ok().cloned(),
                    other => Some(other.clone()),
                });
            if let Some(Object::Array(coords)) = obj {
                if coords.len() >= 4 {
                    let to_f32 = |o: &Object| match o {
                        Object::Real(v) => *v,
                        Object::Integer(v) => *v as f32,
                        _ => 0.0,
                    };
                    let x1 = to_f32(&coords[0]);
                    let y1 = to_f32(&coords[1]);
                    let x2 = to_f32(&coords[2]);
                    let y2 = to_f32(&coords[3]);
                    return Some(((x2 - x1).abs(), (y2 - y1).abs()));
                }
            }
            None
        };

        if let Some(dims) = extract_box(page_dict) {
            return dims;
        }

        // Walk up the page tree via /Parent references for inherited MediaBox
        let mut current = page_dict.get(b"Parent").ok().cloned();
        while let Some(obj) = current {
            let parent_obj = match &obj {
                Object::Reference(id) => source_doc.get_object(*id).ok(),
                _ => None,
            };
            if let Some(parent) = parent_obj {
                if let Ok(dict) = parent.as_dict() {
                    if let Some(dims) = extract_box(dict) {
                        return dims;
                    }
                    current = dict.get(b"Parent").ok().cloned();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (595.0, 842.0)
    }

    /// Generate operations to place an XObject
    fn place_xobject(
        &self,
        xobject_name: &str,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
        rotation: f32,
        opacity: f32,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();

        // Save graphics state
        operations.push(Operation::new("q", vec![]));

        // Apply transformations
        let angle_rad = rotation * std::f32::consts::PI / 180.0;
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        // Combined transformation matrix: scale, rotate, and translate
        operations.push(Operation::new(
            "cm",
            vec![
                (scale_x * cos).into(),
                (scale_x * sin).into(),
                (-scale_y * sin).into(),
                (scale_y * cos).into(),
                x.into(),
                y.into(),
            ],
        ));

        // Apply opacity if needed (simplified - in production you'd need to properly handle ExtGState)
        if opacity < 1.0 {
            // This is simplified - proper implementation would require adding to Resources/ExtGState
            // For now, we'll skip opacity handling in the actual rendering
        }

        // Draw the XObject
        operations.push(Operation::new(
            "Do",
            vec![Object::Name(xobject_name.as_bytes().to_vec())],
        ));

        // Restore graphics state
        operations.push(Operation::new("Q", vec![]));

        operations
    }

    /// Extract information from a PDF document
    fn extract_pdf_info(&self, doc: &Document) -> Result<EmbeddedPdfInfo> {
        let pages = doc.get_pages();
        let page_count = pages.len();
        let mut page_dimensions = Vec::new();

        for (_page_num, page_id) in pages.iter() {
            if let Ok(page_obj) = doc.get_object(*page_id) {
                if let Ok(page_dict) = page_obj.as_dict() {
                    let dimensions = self.get_page_dimensions(page_dict, doc);
                    page_dimensions.push(dimensions);
                }
            }
        }

        // Extract metadata
        let mut metadata = HashMap::new();
        if let Ok(info_obj) = doc.trailer.get(b"Info") {
            if let Object::Reference(info_ref) = info_obj {
                if let Ok(info_obj) = doc.get_object(*info_ref) {
                    if let Ok(info_dict) = info_obj.as_dict() {
                        // Extract common metadata fields
                        for (key, value) in info_dict.iter() {
                            if let Ok(string_val) = value.as_str() {
                                metadata.insert(
                                    String::from_utf8_lossy(key).to_string(),
                                    String::from_utf8_lossy(string_val).to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(EmbeddedPdfInfo {
            page_count,
            page_dimensions,
            metadata,
        })
    }

    /// Get dimensions of a page from its dictionary.
    /// Returns the VISUAL (post-/Rotate and post-/UserUnit) dimensions so layout calculations use correct physical sizes.
    fn get_page_dimensions(&self, page_dict: &Dictionary, source_doc: &Document) -> (f32, f32) {
        let (raw_w, raw_h) = self.get_raw_media_box_dims(page_dict, source_doc);
        let user_unit = self.resolve_page_user_unit(page_dict, source_doc);
        let (phys_w, phys_h) = (raw_w * user_unit, raw_h * user_unit);
        let rotate = self.resolve_page_rotate(page_dict, source_doc);
        match rotate {
            90 | 270 => (phys_h, phys_w),
            _ => (phys_w, phys_h),
        }
    }

    /// Resolve the effective /Rotate value for a page, walking up the page tree
    /// to find inherited values. Returns 0, 90, 180, or 270.
    fn resolve_page_rotate(&self, page_dict: &Dictionary, doc: &Document) -> i64 {
        let extract_rotate = |dict: &Dictionary| -> Option<i64> {
            let obj = dict.get(b"Rotate").ok().and_then(|o| match o {
                Object::Reference(id) => doc.get_object(*id).ok().cloned(),
                other => Some(other.clone()),
            });
            match obj {
                Some(Object::Integer(n)) => Some(n),
                _ => None,
            }
        };

        if let Some(n) = extract_rotate(page_dict) {
            return n.rem_euclid(360);
        }

        // Walk up the page tree via /Parent references
        let mut current = page_dict.get(b"Parent").ok().cloned();
        while let Some(obj) = current {
            let parent_obj = match &obj {
                Object::Reference(id) => doc.get_object(*id).ok(),
                _ => None,
            };
            if let Some(parent) = parent_obj {
                if let Ok(dict) = parent.as_dict() {
                    if let Some(n) = extract_rotate(dict) {
                        return n.rem_euclid(360);
                    }
                    current = dict.get(b"Parent").ok().cloned();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        0
    }

    /// Resolve the effective /UserUnit value for a page, walking up the page tree
    /// to find inherited values (PDF 1.6+, §7.7.3.3). Returns 1.0 by default.
    fn resolve_page_user_unit(&self, page_dict: &Dictionary, doc: &Document) -> f32 {
        let extract_user_unit = |dict: &Dictionary| -> Option<f32> {
            let obj = dict.get(b"UserUnit").ok().and_then(|o| match o {
                Object::Reference(id) => doc.get_object(*id).ok().cloned(),
                other => Some(other.clone()),
            });
            match obj {
                Some(Object::Real(v)) => Some(v as f32),
                Some(Object::Integer(v)) => Some(v as f32),
                _ => None,
            }
        };

        if let Some(uu) = extract_user_unit(page_dict) {
            if uu > 0.0 {
                return uu;
            }
        }

        // Walk up the page tree via /Parent references
        let mut current = page_dict.get(b"Parent").ok().cloned();
        while let Some(obj) = current {
            let parent_obj = match &obj {
                Object::Reference(id) => doc.get_object(*id).ok(),
                _ => None,
            };
            if let Some(parent) = parent_obj {
                if let Ok(dict) = parent.as_dict() {
                    if let Some(uu) = extract_user_unit(dict) {
                        if uu > 0.0 {
                            return uu;
                        }
                    }
                    current = dict.get(b"Parent").ok().cloned();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        1.0
    }

    /// Determine which pages to include based on options
    fn determine_pages(&self, options: &EmbedOptions, total_pages: usize) -> Vec<usize> {
        let range = options.page_range.as_ref().unwrap_or(&PageRange::All);

        let mut pages = match range {
            PageRange::Single(page) => vec![*page],
            PageRange::Range(start, end) => (*start..=*end.min(&(total_pages - 1))).collect(),
            PageRange::Pages(specific) => specific.clone(),
            PageRange::All => (0..total_pages).collect(),
        };

        // Apply layout-specific filtering
        match options.layout {
            MultiPageLayout::FirstPageOnly => {
                if !pages.is_empty() {
                    pages = vec![pages[0]];
                }
            }
            MultiPageLayout::SpecificPage(page) => {
                if page < total_pages {
                    pages = vec![page];
                }
            }
            _ => {} // Other layouts use all specified pages
        }

        pages
    }

    /// Calculate positions for each page based on layout strategy
    fn calculate_page_positions(
        &self,
        pages: &[usize],
        info: &EmbeddedPdfInfo,
        options: &EmbedOptions,
    ) -> Vec<(usize, f32, f32, f32, f32)> {
        let mut positions = Vec::new();
        let base_x = options.position.0;
        let base_y = options.position.1;

        for (idx, &page_num) in pages.iter().enumerate() {
            let (page_w, page_h) = info.page_dimensions[page_num];
            let (scale_x, scale_y) = self.calculate_scale(page_w, page_h, options);
            let scaled_w = page_w * scale_x;
            let scaled_h = page_h * scale_y;

            let (x, y) = match options.layout {
                MultiPageLayout::FirstPageOnly | MultiPageLayout::SpecificPage(_) => {
                    (base_x, base_y)
                }
                MultiPageLayout::Vertical { gap } => {
                    let total_height: f32 = (0..idx)
                        .map(|i| {
                            let (_, h) = info.page_dimensions[pages[i]];
                            h * scale_y + gap
                        })
                        .sum();
                    (base_x, base_y - total_height)
                }
                MultiPageLayout::Horizontal { gap } => {
                    let total_width: f32 = (0..idx)
                        .map(|i| {
                            let (w, _) = info.page_dimensions[pages[i]];
                            w * scale_x + gap
                        })
                        .sum();
                    (base_x + total_width, base_y)
                }
                MultiPageLayout::Grid {
                    columns,
                    gap_x,
                    gap_y,
                    fill_order,
                } => {
                    let (row, col) = match fill_order {
                        GridFillOrder::RowFirst => (idx / columns, idx % columns),
                        GridFillOrder::ColumnFirst => (idx % columns, idx / columns),
                    };
                    (
                        base_x + col as f32 * (scaled_w + gap_x),
                        base_y - row as f32 * (scaled_h + gap_y),
                    )
                }
                MultiPageLayout::Custom(strategy) => {
                    let (x_offset, y_offset) = (strategy.position_fn)(idx, page_w, page_h);
                    (base_x + x_offset, base_y + y_offset)
                }
            };

            positions.push((page_num, x, y, scale_x, scale_y));
        }

        positions
    }

    /// Calculate scale factors considering constraints
    fn calculate_scale(&self, width: f32, height: f32, options: &EmbedOptions) -> (f32, f32) {
        let mut scale_x = options.scale.0;
        let mut scale_y = options.scale.1;

        // Apply max size constraints
        if let Some(max_w) = options.max_width {
            let required_scale_x = max_w / width;
            scale_x = scale_x.min(required_scale_x);
            if options.preserve_aspect_ratio {
                scale_y = scale_x;
            }
        }

        if let Some(max_h) = options.max_height {
            let required_scale_y = max_h / height;
            scale_y = scale_y.min(required_scale_y);
            if options.preserve_aspect_ratio {
                scale_x = scale_y;
            }
        }

        (scale_x, scale_y)
    }
}

/// Builder for creating complex embedded PDF layouts
pub struct EmbedLayoutBuilder {
    embedder: PdfEmbedder,
    operations: Vec<Operation>,
    xobject_resources: HashMap<String, Object>,
}

impl Default for EmbedLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbedLayoutBuilder {
    pub fn new() -> Self {
        EmbedLayoutBuilder {
            embedder: PdfEmbedder::new(),
            operations: Vec::new(),
            xobject_resources: HashMap::new(),
        }
    }

    /// Load a PDF for embedding
    pub fn load_pdf(&mut self, path: impl AsRef<Path>) -> Result<String> {
        self.embedder.load_pdf(path)
    }

    /// Add an embedded PDF with options
    pub fn add_embedded_pdf(
        &mut self,
        target_doc: &mut Document,
        source_id: &str,
        options: EmbedOptions,
    ) -> Result<&mut Self> {
        let result = self.embedder.embed_pdf(target_doc, source_id, &options)?;
        self.operations.extend(result.operations);
        self.xobject_resources.extend(result.xobject_resources);
        Ok(self)
    }

    /// Create a thumbnail gallery of PDF pages
    pub fn create_thumbnail_gallery(
        &mut self,
        target_doc: &mut Document,
        source_id: &str,
        x: f32,
        y: f32,
        thumb_size: f32,
        columns: usize,
        gap: f32,
    ) -> Result<&mut Self> {
        let options = EmbedOptions::new()
            .at_position(x, y)
            .with_max_size(thumb_size, thumb_size)
            .with_layout(MultiPageLayout::Grid {
                columns,
                gap_x: gap,
                gap_y: gap,
                fill_order: GridFillOrder::RowFirst,
            });

        self.add_embedded_pdf(target_doc, source_id, options)
    }

    /// Create a side-by-side comparison of two PDFs
    pub fn create_comparison(
        &mut self,
        target_doc: &mut Document,
        left_pdf: &str,
        right_pdf: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        gap: f32,
    ) -> Result<&mut Self> {
        let half_width = (width - gap) / 2.0;

        // Left PDF
        let left_options = EmbedOptions::new()
            .at_position(x, y)
            .with_max_size(half_width, height)
            .with_layout(MultiPageLayout::FirstPageOnly);

        self.add_embedded_pdf(target_doc, left_pdf, left_options)?;

        // Right PDF
        let right_options = EmbedOptions::new()
            .at_position(x + half_width + gap, y)
            .with_max_size(half_width, height)
            .with_layout(MultiPageLayout::FirstPageOnly);

        self.add_embedded_pdf(target_doc, right_pdf, right_options)?;

        Ok(self)
    }

    /// Build and return the result
    pub fn build(self) -> EmbedResult {
        EmbedResult {
            operations: self.operations,
            xobject_resources: self.xobject_resources,
        }
    }

    /// Get the embedder for advanced operations
    pub fn embedder(&mut self) -> &mut PdfEmbedder {
        &mut self.embedder
    }
}

/// Utility functions for common embedding patterns
pub struct EmbedUtils;

impl EmbedUtils {
    /// Create options for a watermark-style embed
    pub fn watermark_options(opacity: f32, scale: f32) -> EmbedOptions {
        EmbedOptions::new()
            .with_opacity(opacity)
            .with_scale(scale)
            .at_position(100.0, 100.0)
            .with_rotation(45.0)
    }

    /// Create options for a thumbnail
    pub fn thumbnail_options(x: f32, y: f32, size: f32) -> EmbedOptions {
        EmbedOptions::new()
            .at_position(x, y)
            .with_max_size(size, size)
            .preserve_aspect_ratio(true)
    }

    /// Create options for full-page embed
    pub fn full_page_options(page_width: f32, page_height: f32) -> EmbedOptions {
        EmbedOptions::new()
            .at_position(0.0, 0.0)
            .with_max_size(page_width, page_height)
            .preserve_aspect_ratio(true)
    }
}
