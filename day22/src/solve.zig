const std = @import("std");
const mem = std.mem;
const log = std.log;
const testing = std.testing;
const Allocator = mem.Allocator;
const AutoHashMap = std.AutoHashMap;

const readInput = @import("parser.zig").readInput;
const Instruction = @import("instruction.zig").Instruction;
// for testing
const Deck = @import("deck.zig").Deck;

pub fn part1(allocator: Allocator) !i64 {
    var instructions = try readInput(allocator);
    defer instructions.deinit();

    const m = 10007;
    const poly = Instruction.manyToPolynomial(instructions.items, m);
    return poly.eval(2019, m);
}

fn part2_helper(allocator: Allocator, instructions: []const Instruction, p: i64, count: usize) !i64 {
    var inv_instructions = try Instruction.invertMany(allocator, instructions, p);
    defer allocator.free(inv_instructions);

    const poly = Instruction.manyToPolynomial(inv_instructions, p).power(count, p);
    return poly.eval(2020, p);
}

pub fn part2(allocator: Allocator) !i64 {
    // Key insight: all positions are `mod m` and correspond to a linear polynomial mod m.
    // So, shuffle(x) = (f_n * ... f_1)(x) = p(x) for some linear polynomial p.
    // Hence, (f_n * ... f_1)^n = p^n and we need to use "exponentiation by squaring"
    // (here, the term exponentiation means function composition).
    // Since we have to calculate the starting position, we need to invert the instructions first,
    // i.e. shuffle^{-1} = f_1^{-1} * ... f_n^{-1} = g(x) and then compute g^k(x) for large k,
    // where k is part of the puzzle input.
    var instructions = try readInput(allocator);
    defer instructions.deinit();
    const p = 119315717514047;
    const count = 101741582076661;
    return part2_helper(allocator, instructions.items, p, count);
}

test "2019 Day 22, Part 1" {
    const answer = try part1(testing.allocator);
    try testing.expectEqual(@as(i64, 1867), answer);
}

test "2019 Day 22, Part 2" {
    const answer = try part2(testing.allocator);
    try testing.expectEqual(@as(i64, 71047285772808), answer);
}

test "2019 Day 22, Part 2 Small" {
    var instructions = try readInput(testing.allocator);
    defer instructions.deinit();

    const p = 10007;
    const count = 1;
    const answer = try part2_helper(testing.allocator, instructions.items, p, count);

    // verify answer
    var deck = try Deck.init(testing.allocator, p);
    defer deck.deinit();
    var i: usize = 0;
    while (i < count) : (i += 1) {
        try deck.apply_instructions(instructions.items);
    }
    try testing.expectEqual(@as(usize, 9596), deck.get_card(2020).?);
    try testing.expectEqual(@as(usize, 2020), deck.find_card(@intCast(usize, 9596)).?);
    try testing.expectEqual(@as(usize, 2020), deck.find_card(@intCast(usize, answer)).?);
    try testing.expectEqual(@as(i64, 9596), answer);
}
