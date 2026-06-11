# Error: multidimensional indexing is not supported in compound assignment
# targets. The parser used to lower the target to `arr[0]` (dropping the
# second index); the program was still rejected, but only because the
# desugared RHS read `arr[0][1]` is caught downstream. This file pins the
# dedicated parse-time check on the target itself.
def main():
    arr = Array(4)
    arr[0] = 1
    arr[0][1] += 1
    return
