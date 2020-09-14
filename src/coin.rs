use crate::vector::VectorTrait;
use core::marker::PhantomData;
use crate::geometry::Shape;
use specs::prelude::*;

pub struct Coin;

impl Component for Coin {
    type Storage = VecStorage<Self>;
}

pub struct CoinSpinningSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for CoinSpinningSystem<V> {

    type SystemData = (ReadStorage<'a,Coin>,WriteStorage<'a,Shape<V>>);

    fn run(&mut self, (read_coin, mut write_shape) : Self::SystemData) {

        for (_c, shape) in (&read_coin, &mut write_shape).join() {
        	shape.rotate(0,-1,0.05);
        }

    }
}