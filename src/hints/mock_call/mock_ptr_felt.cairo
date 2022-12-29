from starkware.cairo.common.registers import get_label_location
from starkware.cairo.common.alloc import alloc

func test_mock_call() {
    alloc_locals;
    let (mock_value: felt*)  = alloc();
    assert [mock_value] = 1;
    assert [mock_value + 1] = 2;
    let mock_value_len = 2;
    let func_to_mock = get_label_location(mocked_func);
    %{ mock_call(func_to_mock, mock_value_len, mock_value) %}
    let x = mocked_func();
    assert x[0] = 1;
    assert x[1] = 2;
    return ();
}

func mocked_func() -> felt* {
    let (x: felt*) = alloc();
    assert [x] = 10;
    assert [x + 1] = 11;
    assert [x + 2] = 12;
    assert [x + 3] = 13;
    return x;
}
