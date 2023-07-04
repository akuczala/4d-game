use std::{marker::PhantomData, ops::Deref};

use specs::{
    hibitset::BitSetOr,
    shrev::{EventChannel, EventIterator},
    storage::ComponentEvent,
    BitSet, ReaderId,
};

use crate::vector::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4, VectorTrait};

//the 'static lifetime here tells the compiler that any type with the componentable trait
//does not hold any references that might require lifetimes

pub trait Componentable: 'static + Sync + Send {}

// pub trait VectorComponentable: VectorTrait + Componentable where
//     <Self as VectorTrait>::SubV: Componentable,
//     <Self as VectorTrait>::M: Componentable
//     {}

pub trait SystemName {
    const NAME: &'static str;
}
#[derive(Default)]
pub struct ModSystem<V> {
    pub ph: PhantomData<V>,
    pub modified: BitSet,
    pub inserted: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<V: Componentable> ModSystem<V> {
    pub fn typed_default(ph: PhantomData<V>) -> Self {
        Self {
            ph,
            modified: Default::default(),
            reader_id: Default::default(),
            inserted: Default::default(),
        }
    }
    pub fn get_events<'a>(
        &'a mut self,
        channel: &'a EventChannel<ComponentEvent>,
    ) -> EventIterator<ComponentEvent> {
        channel.read(self.reader_id.as_mut().unwrap())
    }
    //I tried to make ReadStorage an argument here but the trait constraints were a nightmare
    pub fn gather_events(&mut self, channel: &EventChannel<ComponentEvent>) {
        self.modified.clear();
        self.inserted.clear();
        for event in channel.read(self.reader_id.as_mut().unwrap()) {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                }
                ComponentEvent::Inserted(id) => {
                    self.inserted.add(*id);
                }
                _ => (),
            }
        }
    }
    pub fn modified_or_inserted(&self) -> BitSetOr<&BitSet, &BitSet> {
        (&self.modified) | (&self.inserted)
    }
    pub fn for_each_modified<F>(&mut self, channel: &EventChannel<ComponentEvent>, mut f: F)
    where
        F: FnMut(&u32) -> (),
    {
        for event in self.get_events(channel) {
            match event {
                ComponentEvent::Modified(id) => f(id),
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
