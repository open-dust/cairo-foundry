%builtins output

from starkware.cairo.common.registers import get_label_location
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.serialize import serialize_word
from starkware.cairo.common.memcpy import memcpy


func test_mock_call{output_ptr: felt*}() {
    alloc_locals;

    // Initialize a mock array: mock_value = [1, 2]
    let (mock_value: felt*)  = alloc();
    assert [mock_value] = 1;
    assert [mock_value + 1] = 2;

    // The associated len is 2
    let mock_value_len = 2;
    let func_to_mock = get_label_location(mocked_func);

    %{ mock_call(func_to_mock, mock_value_len, mock_value) %}
    let x = mocked_func();

    let (y: felt*) = alloc();
    memcpy(y, x, 2);

    return ();
}


// This functions is supposed to fill the empty array x with 4 values, thus initializing x to be [10, 11, 12, 13]
func mocked_func() -> felt* {
    alloc_locals;
    let (local x: felt*) = alloc();
    assert [x] = 10;
    assert [x + 1] = 11;
    assert [x + 2] = 12;
    assert [x + 3] = 13;
    return x;
}


