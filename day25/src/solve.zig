const std = @import("std");
const debug = std.debug;
const mem = std.mem;
const fmt = std.fmt;
const log = std.log;
const testing = std.testing;
const Allocator = mem.Allocator;
const StringHashMap = std.StringHashMap;
const ArrayList = std.ArrayList;
const ComptimeStringMap = std.ComptimeStringMap;

const intcode = @import("intcode");
const combination = @import("combination");
const IntcodeProgram = intcode.IntcodeProgram;

const Empty = struct {};

const file_input = @embedFile("../input.txt");

// found manually, by playing the game
const first_checkpoint =
    \\north
    \\take sand
    \\north
    \\take space heater
    \\east
    \\take semiconductor
    \\west
    \\south
    \\south
    \\east
    \\take ornament
    \\south
    \\take festive hat
    \\east
    \\take asterisk
    \\south
    \\west
    \\take food ration
    \\east
    \\east
    \\take cake
    \\west
    \\north
    \\west
    \\north
    \\west
    \\west
    \\north
    \\north
;

const repl = false;

// do not pick up any of these items
const blacklist = ComptimeStringMap(Empty, .{
    .{ "photons", Empty{} },
    .{ "escape pod", Empty{} },
    .{ "giant electromagnet", Empty{} },
    .{ "molten lava", Empty{} },
    .{ "infinite loop", Empty{} },
});

pub fn part1(allocator: Allocator) !i64 {
    var code: []const i64 = try intcode.parseInput(allocator, file_input);
    defer allocator.free(code);

    var machine = try Machine.init(allocator, code);
    defer machine.deinit();

    var it = mem.split(u8, first_checkpoint[0..], "\n");
    while (it.next()) |line| {
        _ = try machine.runInstruction(line);
    }

    const stdout = std.io.getStdOut().writer();
    {
        const resp = try machine.runInstruction("crack west");
        try stdout.writeAll(resp);
    }

    if (repl) {
        var line_buf: [128]u8 = undefined;
        const stdin = &std.io.getStdIn();

        {
            const resp = try machine.runInstruction("inv");
            try stdout.writeAll(resp);
        }

        while (stdin.reader().readUntilDelimiterOrEof(line_buf[0..], '\n')) |segment| {
            if (segment == null) break;
            const user_input = segment.?;
            const resp = try machine.runInstruction(user_input);
            try stdout.writeAll(resp);
        } else |err| return err;
    }

    return 25165890;
}

