use std::{marker::PhantomData, ops::Deref};

use specs::{BitSet, ReaderId, shrev::EventChannel, storage::ComponentEvent};

use crate::vector::VectorTrait;

#[derive(Default)]
pub struct ModSystem<V: VectorTrait> {
    pub ph: PhantomData<V>,
    pub modified: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>
}


impl<V: VectorTrait> ModSystem<V> {
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