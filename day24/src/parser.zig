const std = @import("std");
const mem = std.mem;
const log = std.log;
const math = std.math;

const File = std.fs.File;
const ArrayList = std.ArrayList;
const Allocator = mem.Allocator;

const Grid = @import("grid.zig").Grid;

const bug: u8 = '#';
const empty: u8 = '.';

const input = @embedFile("../input.txt");

pub fn readInput() Grid {
    // the fact that this is possible is just awesome
    comptime var g = Grid.init();
    comptime {
        var it = mem.split(u8, input[0..], "\n");
        var row: usize = 0;
        while (it.next()) |line| {
            var col: usize = 0;
            while (col < line.len) : (col += 1) {
                if (line[col] == bug) {
                    g.setBug(row, col);
                }
            }
            row += 1;
        }
    }
    return g;
}
