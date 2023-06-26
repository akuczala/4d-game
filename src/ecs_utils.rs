use std::{marker::PhantomData, ops::Deref};

use specs::{BitSet, ReaderId, shrev::EventChannel, storage::ComponentEvent};

use crate::vector::{VectorTrait, Vec2, Vec3, Vec4, Mat2, Mat4, Mat3};

//the 'static lifetime here tells the compiler that any type with the componentable trait
//does not hold any references that might require lifetimes

pub trait Componentable: 'static + Sync + Send {}

#[derive(Default)]
pub struct ModSystem<V> {
    pub ph: PhantomData<V>,
    pub modified: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>
}


impl<V: Componentable> ModSystem<V> {
    pub fn typed_default(ph: PhantomData<V>) -> Self {
        Self {
            ph,
            modified: Default::default(),
            reader_id: Default::default()
        }
    }
    //I tried to make ReadStorage an argument here but the trait constraints were a nightmare
    pub fn gather_events(&mut self, channel: &EventChannel<ComponentEvent>)
    //where C: Component<Storage = T>, T: Tracked
    {
        self.modified.clear();
        let events = channel.read(self.reader_id.as_mut().unwrap());
        for event in events {
            match event {
                ComponentEvent::Modified(id) => {self.modified.add(*id);},
                _ => (),
            }
        }
    }
}

impl Componentable for Vec2 {}
impl Componentable for Vec3 {}
impl Componentable for Vec4 {}

impl Componentable for Mat2 {}
impl Componentable for Mat3 {}
impl Componentable for Mat4 {}