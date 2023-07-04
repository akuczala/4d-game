use crate::geometry::Line;
use crate::vector::{Field, Vec3};

pub fn build_floor3(n: usize, scale: Field, h: Field) -> Vec<Line<Vec3>> {
    let r = scale * (n as Field);

    (-(n as i32)..(n as i32) + 1)
        .map(|i| {
            Line(
                Vec3::new(scale * (i as Field), h, -r),
                Vec3::new(scale * (i as Field), h, r),
            )
        })
        .chain((-(n as i32)..(n as i32) + 1).map(|i| {
            Line(
                Vec3::new(-r, h, scale * (i as Field)),
                Vec3::new(r, h, scale * (i as Field)),
            )
        }))
        .collect()
}
