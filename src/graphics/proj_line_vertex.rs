#[derive(Copy, Clone)]
pub struct NewVertex {
    pub position: [f32; 3],
    pub color : [f32 ; 4],
    pub direction : f32,
    pub next : [f32 ; 3],
    pub previous : [f32 ; 3],
}
impl Default for NewVertex {
    fn default() -> Self {
        Self{
            position : [0.,0.,0.],
            color : [0.,0.,0.,0.],
            direction : 0.,
            next : [0.,0.,0.],
            previous : [0.,0.,0.],
        }
    }