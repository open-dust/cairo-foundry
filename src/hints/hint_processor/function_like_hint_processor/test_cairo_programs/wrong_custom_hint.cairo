func test_wrong_custom_hint() {
    %{ wrong_custom_hint) wrong ( %}
    assert 2 = 3;
    return ();
}