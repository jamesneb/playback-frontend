use taffy::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compute_layout(vp_width: f32, vp_height: f32) -> Vec<f32> {
    let mut domtree: TaffyTree<()> = TaffyTree::new();

    let track_menu = domtree.new_leaf(
        Style {
            size: Size {width: percent(0.2), height: percent(0.4)},
            ..Default::default()
        },
    ).unwrap();

    let timeline = domtree.new_leaf(
        Style {
            size: Size {width: percent(0.8), height: percent(0.4)},
            ..Default::default()
        }
    ).unwrap();

    let root = domtree.new_with_children(
        Style {
            display:        Display::Flex,
            flex_direction: FlexDirection::Row,
            size:           Size {
                                width:  length(vp_width),
                                height: length(vp_height),

            },
            ..Default::default()
        },
        &[track_menu, timeline],
    ).unwrap();

    let _ = domtree.compute_layout(root, Size::MAX_CONTENT);

    let tm = domtree.layout(track_menu).unwrap();
    let tl = domtree.layout(timeline).unwrap();

    vec![tm.location.x, tm.location.y, tm.size.width, tm.size.height,
         tl.location.x, tl.location.y, tl.size.width, tl.size.height]

}


