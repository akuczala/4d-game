use std::marker::PhantomData;

use specs::prelude::*;
use specs::{Component};

use crate::components::Shape;
use crate::vector::{VectorTrait, Field};

//axis-aligned bounding box
#[derive(Component,Debug,Clone)]
#[storage(VecStorage)]
pub struct BBox<V : VectorTrait> {
	pub min : V,
	pub max : V,
}
impl<V: VectorTrait> BBox<V> {
	pub fn max_length(&self) -> Field {
		(self.max - self.min).fold(Some(0.0), |x,y| match x > y {true => x, false => y})
	}
	pub fn center(&self) -> V {
		(self.max + self.min)/2.0
	}
}

pub trait HasBBox<V : VectorTrait>: specs::Component {
	fn calc_bbox(&self) -> BBox<V>;
}

#[derive(Default)]
pub struct UpdateBBoxSystem<V: VectorTrait> {
    pub ph: PhantomData<V>,
    pub modified: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>
}

impl<'a,V: VectorTrait> System<'a> for UpdateBBoxSystem<V> {

	type SystemData = (
		ReadStorage<'a, Shape<V>>,
		WriteStorage<'a, BBox<V>>
	);

	fn run(
        &mut self, (
            read_shape,
            mut write_bbox
        ) : Self::SystemData
    ) {
        self.modified.clear();
        let events = read_shape.channel().read(self.reader_id.as_mut().unwrap());
        for event in events {
            match event {
                ComponentEvent::Modified(id) => {self.modified.add(*id);},
                _ => (),
            }
        }
		for (_, shape, bbox) in (&self.modified, &read_shape, &mut write_bbox).join() {
			*bbox = shape.calc_bbox();
		}
	}

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }
}