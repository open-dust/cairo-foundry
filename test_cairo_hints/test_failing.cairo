func test_failing_test() {
    assert 2 = 3;
    return ();
}

func test_not_failing_test() {
    assert 2 = 2;
    return ();
}