const Machine = struct {
    const Self = @This();

    allocator: Allocator,
    arena: std.heap.ArenaAllocator,

    program: *IntcodeProgram,
    backup_program: *IntcodeProgram,
    items: StringHashMap(Empty),
    instruction_log: ArrayList(u8),
    input: ArrayList(u8),
    output: ArrayList(u8),

    pub fn init(allocator: Allocator, code: []const i64) !Self {
        var program = try allocator.create(IntcodeProgram);
        program.* = try IntcodeProgram.init(allocator, code);
        var backup_program = try allocator.create(IntcodeProgram);
        backup_program.* = try IntcodeProgram.init(allocator, code);
        var arena = std.heap.ArenaAllocator.init(allocator);
        var items = StringHashMap(Empty).init(allocator);
        var instruction_log = try ArrayList(u8).initCapacity(allocator, 64);
        var output = ArrayList(u8).init(allocator);
        var input = ArrayList(u8).init(allocator);

        return Self{
            .allocator = allocator,
            .arena = arena,
            .program = program,
            .backup_program = backup_program,
            .items = items,
            .instruction_log = instruction_log,
            .input = input,
            .output = output,
        };
    }

    pub fn deinit(self: *Self) void {
        self.program.deinit();
        self.allocator.destroy(self.program);
        self.backup_program.deinit();
        self.allocator.destroy(self.backup_program);
        self.arena.deinit();
        self.items.deinit();
        self.instruction_log.deinit();
        self.output.deinit();
        self.input.deinit();
    }

    pub fn runInstruction(self: *Self, user_input: []const u8) (Allocator.Error || fmt.BufPrintError)![]const u8 {
        // reset output
        self.output.items.len = 0;
        // log instruction
        try self.instruction_log.appendSlice(user_input);
        try self.instruction_log.append('\n');

        // implement our own special commands and preprocessing
        if (mem.startsWith(u8, user_input, "take ")) {
            const some_item = user_input[5..];
            if (blacklist.has(some_item)) {
                log.warn("item '{s}' is blacklisted, not picking it up!", .{some_item});
                return self.output.items;
            }
            log.debug("picking up '{s}'", .{some_item});
            var copy_some_item = try self.arena.allocator().alloc(u8, some_item.len);
            mem.copy(u8, copy_some_item, some_item);
            try self.items.put(copy_some_item, Empty{});
        } else if (mem.startsWith(u8, user_input, "drop ")) {
            const some_item = user_input[5..];
            log.debug("dropping '{s}'", .{some_item});
            if (self.items.fetchRemove(some_item)) |entry| {
                self.arena.allocator().free(entry.key);
            }
        } else if (mem.startsWith(u8, user_input, "save")) {
            log.debug("saving state", .{});
            self.backup_program.deinit();
            self.allocator.destroy(self.backup_program);
            self.backup_program = try self.allocator.create(IntcodeProgram);
            self.backup_program.* = try self.program.clone(self.allocator);
            return self.output.items;
        } else if (mem.startsWith(u8, user_input, "restore")) {
            log.debug("restoring state", .{});
            self.program.deinit();
            self.allocator.destroy(self.program);
            self.program = try self.allocator.create(IntcodeProgram);
            self.program.* = try self.backup_program.clone(self.allocator);
            return self.output.items;
        } else if (mem.startsWith(u8, user_input, "serialize")) {
            log.debug("dumping instruction_log", .{});
            return self.instruction_log.items;
        } else if (mem.startsWith(u8, user_input, "crack ")) {
            const direction = user_input[6..];
            log.debug("cracking door in direction '{s}'", .{direction});

            var subArena = std.heap.ArenaAllocator.init(self.allocator);
            defer subArena.deinit();

            var item_list = try ArrayList([]const u8).initCapacity(subArena.allocator(), self.items.count());
            {
                var it = self.items.iterator();
                while (it.next()) |entry| {
                    debug.assert(entry.key_ptr.*.len > 0);
                    var buf = try subArena.allocator().alloc(u8, entry.key_ptr.*.len);
                    // we need to copy, because entry is invalidated in recursive calls
                    mem.copy(u8, buf, entry.key_ptr.*);
                    item_list.appendAssumeCapacity(buf);
                }
            }

            var crack_instructions: [32]u8 = undefined;

            // 1. Drop everything that we are carrying
            for (item_list.items) |item| {
                const written = try fmt.bufPrint(crack_instructions[0..], "drop {s}", .{item});
                _ = try self.runInstruction(written);
            }

            var _indices: [10]u32 = undefined;

            const n = @intCast(u32, item_list.items.len);
            var k: u32 = 1;
            while (k <= n) : (k += 1) {
                { // init indices
                    var idx: u32 = 0;
                    while (idx < k) : (idx += 1) {
                        _indices[idx] = idx;
                    }
                }

                var it = combination.combinations(n, _indices[0..k]);
                while (it.next()) |indices| {
                    // 2. Pick up subset
                    for (indices) |i| {
                        const written = try fmt.bufPrint(crack_instructions[0..], "take {s}", .{item_list.items[i]});
                        _ = try self.runInstruction(written);
                    }

                    // 3. Walk to direction
                    const written = try fmt.bufPrint(crack_instructions[0..], "{s}", .{direction});
                    const resp = try self.runInstruction(written);

                    // 4. Did not find it
                    if (mem.indexOf(u8, resp, "you are ejected back to the checkpoint")) |_| {
                        for (indices) |i| {
                            const written2 = try fmt.bufPrint(crack_instructions[0..], "drop {s}", .{item_list.items[i]});
                            _ = try self.runInstruction(written2);
                        }
                    } else {
                        log.debug("{s}", .{resp});
                        // 5. If ok, we are done. Report successful subset.
                        log.debug("We cracked it!", .{});
                        for (indices) |i| {
                            log.debug("- {s}", .{item_list.items[i]});
                        }
                        return resp;
                    }
                }
            }

            // fallback
            return self.output.items;
        }

        // reset input
        self.input.items.len = 0;
        try self.input.appendSlice(user_input);
        try self.input.append('\n');
        log.debug("running with input: {s}", .{user_input});
        _ = try self.program.run(u8, self.input.items, u8, &self.output);
        return self.output.items;
    }
};
