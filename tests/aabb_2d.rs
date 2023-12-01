#[cfg(test)]
mod aabb_2d_union {
    use wallace::aabb::aabb_2d::*;

    #[test]
    fn strict_subset() {
        let a = Aabb2D {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: -2.0,
            max_x: 2.0,
            min_y: -2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![b.clone()], a.union(&b));
        assert_eq!(vec![b.clone()], b.union(&a));
    }

    #[test]
    fn subset() {
        let a = Aabb2D {
            min_x: -2.0,
            max_x: 2.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: -1.0,
            min_y: -1.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone()], a.union(&b));
        assert_eq!(vec![a.clone()], b.union(&a));
    }

    #[test]
    fn subset_self() {
        let a = Aabb2D {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone()], a.union(&a));
    }

    #[test]
    fn disjoint() {
        let a = Aabb2D {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: 2.0,
            max_x: 4.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn disjoint_touching_corner() {
        let a = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn disjoint_touching_edge() {
        let a = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: 1.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], a.union(&b));
        assert_eq!(vec![b.clone(), a.clone()], b.union(&a));
    }

    #[test]
    fn complex_corner() {
        let a = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Aabb2D {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 7.0f32;

        {
            let result = a.union(&b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
        {
            let result = b.union(&a);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
    }

    #[test]
    fn complex_edge() {
        let a = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Aabb2D {
            min_x: -1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 10.0f32;
        {
            let result = a.union(&b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
        {
            const EXPECTED_AREA: f32 = 10.0f32;

            let result = a.union(&b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
    }
}

mod aabb_2d_subtract {
    use wallace::aabb::aabb_2d::Aabb2D;

    #[test]
    fn subtract_equal() {
        let a = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let c = a.subtract(&b);
        assert_eq!(c, vec![]);
    }
    #[test]
    fn subtract_superset() {
        let a = Aabb2D {
            min_x: -1.0,
            min_y: -1.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Aabb2D {
            min_x: -2.0,
            min_y: -2.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let c = a.subtract(&b);
        assert_eq!(c, vec![]);
    }
}
