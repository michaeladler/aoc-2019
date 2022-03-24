const std = @import("std");
const mem = std.mem;
const log = std.log;

const File = std.fs.File;
const ArrayList = std.ArrayList;
const Allocator = mem.Allocator;
const Instruction = @import("instruction.zig").Instruction;

/// caller owns result and must call `deinit`
pub fn readInput(allocator: Allocator) anyerror!ArrayList(Instruction) {
    const file_content = @embedFile("../input.txt");
    return parseInstructions(allocator, file_content);
}

pub fn parseInstructions(allocator: Allocator, content: []const u8) anyerror!ArrayList(Instruction) {
    var result = ArrayList(Instruction).init(allocator);
    var it = mem.tokenize(u8, content, "\n");
    while (it.next()) |line| {
        var it2 = mem.tokenize(u8, line, " ");
        // deal with increment 73
        // cut -6744
        // deal into new stack
        if (it2.next()) |first| {
            if (it2.next()) |second| {
                if (mem.eql(u8, first, "cut")) {
                    const n = try std.fmt.parseInt(i32, second, 10);
                    try result.append(Instruction{ .cut_n = n });
                } else if (mem.eql(u8, second, "into")) {
                    try result.append(Instruction.deal_into);
                } else {
                    _ = it2.next().?;
                    const inc = it2.next().?;
                    const n = try std.fmt.parseInt(u32, inc, 10);
                    try result.append(Instruction{ .deal_inc = n });
                }
            }
        }
    }
    log.debug("parsed {d} instructions", .{result.items.len});
    return result;
}
