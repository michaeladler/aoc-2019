const std = @import("std");
const time = std.time;
const Timer = std.time.Timer;

const solve = @import("solve.zig");

pub const log_level: std.log.Level = .info;

pub fn main() anyerror!void {
    const stdout = std.io.getStdOut().writer();

    var timer = try Timer.start();

    const answer1 = try solve.part1();
    var millis = @intToFloat(f64, timer.lap()) / @intToFloat(f64, time.ns_per_ms);
    try stdout.print("Part 1 (solved in {d}ms): {d}\n", .{ millis, answer1 });

    const answer2 = try solve.part2();
    millis = @intToFloat(f64, timer.lap()) / @intToFloat(f64, time.ns_per_ms);
    try stdout.print("Part 2 (solved in {d}ms): {d}\n", .{ millis, answer2 });
}

test "all" {
    _ = solve;
}
