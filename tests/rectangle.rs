#[cfg(test)]
mod tests {
    use wallace::aabb::rectangle::*;

    #[test]
    fn strict_subset() {
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: -2.0,
            max_x: 2.0,
            min_y: -2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone()], union(&b, &a));
    }

    #[test]
    fn superset() {
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone()], union(&a, &a));
    }

    #[test]
    fn disjoint() {
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 2.0,
            max_x: 4.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }

    #[test]
    fn disjoint_touching_corner() {
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }

    #[test]
    fn disjoint_touching_edge() {
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }

    #[test]
    fn complex_corner() {
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 7.0f32;

        {
            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
        {
            let result = union(&b, &a);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
    }

    #[test]
    fn complex_edge() {
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Rectangle {
            min_x: -1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 10.0f32;
        {
            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
        {
            const EXPECTED_AREA: f32 = 10.0f32;

            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
    }
}
