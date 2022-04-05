use serde::{Deserialize, Serialize};

pub type Color = [u8; 4];
/// This type is used anywere that needs to specify a dimension of some sorts,
/// like a position, a width, or border radius
pub type Dim = f32;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Tag {
    pub style_type: StyleType,
    pub style_base: StyleBase,
    pub children: Vec<Tag>,
}

/// These are ALL base tags (= tags that can be rendered
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum StyleType {
    Flex,
    Inline {
        flow_horizontal: FlowHorizontal,
        flow_vertical: FlowVertical,
    },
    Table {
        size_method: TableSizeMethod,
        columns: u32,
        rows: u32,
    },
    Newline,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum TableSizeMethod {
    Auto,
    Fixed,
}

/// This is style information apparent on any tag
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct StyleBase {
    pub width: Dim,
    pub height: Dim,
    pub border_radius: Dim,
    pub stroke_width: Dim,
    pub fill_color: Color,
    pub outline_color: Color,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum FlowHorizontal {
    LeftToRight,
    RightToLeft,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum FlowVertical {
    TopToBottom,
    BottomToTop,
}
