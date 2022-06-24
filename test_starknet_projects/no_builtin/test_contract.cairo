%lang starknet

from test_starknet_projects.no_builtin.contract import add

@external
func test_add():
    let (res) = add(2, 3)
    assert res = 5

    return ()
end
