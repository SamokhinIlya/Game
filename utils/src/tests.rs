#[cfg(test)]
mod tests {
    #[test]
    fn point_clamp_test() {
        let bound0 = (0, 0);
        let bound1 = (10, 15);
        let valid_point = (3, 4);
        let top_left_invalid_point = (-1, -28);
        let bottom_right_invalid_point = (15, 388);

        let mut check = valid_point;
        point_clamp(&mut check, bound0, bound1);        
        assert_eq!(check, valid_point);

        let mut check = top_left_invalid_point;
        point_clamp(&mut check, bound0, bound1);        
        assert_eq!(check, bound0);

        let mut check = bottom_right_invalid_point;
        point_clamp(&mut check, bound0, bound1);        
        assert_eq!(check, bound1);
    }
}