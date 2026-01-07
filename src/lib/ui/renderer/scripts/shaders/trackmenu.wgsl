#include "/common/style.wgsl"
struct TrackMenu {
  style:  Style,
  x0:     f32,
  y0:     f32,
  width:  f32,
  height: f32

}
// TODO: Refactor into basic quad generator, + Style application
//
//   --- Build the track menu from a quad ---
//
//                           q2(x2,y2)
//        q1(xo,yo)          q1(x2,y2)
//          .-----------------.
//          |                /|
//          |               / |
//          |              /  |
//          |             /   |
//          |            /    |
//          |           /     |
//          |          /      |
//          |         /       |
//          |        /        |
//          |       /         |
//          |      /          |
//          |     /           |
//          |    /            |
//          |   /             |
//          |  /              |
//          | /               |
//           . _______________.
//        q1(x1,y1)            q2(x1,y1)
//        q2(xo, yo)
//
@vertex
fn vs(@builtin(vertex_index) i: u32 ) -> @builtin(position) vec4f {

  var pos = array<vec2f, 6>(
      vec2f(-1.0, 1.0),   // TOP LEFT -> q1
      vec2f(-1.0, -1.0),  // BOTTOM LEFT -> q1
      vec2f(1.0, 1.0),    // TOP RIGHT -> q1
      vec2f(-1.0, -1.0),  // BOTTOM LEFT -> q2
      vec2f(1.0, -1.0),   // BOTTOM RIGHT -> q2
      vec2f(1.0, 1.0)     // TOP RIGHT -> q2
  );
  return vec4f(pos[i], 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
  return vec4f(1.0,1.0,0.0,1.0); // TODO: Apply Style object
}
