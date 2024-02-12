def get_random_values(amount: int):
    from random import randint
    l = []
    for i in range(amount):
        l.append((randint(0,1000),randint(0, i)))
    return l

if __name__ == "__main__":
    with open("constant_values.rs", "w") as file:
        for i in [100, 500, 1000, 5000, 10000]:
            file.write(f"pub const VALUES_{i}: [(i32, usize); {i}] = ")
            file.write(str(get_random_values(i)))
            file.write(";\n")
        file.write("\n")
