const std = @import("std");
const testing = std.testing;
const log = std.log;

pub const row_count = 5;
pub const col_count = 5;
pub const center = 2;

const BS = std.StaticBitSet(row_count * col_count);

pub const Grid = struct {
    const Self = @This();

    bs: BS,

    pub fn init() Self {
        const bs = BS.initEmpty();
        return Grid{ .bs = bs };
    }

    pub fn isBug(self: Self, row: usize, col: usize) bool {
        return self.bs.isSet(pos(row, col));
    }

    pub fn setBug(self: *Self, row: usize, col: usize) void {
        self.bs.setValue(pos(row, col), true);
    }

    pub fn clearBug(self: *Self, row: usize, col: usize) void {
        self.bs.setValue(pos(row, col), false);
    }

    pub fn countBugs(self: Self) usize {
        return self.bs.count();
    }

    pub fn countBugsRow(self: Self, row: usize) usize {
        var sum: usize = 0;
        var col: usize = 0;
        while (col < col_count) : (col += 1) {
            if (self.isBug(row, col)) {
                sum += 1;
            }
        }
        return sum;
    }

    pub fn countBugsCol(self: Self, col: usize) usize {
        var sum: usize = 0;
        var row: usize = 0;
        while (row < row_count) : (row += 1) {
            if (self.isBug(row, col)) {
                sum += 1;
            }
        }
        return sum;
    }

    pub fn prettyPrint(self: Self) !void {
        const stdout = std.io.getStdOut().writer();

        var i: usize = 0;
        while (i < row_count) : (i += 1) {
            var j: usize = 0;
            while (j < col_count) : (j += 1) {
                const b: u8 = if (self.isBug(i, j)) '#' else '.';
                try stdout.writeByte(b);
            }
            try stdout.writeByte('\n');
        }
    }
};

pub inline fn pos(row: usize, col: usize) usize {
    return row * @as(usize, col_count) + col;
}
