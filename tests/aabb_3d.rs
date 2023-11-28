#[cfg(test)]
mod aabb_3d_union {
    use wallace::aabb::aabb_3d::*;

    #[test]
    fn strict_subset() {
        let a = Aabb3D([-2.0, -2.0, -2.0, 2.0, 2.0, 2.0]);
        let b = Aabb3D([-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
        assert_eq!(vec![a.clone()], a.union(&b));
        assert_eq!(vec![a.clone()], b.union(&a));
    }

    #[test]
    fn subset() {
        let a = Aabb3D([-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([-1.0, -2.0, -1.0, 1.0, 2.0, 1.0]);

        assert_eq!(vec![b.clone()], a.union(&b));
        assert_eq!(vec![b.clone()], b.union(&a));
    }

    #[test]
    fn subset_self() {
        let a = Aabb3D([-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
        assert_eq!(vec![a.clone()], a.union(&a));
    }

    #[test]
    fn disjoint() {
        let a = Aabb3D([-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([-1.0, 2.0, -1.0, 1.0, 4.0, 1.0]);

        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn disjoint_touching_corner() {
        let a = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);

        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn disjoint_touching_edge() {
        let a = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([0.0, 1.0, 1.0, 1.0, 2.0, 2.0]);
        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn disjoint_touching_face() {
        let a = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([0.0, 0.0, 1.0, 1.0, 1.0, 2.0]);
        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }
    #[test]
    fn overlap_1_axis() {
        let a = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Aabb3D([0.5, 0.0, 0.0, 1.5, 1.0, 1.0]);
        assert_eq!(vec![Aabb3D([0.0, 0.0, 0.0, 1.5, 1.0, 1.0])], a.union(&b));
        assert_eq!(vec![Aabb3D([0.0, 0.0, 0.0, 1.5, 1.0, 1.0])], b.union(&a));
    }
}
