const std = @import("std");
const mem = std.mem;
const testing = std.testing;
const Allocator = mem.Allocator;

pub fn combinations(n: u32, indices: []u32) CombinationsIterator {
    return CombinationsIterator{ .n = n, .indices = indices };
}

// Based on https://docs.python.org/3/library/itertools.html#itertools.combinations
pub const CombinationsIterator = struct {
    const Self = @This();

    n: u32,
    indices: []u32,

    is_first: bool = true,
    done: bool = false,

    pub fn next(self: *Self) ?[]u32 {
        if (self.done) return null;
        const n = self.n;
        const r = self.indices.len;
        if (r > n or r == 0) {
            self.done = true;
            return null;
        }
        if (self.is_first) {
            self.is_first = false;
            return self.indices;
        }

        var i = r - 1;
        outer: {
            while (true) {
                if (self.indices[i] != i + n - r) break :outer;
                if (i == 0) break;
                i -= 1;
            }
            self.done = true;
            return null;
        }
        self.indices[i] += 1;
        var j = i + 1;
        while (j < r) : (j += 1) {
            self.indices[j] = self.indices[j - 1] + 1;
        }
        return self.indices;
    }
};

test "combinations" {
    const expected = [_][3]u32{
        [_]u32{ 0, 1, 2 },
        [_]u32{ 0, 1, 3 },
        [_]u32{ 0, 1, 4 },
        [_]u32{ 0, 2, 3 },
        [_]u32{ 0, 2, 4 },
        [_]u32{ 0, 3, 4 },
        [_]u32{ 1, 2, 3 },
        [_]u32{ 1, 2, 4 },
        [_]u32{ 1, 3, 4 },
        [_]u32{ 2, 3, 4 },
    };

    var indices = [_]u32{ 0, 1, 2 };
    var it = combinations(5, indices[0..]);
    var i: usize = 0;
    while (it.next()) |slice| {
        testing.expectEqualSlices(u32, expected[i][0..], slice);
        i += 1;
    }
}
