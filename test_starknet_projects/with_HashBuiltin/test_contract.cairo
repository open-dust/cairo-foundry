%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin
from test_starknet_projects.with_HashBuiltin.contract import view_bool, toggle

@external
func test_toggle_and_view{syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}():
    let (x) = view_bool()
    assert x = 0

    toggle()

    let (x) = view_bool()
    assert x = 1

    toggle()

    let (x) = view_bool()
    assert x = 0

    return ()
end
