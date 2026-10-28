def is_even(x):
    if x % 2 == 0:
        return True
    return False

n = 0
for i in range(1, 6):
    if is_even(i):
        n += i
print(n)

count = 0
while count < 3:
    count += 1
print(count)
