use serde::{Deserialize, Serialize};

use super::{convex::ConvexSubFace, single_face::BoundarySubFace, VertIndex};

#[derive(Clone, Serialize, Deserialize)]
pub enum SubFace<V> {
    Interior(ConvexSubFace),
    Boundary(BoundarySubFace<V>),
}
