%builtins output

// Infinite loop
func test_infinite_loop_failing_test{
    output_ptr: felt*
}() {
    %{ expect_revert() %}
    test_infinite_loop_failing_test();
    return();
}
