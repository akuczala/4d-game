use crate::ecs_utils::Componentable;
use crate::geometry::Line;
use crate::components::*;
use crate::vector::{Field,VectorTrait};
use crate::camera::Camera;
use crate::collide::MoveNext;
use specs::prelude::*;
use specs::{Component,HashMapStorage};
use std::marker::PhantomData;
use crate::collide::BBox;
use crate::geometry::shape::buildshapes::ShapeBuilder;
use crate::geometry::transform::Scaling;


pub struct Player(pub Entity); //specifies entity of player

pub fn build_player<V>(world : &mut World, transform: &Transform<V, V::M>)
where
    V: VectorTrait + Componentable,
    V::M: Componentable + Clone,
{
	let camera = Camera::new(&transform);
	let player_entity = world.create_entity()
		.with(transform.clone())
	    .with(BBox{min : V::ones()*(-0.1) + transform.pos, max : V::ones()*(0.1) + transform.pos})
	    .with(camera) //decompose
	    .with(MoveNext::<V>::default())
	    .with(MaybeTarget::<V>(None))
		.with(MaybeSelected::<V>(None))
	    .build();

    world.insert(Player(player_entity));

}

const MAX_TARGET_DIST : Field = 10.;

pub struct ShapeTargetingSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for ShapeTargetingSystem<V>
where
        V: VectorTrait + Componentable,
        V::M: Componentable
{
	type SystemData = (
		ReadExpect<'a,Player>,
		ReadStorage<'a,Transform<V, V::M>>,
		ReadStorage<'a,Shape<V>>,
		ReadStorage<'a,ShapeType<V>>,
		ReadStorage<'a,ShapeClipState<V>>,
		Entities<'a>,
		WriteStorage<'a,MaybeTarget<V>>);

	fn run(&mut self, (player, transforms, shapes, shape_types, shape_clip_state, entities, mut targets) : Self::SystemData) {
		let transform = transforms.get(player.0).expect("Player has no transform");
		let target = shape_targeting(&transform,(&shapes, &shape_types, &shape_clip_state,&*entities).join()); //filter by shapes having a clip state
		*targets.get_mut(player.0).expect("Player has no target") = target;
		

	}
}
#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Cursor;

pub struct MaybeTarget<V>(pub Option<Target<V>>);

pub struct MaybeSelected<V>(pub Option<Selected<V>>);

pub struct Selected<V> {
	pub entity: Entity,
	pub selection_box_shape: Shape<V>,
}
impl<V: VectorTrait> Selected<V> {
	pub fn new_from_bbox(entity: Entity, bbox: &BBox<V>) -> Self {
		Self::new_from_shape(entity, &Self::make_selection_box(bbox))
	}
	pub fn new_from_shape(entity: Entity, shape: &Shape<V>) -> Self {
		Selected{
			entity,
			selection_box_shape: shape.clone()
		}
	}
	fn make_selection_box(bbox: &BBox<V>) -> Shape<V> {
		let bbox_lengths = bbox.max - bbox.min;
		let mut shape: Shape<V> = ShapeBuilder::build_cube(1.0)
			.stretch(Scaling::Vector(bbox_lengths)).build();
		shape.update_from_ref(&shape.clone(),&Transform::pos(bbox.center()));
		shape
	}
	// fn update_selection_box(&mut self, bbox: &BBox<V>) {
	// 	//would like to update selection box shape without having to make a new one
	// }
}

pub struct Target<V> {
	pub entity : Entity,
	pub distance : Field,
	pub point : V,
	//pub all_points : Vec<V>,

}

fn shape_targeting<'a, V : VectorTrait + 'a, I>(transform : &Transform<V, V::M>, iter : I) -> MaybeTarget<V>
where
	//for<'a> &'a I: std::iter::Iterator<Item=(&'a Shape<V>, &'a ShapeType<V>,&'a ShapeClipState<V>, Entity)>
	I: std::iter::Iterator<Item=(&'a Shape<V>, &'a ShapeType<V>,&'a ShapeClipState<V>, Entity)>
{
	let pos = transform.pos;
	let dir = transform.frame[-1];
	let ray = Line(pos, pos + dir*MAX_TARGET_DIST);

	//loop through all shapes and check for nearest intersection
	let mut closest : Option<(Entity,Field,V)> = None;
	let mut all_points = Vec::<V>::new();
	for (shape, shape_type,shape_clip_state,e) in iter {

		for intersect_point in shape_type.line_intersect(shape, &ray, true, &shape_clip_state.face_visibility) { //find intersections of ray with visible faces
			all_points.push(intersect_point);
			let distsq = (intersect_point - pos).norm_sq();
			closest = match closest {
				Some((_cle, cldistsq, _clpoint)) => match distsq < cldistsq {
					true => Some((e,distsq,intersect_point)),
					false => closest,
				}
				None => Some((e,distsq,intersect_point)),
			}
		} 
	}
	match closest {
		Some((e,distsq,point)) => MaybeTarget(Some(Target{entity : e, distance : distsq.sqrt(), point})),
		None => MaybeTarget(None),
	}
}