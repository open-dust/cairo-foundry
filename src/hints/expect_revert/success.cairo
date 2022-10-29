func test_expect_revert() {
    %{ expect_revert() %}
    assert 2 = 3;
    return ();
}
