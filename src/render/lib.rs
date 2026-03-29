use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + self.width
            && point.1 >= self.y
            && point.1 <= self.y + self.height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoxModel {
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub border_top: f32,
    pub border_right: f32,
    pub border_bottom: f32,
    pub border_left: f32,
}

impl BoxModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin_width(&self) -> f32 {
        self.margin_left + self.margin_right
    }

    pub fn margin_height(&self) -> f32 {
        self.margin_top + self.margin_bottom
    }

    pub fn padding_width(&self) -> f32 {
        self.padding_left + self.padding_right
    }

    pub fn padding_height(&self) -> f32 {
        self.padding_top + self.padding_bottom
    }

    pub fn border_width(&self) -> f32 {
        self.border_left + self.border_right
    }

    pub fn border_height(&self) -> f32 {
        self.border_top + self.border_bottom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Element,
    Text,
    Comment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMNode {
    pub id: String,
    pub node_type: NodeType,
    pub tag_name: String,
    pub text_content: String,
    pub children: Vec<DOMNode>,
    pub bounds: Rect,
    pub box_model: BoxModel,
}

impl DOMNode {
    pub fn new(id: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            tag_name: String::new(),
            text_content: String::new(),
            children: Vec::new(),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            box_model: BoxModel::new(),
        }
    }

    pub fn with_tag_name(mut self, tag_name: impl Into<String>) -> Self {
        self.tag_name = tag_name.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_content = text.into();
        self
    }

    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn with_box_model(mut self, box_model: BoxModel) -> Self {
        self.box_model = box_model;
        self
    }

    pub fn add_child(mut self, child: DOMNode) -> Self {
        self.children.push(child);
        self
    }
}

pub struct LayoutEngine {
    dom: Vec<DOMNode>,
    element_map: HashMap<String, usize>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            dom: Vec::new(),
            element_map: HashMap::new(),
        }
    }

    pub fn parse_html(&mut self, _html: &str) {
        self.dom.clear();
        self.element_map.clear();

        let root = DOMNode::new("root", NodeType::Element)
            .with_tag_name("html")
            .with_bounds(Rect::new(0.0, 0.0, 1920.0, 1080.0));

        self.dom.push(root);
        self.element_map.insert("root".to_string(), 0);
    }

    pub fn calculate_layout(&mut self) {
        for node in &mut self.dom {
            let parent_bounds = if node.id == "root" {
                node.bounds.clone()
            } else {
                Rect::new(0.0, 0.0, 0.0, 0.0)
            };
            node.bounds = parent_bounds;
        }
    }

    pub fn get_element_bounds(&self, element_id: &str) -> Option<Rect> {
        self.element_map
            .get(element_id)
            .and_then(|&idx| self.dom.get(idx))
            .map(|node| node.bounds.clone())
    }

    pub fn get_element_box_model(&self, element_id: &str) -> Option<BoxModel> {
        self.element_map
            .get(element_id)
            .and_then(|&idx| self.dom.get(idx))
            .map(|node| node.box_model.clone())
    }

    pub fn get_element_by_id(&self, element_id: &str) -> Option<&DOMNode> {
        self.element_map
            .get(element_id)
            .and_then(|&idx| self.dom.get(idx))
    }

    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&DOMNode> {
        self.dom
            .iter()
            .filter(|node| node.tag_name == tag_name)
            .collect()
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(rect.area(), 50.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        assert!(rect.contains((50.0, 50.0)));
        assert!(!rect.contains((5.0, 5.0)));
        assert!(!rect.contains((150.0, 150.0)));
    }

    #[test]
    fn test_box_model() {
        let box_model = BoxModel::new();
        assert_eq!(box_model.margin_width(), 0.0);
        assert_eq!(box_model.padding_width(), 0.0);
        assert_eq!(box_model.border_width(), 0.0);
    }

    #[test]
    fn test_dom_node_creation() {
        let node = DOMNode::new("test", NodeType::Element)
            .with_tag_name("div")
            .with_text("Hello");

        assert_eq!(node.id, "test");
        assert_eq!(node.tag_name, "div");
        assert_eq!(node.text_content, "Hello");
    }

    #[test]
    fn test_layout_engine() {
        let mut engine = LayoutEngine::new();
        engine.parse_html("<html></html>");

        assert_eq!(engine.dom.len(), 1);
        assert!(engine.get_element_by_id("root").is_some());
    }
}
