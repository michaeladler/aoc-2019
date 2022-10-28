const std = @import("std");
const testing = std.testing;
const log = std.log;
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;

const numtheory = @import("numtheory");
const inv_mod = numtheory.inv_mod;
const Polynomial = @import("polynomial.zig").Polynomial;

// only used for testing
const Deck = @import("deck.zig").Deck;

pub const InstructionTag = enum {
    deal_into,
    cut_n,
    deal_inc,
};

pub const Instruction = union(InstructionTag) {
    const Self = @This();

    deal_into: void,
    cut_n: i64,
    deal_inc: i64,

    pub fn invert(item: Self, len: i64) Self {
        switch (item) {
            .deal_into => {
                return item;
            },
            .cut_n => {
                const n = item.cut_n;
                const val = blk: {
                    if (n >= 0) {
                        break :blk len - n;
                    } else {
                        break :blk -n;
                    }
                };
                return Instruction{ .cut_n = val };
            },
            .deal_inc => {
                const n = item.deal_inc;
                const val = inv_mod(n, len).?;
                return Instruction{ .deal_inc = val };
            },
        }
    }

    /// Compute (f_n * ... f_1)^{-1} = f_1^{-1} * ... f_n^{-1}.
    /// Caller must free result.
    pub fn invertMany(allocator: Allocator, instructions: []const Instruction, m: i64) ![]const Instruction {
        var result = try ArrayList(Instruction).initCapacity(allocator, instructions.len);

        if (instructions.len > 0) {
            var i: usize = instructions.len - 1;
            while (true) {
                result.appendAssumeCapacity(instructions[i].invert(m));
                if (i == 0) break;
                i -= 1;
            }
        }
        return result.toOwnedSlice();
    }

    pub fn toPolynomial(item: Self) Polynomial {
        switch (item) {
            .deal_into => {
                return Polynomial{ .a = -1, .b = -1 }; // -x-1
            },
            .cut_n => {
                const n = item.cut_n;
                return Polynomial{ .a = 1, .b = -n }; // x-n
            },
            .deal_inc => {
                const n = item.deal_inc;
                return Polynomial{ .a = n, .b = 0 }; // n*x
            },
        }
    }

    pub fn manyToPolynomial(instructions: []const Instruction, m: i64) Polynomial {
        // instructions:  f0, ..., fk
        // polynomials: p0, ..., pk
        // result: pk * ... * p0
        const len = instructions.len;
        const inst = instructions[len - 1];
        var poly = inst.toPolynomial(); // start with last poly
        if (len >= 2) {
            var i: usize = len - 2;
            while (true) {
                const item = instructions[i];
                const curPoly = item.toPolynomial();
                poly = poly.composeWith(curPoly, m);
                if (i == 0) {
                    break;
                }
                i -= 1;
            }
        }
        return poly;
    }
};

test "manyToPolynomial" {
    const instructions = [_]Instruction{
        Instruction{ .cut_n = 6 },
        Instruction{ .deal_inc = 7 },
        Instruction{ .deal_into = {} },
    };
    const m = 10;
    const poly = Instruction.manyToPolynomial(instructions[0..], m);

    {
        var deck = try Deck.init(testing.allocator, m);
        defer deck.deinit();
        try deck.apply_instructions(instructions[0..]);

        var i: usize = 0;
        while (i < m) : (i += 1) {
            try testing.expectEqual(deck.find_card(i).?, @intCast(usize, poly.eval(@intCast(i64, i), m)));
        }
    }
}

test "manyToPolynomial 2" {
    const instructions = [_]Instruction{
        Instruction{ .deal_into = {} },
        Instruction{ .cut_n = 8 },
        Instruction{ .deal_inc = 7 },

        Instruction{ .cut_n = 8 },
        Instruction{ .cut_n = -4 },
        Instruction{ .deal_inc = 7 },

        Instruction{ .cut_n = 3 },
        Instruction{ .deal_inc = 9 },
        Instruction{ .deal_inc = 3 },
        Instruction{ .cut_n = -1 },
    };
    const m = 10;
    const poly = Instruction.manyToPolynomial(instructions[0..], m);

    {
        var deck = try Deck.init(testing.allocator, 10);
        defer deck.deinit();
        try deck.apply_instructions(instructions[0..]);

        var i: usize = 0;
        while (i < m) : (i += 1) {
            try testing.expectEqual(deck.find_card(i).?, @intCast(usize, poly.eval(@intCast(i64, i), m)));
        }
    }
}

test "invertMany simple" {
    const instructions = [_]Instruction{
        Instruction{ .deal_inc = 7 },
    };

    const p = 10;
    const inv_instructions = try Instruction.invertMany(testing.allocator, instructions[0..], p);
    defer testing.allocator.free(inv_instructions);

    const f = Instruction.manyToPolynomial(instructions[0..], p);
    const g = Instruction.manyToPolynomial(inv_instructions, p);

    {
        const id = f.composeWith(g, p);
        try testing.expectEqual(@as(i64, 1), id.a);
        try testing.expectEqual(@as(i64, 0), id.b);
    }
    {
        const id = g.composeWith(f, p);
        try testing.expectEqual(@as(i64, 1), id.a);
        try testing.expectEqual(@as(i64, 0), id.b);
    }
}

test "invertMany" {
    const instructions = [_]Instruction{
        Instruction{ .deal_into = {} },
        Instruction{ .cut_n = 8 },
        Instruction{ .deal_inc = 7 },

        Instruction{ .cut_n = 8 },
        Instruction{ .cut_n = -4 },
        Instruction{ .deal_inc = 7 },

        Instruction{ .cut_n = 3 },
        Instruction{ .deal_inc = 9 },
        Instruction{ .deal_inc = 3 },
        Instruction{ .cut_n = -1 },
    };

    const p = 10;
    const inv_instructions = try Instruction.invertMany(testing.allocator, instructions[0..], p);
    defer testing.allocator.free(inv_instructions);

    const f = Instruction.manyToPolynomial(instructions[0..], p);
    const g = Instruction.manyToPolynomial(inv_instructions, p);

    {
        const id = f.composeWith(g, p);
        try testing.expectEqual(@as(i64, 1), id.a);
        try testing.expectEqual(@as(i64, 0), id.b);
    }
    {
        const id = g.composeWith(f, p);
        try testing.expectEqual(@as(i64, 1), id.a);
        try testing.expectEqual(@as(i64, 0), id.b);
    }
}
