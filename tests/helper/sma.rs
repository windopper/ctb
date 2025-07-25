use ctb::helper::sma::calculate_sma;

#[test]
pub fn test_calculate_sma() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let sma = calculate_sma(&data, 3);
    assert_eq!(sma, Some(9.0));

    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let sma = calculate_sma(&data, 5);
    assert_eq!(sma, Some(8.0));

    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let sma = calculate_sma(&data, 7);
    assert_eq!(sma, Some(7.0));
}

