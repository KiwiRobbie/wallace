use azalea::blocks::BlockState;
use wallace::aabb::aabb_3d::*;

fn main() {
    let state = BlockState { id: 12497 };
    dbg!(state);

    {
        let a = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([0.5, 0.0, 0.0, 1.5, 1.0, 1.0]);
        assert_eq!(vec![Aabb3D([0.0, 0.0, 0.0, 1.5, 1.0, 1.0])], a.union(&b));
        assert_eq!(vec![Aabb3D([0.0, 0.0, 0.0, 1.5, 1.0, 1.0])], b.union(&a));
    }
}
