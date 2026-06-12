# Error: multidimensional indexing is only supported on compile-time const
# arrays. Same as error_74, but as a bare statement RHS (this path used to
# panic in handle_array_assignment instead of returning a compile error).
def main():
    arr = Array(4)
    x = arr[0][1]
    assert x == 0
    return
