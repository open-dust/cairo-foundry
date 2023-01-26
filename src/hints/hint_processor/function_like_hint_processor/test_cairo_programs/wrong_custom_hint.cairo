func test_wrong_custom_hint() {
    %{ wrong_custom_hint ) Should be not ok ( %}
    assert 2 = 3;
    return ();
}