def sum_to(n: int) -> int:
    total: int = 0
    for i in range(n):
        total += i
    return total

print(sum_to(100000000))
