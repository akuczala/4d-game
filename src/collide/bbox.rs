use std::marker::PhantomData;

use specs::prelude::*;
use specs::{Component};

use crate::components::Shape;
use crate::ecs_utils::ModSystem;
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
pub struct UpdateBBoxSystem<V: VectorTrait>(pub ModSystem<V>);

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
        self.0.gather_events(read_shape.channel());
		for (_, shape, bbox) in (&self.0.modified, &read_shape, &mut write_bbox).join() {
			*bbox = shape.calc_bbox();
		}
	}

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }
}