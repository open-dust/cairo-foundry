%lang starknet

@external
func add(lhs: felt, rhs: felt) -> (res: felt):
    return (lhs + rhs)
end
