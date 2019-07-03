use crate::vector::vec3::Vec3;
use crate::colors::YELLOW;
use crate::geometry::{Shape,buildshapes};
use crate::vector::{VectorTrait,Field};

pub fn build_corridor_cross<V : VectorTrait>(cube : &Shape<V>, wall_length : Field) -> Vec<Shape<V>> {

    let corr_width = 1.0;
    let wall_height = 1.0;
    //let origin = V::zero();
    let signs = vec![-1.0,1.0];
    let axis_pairs = match V::DIM {
        3 => vec![(0,2)],
        4 => vec![(0,2),(2,3),(3,0)],
        _ => panic!("Invalid dimension for build_corridor_cross")
    };
    let axes = match V::DIM {
        3 => (-1..1),
        4 => (-2..1),
        _ => panic!("Invalid dimension for build_corridor_cross")
    };
    let mut shapes : Vec<Shape<V>> = Vec::new();
    //corridor walls
    let walls1 = iproduct!(signs.iter(),signs.iter(),axis_pairs.iter())
        .map(|(s1,s2,(ax1,ax2))| cube.clone()
            .set_pos(&(
                V::one_hot(*ax1)*(*s1)*(corr_width+wall_length)/2.0
                + V::one_hot(*ax2)*(*s2)*(corr_width+wall_length)/2.0
                ))
            .stretch(&(V::one_hot(1)*(wall_height - corr_width)
                + V::one_hot(*ax1)*(wall_length - corr_width)
                + V::one_hot(*ax2)*(wall_length - corr_width)
                + V::ones()*corr_width
                    ))
            );
    shapes.append(&mut walls1.collect());

    //end walls
    
    let walls2 = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .set_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)))
                .stretch(&(
                    V::one_hot(1)*(wall_height-corr_width) + V::ones()*corr_width
                    ))
                );
    shapes.append(&mut walls2.collect());
    //floors and ceilings
    let mut floors_long : Vec<Shape<V>> = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .set_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)/2.0
                    - V::one_hot(1)*(wall_height + corr_width)/2.0
                    ))
                .stretch(&(V::one_hot(i)*(wall_length-corr_width) + V::ones()*corr_width
                    ))
                ).collect();
    let mut ceilings_long : Vec<Shape<V>> = floors_long.iter()
        .map(|block| block.clone().set_pos(&(
            *block.get_pos()+V::one_hot(1)*(wall_height+corr_width))
        ))
        .collect();
    shapes.append(&mut floors_long);
    shapes.append(&mut ceilings_long);
    //center floor
    shapes.push(cube.clone().set_pos(&(-V::one_hot(1)*(wall_height + corr_width)/2.0)));
    shapes
    
}

pub fn build_lvl_1_3d() -> Vec<Shape<Vec3>> {
    let wall_length = 3.0;
    let mut shapes = build_corridor_cross(
        &buildshapes::color_cube(buildshapes::build_cube_3d(1.0)),wall_length);
    shapes.push(buildshapes::build_prism_3d(0.1,0.025,6)
        .set_color(YELLOW)
        .set_pos(&Vec3::new(wall_length - 0.5,0.0,0.0)));

    shapes
}