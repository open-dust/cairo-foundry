from starkware.cairo.common.registers import get_label_location

func test_mock_call() {
    let mock_ret_value = 42;
    let func_to_mock = get_label_location(mocked_func);
    %{ mock_call(func_to_mock, mock_ret_value) %}
    let x = mocked_func();
    assert 42 = x;
    return ();
}

func mocked_func() -> felt {
    assert 21 = 42;
    return 21;
}
