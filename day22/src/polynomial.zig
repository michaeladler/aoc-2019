const std = @import("std");
const testing = std.testing;
const log = std.log;

const numtheory = @import("numtheory");
const mul_mod = numtheory.mul_mod;

/// Represents p(x)=a*x+b
pub const Polynomial = struct {
    const Self = @This();

    a: i64,
    b: i64,

    // Computes f * g  (mod m)
    pub fn composeWith(f: Self, g: Self, m: i64) Polynomial {
        std.debug.assert(m > 0);
        // f = a*x + b
        // g = a'*x + b'
        // (f * g)(x) = f(a'x+b') = a*(a'x+b') + b = aa' * x + (ab' + b)
        // FIXME: incorrect result when directly returning, i.e. return Polynomial { ... }
        // See https://github.com/ziglang/zig/issues/4021
        const new_b = @mod(mul_mod(f.a, g.b, m) + f.b, m);
        return Polynomial{ .a = mul_mod(f.a, g.a, m), .b = new_b };
    }

    /// Compute f^e mod m (composition).
    pub fn power(f: Self, e: u64, m: i64) Polynomial {
        var x = Polynomial{ .a = 1, .b = 0 };
        var y = f;
        var ee = e;

        while (ee != 0) {
            if (ee & 1 != 0) x = x.composeWith(y, m);
            y = y.composeWith(y, m);
            ee = ee >> 1;
        }

        return x;
    }

    pub fn eval(self: Self, x: i64, m: i64) i64 {
        std.debug.assert(m > 0);
        return @mod(self.a * x + self.b, m);
    }
};

test "composeWith is associative" {
    const p1 = Polynomial{ .a = 1, .b = -6 }; // x - 6
    const p2 = Polynomial{ .a = 7, .b = 0 }; // 7x
    const p3 = Polynomial{ .a = -1, .b = -1 }; // -x-1

    const m: i64 = 10;

    var result = p2.composeWith(p1, m);
    try testing.expectEqual(@as(i64, 7), result.a);
    try testing.expectEqual(@as(i64, 8), result.b);

    result = p3.composeWith(result, m);
    try testing.expectEqual(@as(i64, 3), result.a);
    try testing.expectEqual(@as(i64, 1), result.b);

    result = p3.composeWith(p2, m).composeWith(p1, m);
    try testing.expectEqual(@as(i64, 3), result.a);
    try testing.expectEqual(@as(i64, 1), result.b);
}

test "composeWith 2" {
    const p1 = Polynomial{ .a = -1, .b = -1 };
    const p2 = Polynomial{ .a = 1, .b = -8 };

    const m: i64 = 10;

    var result = p2.composeWith(p1, m);
    try testing.expectEqual(@as(i64, 9), result.a);
    try testing.expectEqual(@as(i64, 1), result.b);
}
