//out of date test
fn test_shape_hash() {
	use specs::prelude::*;
	use crate::vector::Vec3;
	use crate::collide::StaticCollider;
	type V = Vec3;

	let mut world = World::new();
	world.register::<Shape<V>>();
	world.register::<StaticCollider>();
	world.register::<BBox<V>>();

	crate::build_level::build_shapes_3d(&mut world);

    //let shapes_len = shapes.len();
    //let coin_shape = shapes.pop();
    //add shape entities and intialize spatial hash set
    let (mut max, mut min) = (V::zero(), V::zero());
    let mut max_lengths = V::zero();
    for bbox in (&world.read_component::<BBox<V>>()).join() {
        min = min.zip_map(bbox.min,Field::min); 
        max = max.zip_map(bbox.max,Field::max);
        max_lengths = max_lengths.zip_map(bbox.max - bbox.min,Field::max);
    }
    println!("Min/max: {},{}",min,max);
    println!("Longest sides {}",max_lengths);
    max[1] = 10.0; min[1] = -10.0; //let's use only one cell vertically for the sake of testing
    max_lengths[1] = (max[1] - min[1])/2.0;
    world.insert(
		SpatialHashSet::<Vec3,Entity>::new(
			min*1.5, //make bounds slightly larger than farthest points
			max*1.5,
			max_lengths*1.1 //make cell size slightly larger than largest shape dimensions
		)
	);


    let mut dispatcher = DispatcherBuilder::new()
    	.with(UpdateBBoxSystem(PhantomData::<V>),"bbox",&[])
    	.with(BBoxHashingSystem(PhantomData::<V>),"hash",&["bbox"])
    	.build();

    dispatcher.dispatch(&mut world);

    let hash = world.read_resource::<SpatialHashSet<V, Entity>>();
    hash.print();
    for n in hash.0.n_cells.iter() {
    	println!("cells: {}",n);
    }



}