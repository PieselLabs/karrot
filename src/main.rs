use crate::archtype::{Archtype, ArchtypeOps, Component, Layout};

mod archtype;


struct Pos {
    x: f32,
    y: f32,
}

impl Component for Pos {}

struct Vel {
    v: f32,
}

impl Component for Vel {}

fn main() {
    let mut layout = Layout::new();
    layout.add_component::<Pos>();
    layout.add_component::<Vel>();

    let mut archtype = Archtype::new(layout, 1024);
    archtype.add_components((Pos { x: 0.3, y: 0.12 }, Vel { v: 12.3 }));
}
