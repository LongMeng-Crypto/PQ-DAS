# Error: multidimensional indexing is not supported in assignment targets.
# The parser used to silently drop the extra indices, compiling this as
# `arr[0] = 5`.
def main():
    arr = Array(4)
    arr[0][1] = 5
    assert arr[0] == 5
    return
