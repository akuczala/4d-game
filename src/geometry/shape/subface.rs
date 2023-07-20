use super::{convex::ConvexSubFace, single_face::BoundarySubFace, VertIndex};

pub enum SubFace<V> {
    Convex(ConvexSubFace),
    Boundary(BoundarySubFace<V>),
}
