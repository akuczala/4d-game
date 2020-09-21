use crate::vector::{VectorTrait,Field};
use core::marker::PhantomData;
use crate::geometry::Shape;
use crate::input::Input;
use specs::prelude::*;

pub struct Coin;

const SPIN_SPEED : Field = 2.0;
impl Component for Coin {
    type Storage = VecStorage<Self>;
}

pub struct CoinSpinningSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for CoinSpinningSystem<V> {

    type SystemData = (ReadStorage<'a,Coin>,ReadExpect<'a,Input>,WriteStorage<'a,Shape<V>>);

    fn run(&mut self, (read_coin, input, mut write_shape) : Self::SystemData) {

        for (_c, shape) in (&read_coin, &mut write_shape).join() {
        	shape.rotate(0,-1,SPIN_SPEED*(input.frame_duration as Field));
        }

    }
}