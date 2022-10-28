func test_assert_revert_hint() {
    %{ assert_revert() %}
    assert 2 = 3;
    return ();
}

func test_failing_assert_revert_hint() {
    %{ assert_revert() %}
    assert 2 = 2;
    return ();
}
