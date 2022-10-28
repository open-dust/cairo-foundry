func test_greater_than_hint() {
    let a = 17;
    let b = 30;
    // Use custom hint
    %{ print(ids.a > ids.b) %}
    return ();
}
