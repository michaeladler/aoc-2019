const std = @import("std");
const debug = std.debug;
const testing = std.testing;

// taken from https://github.com/libntl/ntl/blob/main/src/ZZ.cpp
// Compute s, t, d such that aa * s + bb * t = d = gcd(a,b)
pub fn xgcd(d: *i64, s: *i64, t: *i64, aa: i64, bb: i64) void {
    var u: i64 = undefined;
    var v: i64 = undefined;
    var uu0: i64 = undefined;
    var v0: i64 = undefined;
    var uu1: i64 = undefined;
    var v1: i64 = undefined;
    var uu2: i64 = undefined;
    var v2: i64 = undefined;
    var q: i64 = undefined;
    var r: i64 = undefined;

    var aneg = false;
    var bneg = false;
    var a = aa;
    var b = bb;
    if (a < 0) {
        a = -a;
        aneg = true;
    }

    if (b < 0) {
        b = -b;
        bneg = true;
    }

    uu1 = 1;
    v1 = 0;
    uu2 = 0;
    v2 = 1;
    u = a;
    v = b;

    while (v != 0) {
        q = @divFloor(u, v);
        r = @mod(u, v);
        u = v;
        v = r;
        uu0 = uu2;
        v0 = v2;
        uu2 = uu1 - q * uu2;
        v2 = v1 - q * v2;
        uu1 = uu0;
        v1 = v0;
    }

    if (aneg)
        uu1 = -uu1;

    if (bneg)
        v1 = -v1;

    d.* = u;
    s.* = uu1;
    t.* = v1;
}

// NTL
pub fn inv_mod(a: i64, n: i64) ?i64 {
    var d: i64 = undefined;
    var s: i64 = undefined;
    var t: i64 = undefined;

    xgcd(&d, &s, &t, a, n);
    if (d != 1) {
        return null;
    }
    if (s < 0) {
        return s + n;
    } else {
        return s;
    }
}

// https://www.geeksforgeeks.org/how-to-avoid-overflow-in-modular-multiplication/
pub fn mul_mod(a: i64, b: i64, n: i64) i64 {
    var res: i64 = 0;
    var a2 = @mod(a, n);
    var b2 = @mod(b, n);
    while (b2 > 0) {
        // If b is odd, add 'a' to result
        if (@mod(b2, 2) == 1)
            res = @mod(res + a2, n);

        // Multiply 'a' with 2
        a2 = @mod(a2 * 2, n);

        // Divide b by 2
        b2 = @divFloor(b2, 2);
    }
    return res;
}

// NTL
pub fn power_mod(a: i64, e: i64, n: i64) i64 {
    var x: i64 = 1;
    var y = a;
    var ee: u64 = if (e >= 0) @as(u64, @intCast(e)) else @as(u64, @intCast(-e));

    while (ee != 0) {
        if (ee & 1 != 0) x = mul_mod(x, y, n);
        y = mul_mod(y, y, n);
        ee = ee >> 1;
    }

    if (e < 0) x = inv_mod(x, n).?;

    return x;
}

test "inv_mod" {
    testing.expectEqual(@as(i64, 2), inv_mod(3, 5).?);
    testing.expectEqual(@as(i64, 3), inv_mod(2, 5).?);
    testing.expectEqual(@as(i64, 77695236753), inv_mod(6003722857, 77695236973).?);
    // Worst case set of numbers are the two largest fibonacci numbers: < 2^63 a = 4660046610375530309 , n = 7540113804746346429 , which will take 90 loops. In this case, (1/a mod n) == a.
    testing.expectEqual(@as(i64, 4660046610375530309), inv_mod(4660046610375530309, 7540113804746346429).?);
}

test "mul_mod" {
    const a: i64 = 9223372036854775807;
    const b: i64 = 9223372036854775807;
    testing.expectEqual(@as(i64, 84232501249), mul_mod(a, b, 100000000000));
    testing.expectEqual(@as(i64, 4), mul_mod(@as(i64, -2), @as(i64, 3), @as(i64, 10)));
}

test "power_mod" {
    // Fermat
    const p: i64 = 17;
    testing.expectEqual(@as(i64, 1), power_mod(2, p - 1, p));
}
