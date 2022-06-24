%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin

@storage_var
func bool() -> (bool : felt):
end

@external
func toggle{syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}():
    let (value) = bool.read()

    if value == 0:
        bool.write(1)
    else:
        bool.write(0)
    end

    return ()
end


@view
func view_bool{syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}() -> (
    bool : felt
):
    return bool.read()
end
