const std = @import("std");
const mem = std.mem;
const math = std.math;
const fmt = std.fmt;
const log = std.log.scoped(.intcode);

const Allocator = mem.Allocator;
const AutoHashMap = std.AutoHashMap;
const ArrayList = std.ArrayList;
const TailQueue = std.TailQueue;

const OpcodeTag = enum(u7) {
    add = 1,
    mul = 2,
    get = 3,
    put = 4,
    je = 5,
    jne = 6,
    lt = 7,
    eq = 8,
    base = 9,
    halt = 99,
};

pub const ParamMode = enum(u2) {
    /// causes the parameter to be interpreted as a position - if the parameter
    /// is 50, its value is the value stored at address 50 in memory
    position = 0,
    /// in immediate mode, a parameter is interpreted as a value - if the
    /// parameter is 50, its value is simply 50.
    immediate = 1,
    /// The address a relative mode parameter refers to is itself _plus_ the
    /// current _relative base_. When the relative base is `0`, relative mode
    /// parameters and position mode parameters with the same value refer to
    /// the same address.
    relative = 2,
};

const Args2 = struct { first: ParamMode, second: ParamMode };

const Args3 = struct { first: ParamMode, second: ParamMode, third: ParamMode };

pub const Instruction = union(OpcodeTag) {
    add: Args3,
    mul: Args3,
    get: ParamMode,
    put: ParamMode,
    je: Args2,
    jne: Args2,
    lt: Args3,
    eq: Args3,
    base: ParamMode,
    halt: void,

    /// Parse a single instruction
    pub fn fromValue(value: i64) Instruction {
        // the opcode is the rightmost two digits
        const opcode = @as(u7, @intCast(@mod(value, 100)));

        var rem = @divExact(value - opcode, 100);

        // Parameter modes are single digits, _one per parameter_ read right-to-left from the opcode:
        // - the first parameter's mode is in the hundreds digit,
        // - the second parameter's mode is in the thousands digit,
        // - the third parameter's mode is in the ten-thousands digit,
        // and so on.
        var modes = [_]ParamMode{ .position, .position, .position };
        var i: usize = 0;
        while (i < 3) : (i += 1) {
            const digit = @mod(rem, 10);
            rem = @divFloor(rem - digit, 10);
            modes[i] = parseMode(digit);
            if (rem == 0) {
                break;
            }
        }

        const result = blk: {
            switch (@as(OpcodeTag, @enumFromInt(opcode))) {
                .add => {
                    const val = Instruction{ .add = Args3{ .first = modes[0], .second = modes[1], .third = modes[2] } };
                    break :blk val;
                },
                .mul => {
                    const val = Instruction{ .mul = Args3{ .first = modes[0], .second = modes[1], .third = modes[2] } };
                    break :blk val;
                },
                .get => {
                    const val = Instruction{ .get = modes[0] };
                    break :blk val;
                },
                .put => {
                    const val = Instruction{ .put = modes[0] };
                    break :blk val;
                },
                .je => {
                    const val = Instruction{ .je = Args2{ .first = modes[0], .second = modes[1] } };
                    break :blk val;
                },
                .jne => {
                    const val = Instruction{ .jne = Args2{ .first = modes[0], .second = modes[1] } };
                    break :blk val;
                },
                .lt => {
                    const val = Instruction{ .lt = Args3{ .first = modes[0], .second = modes[1], .third = modes[2] } };
                    break :blk val;
                },
                .eq => {
                    const val = Instruction{ .eq = Args3{ .first = modes[0], .second = modes[1], .third = modes[2] } };
                    break :blk val;
                },
                .base => {
                    const val = Instruction{ .base = modes[0] };
                    break :blk val;
                },
                .halt => {
                    break :blk Instruction{ .halt = {} };
                },
            }
        };
        return result;
    }

    inline fn parseMode(num: i64) ParamMode {
        return @as(ParamMode, @enumFromInt(@as(u2, @intCast(num))));
    }
};

pub const IntcodeProgram = struct {
    const Self = @This();

    allocator: Allocator,

    code: []i64,

    // context
    ip: usize,
    base: i64,
    // state
    ram: AutoHashMap(u64, i64),

    pub const Status = enum(u2) {
        /// Program needs more numbers to continue (number source "exhausted")
        blocked,
        /// Program terminated
        terminated,
        /// EOF
        eof,
    };

    pub fn init(allocator: Allocator, code: []const i64) !Self {
        const ram = AutoHashMap(u64, i64).init(allocator);
        var code_copy = try allocator.alloc(i64, code.len);
        mem.copy(i64, code_copy, code);
        return Self{
            .allocator = allocator,
            .code = code_copy,
            .ip = 0,
            .base = 0,
            .ram = ram,
        };
    }

    pub fn deinit(self: *Self) void {
        self.ram.deinit();
        self.allocator.free(self.code);
    }

    // Clone creates an identical copy of the current IntcodeProgram.
    pub fn clone(self: Self, allocator: Allocator) !Self {
        var copy = try Self.init(allocator, self.code);
        copy.ip = self.ip;
        copy.base = self.base;
        var it = self.ram.iterator();
        try copy.ram.ensureTotalCapacity(self.ram.count());
        while (it.next()) |entry| {
            copy.ram.putAssumeCapacity(entry.key_ptr.*, entry.value_ptr.*);
        }
        return copy;
    }

    // Caller owns the result and must call deinit() to free memory.
    pub fn run(self: *Self, comptime I: type, input: []const I, comptime O: type, output: *ArrayList(O)) Allocator.Error!Status {
        const input_len = input.len;
        var input_idx: usize = 0;
        const n = self.code.len;
        while (self.ip < n) {
            // needed to restore self.ip in certain cases
            const ip = self.ip;
            const instruction = self.readInstruction();
            switch (instruction) {
                .add => {
                    const modes = instruction.add;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    const out_pos = self.readOutPos(modes.third);
                    log.debug("[Add] a={d}, b={d}, out_pos={d}", .{ a, b, out_pos });
                    try self.writeValue(out_pos, a + b);
                },
                .mul => {
                    const modes = instruction.mul;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    const out_pos = self.readOutPos(modes.third);
                    log.debug("[Mul] a={d}, b={d}, out_pos={d}", .{ a, b, out_pos });
                    try self.writeValue(out_pos, a * b);
                },
                .get => {
                    if (input_idx >= input_len) {
                        log.debug("[Get] Need more numbers to continue", .{});
                        self.ip = ip; // restore old ip
                        return Status.blocked;
                    }
                    const data = @as(I, @intCast(input[input_idx]));
                    input_idx += 1;
                    const out_pos = self.readOutPos(instruction.get);
                    log.debug("[Get] Storing {d} it in position out_pos={d}", .{ data, out_pos });
                    try self.writeValue(out_pos, data);
                },
                .put => {
                    const a = self.readParam(instruction.put);
                    log.debug("[Put] Appending {d} to output", .{a});
                    try output.append(@as(O, @intCast(a)));
                },
                .je => {
                    const modes = instruction.je;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    if (a != 0) {
                        // jump
                        log.debug("[JUMP-IF-TRUE] a={d}, b={d} => jumping", .{ a, b });
                        self.ip = @as(usize, @intCast(b));
                    } else {
                        log.debug("[JUMP-IF-TRUE] a={d}, b={d} => no jump", .{ a, b });
                    }
                },
                .jne => {
                    const modes = instruction.jne;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    if (a == 0) {
                        log.debug("[JUMP-IF-FALSE] a={d}, b={d} => jump", .{ a, b });
                        self.ip = @as(usize, @intCast(b));
                    } else {
                        log.debug("[JUMP-IF-FALSE] a={d}, b={d} => no jump", .{ a, b });
                    }
                },
                .lt => {
                    const modes = instruction.lt;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    const out_pos = self.readOutPos(modes.third);
                    log.debug("[LT] Checking if {d} < {d} and storing result in out_pos {d}", .{ a, b, out_pos });
                    const val: i64 = if (a < b) 1 else 0;
                    try self.writeValue(out_pos, val);
                },
                .eq => {
                    const modes = instruction.eq;
                    const a = self.readParam(modes.first);
                    const b = self.readParam(modes.second);
                    const out_pos = self.readOutPos(modes.third);
                    log.debug("[EQ] Checking if {d} == {d} and storing result in out_pos {d}", .{ a, b, out_pos });
                    const val: i64 = if (a == b) 1 else 0;
                    try self.writeValue(out_pos, val);
                },
                .base => {
                    const a = self.readParam(instruction.base);
                    const new_base = self.base + a;
                    log.debug("[BASE] Adjusting base: {d} -> {d}", .{ self.base, new_base });
                    self.base = new_base;
                },
                .halt => {
                    log.debug("[HALT]", .{});
                    return Status.terminated;
                },
            }
            log.debug("ip: {d} -> {d}, base: {d}", .{ ip, self.ip, self.base });
        }
        // program did not terminate with a halt instruction
        return Status.eof;
    }

    inline fn readInstruction(self: *Self) Instruction {
        const instruction = Instruction.fromValue(self.code[self.ip]);
        self.ip += 1;
        return instruction;
    }

    fn readParam(self: *Self, mode: ParamMode) i64 {
        const val = self.code[self.ip];
        self.ip += 1;
        switch (mode) {
            .position => {
                return self.readValue(@as(u64, @intCast(val)));
            },
            .immediate => {
                return val;
            },
            .relative => {
                return self.readValue(@as(u64, @intCast(self.base + val)));
            },
        }
    }

    fn readOutPos(self: *Self, mode: ParamMode) u64 {
        const val = self.code[self.ip];
        self.ip += 1;
        switch (mode) {
            ParamMode.relative => {
                return @as(u64, @intCast(self.base + val));
            },
            else => {
                return @as(u64, @intCast(val));
            },
        }
    }

    fn readValue(self: Self, addr: u64) i64 {
        const value = blk: {
            if (addr < self.code.len) {
                const val = self.code[@as(usize, @intCast(addr))];
                log.debug("reading addr {d} from code: {d}", .{ addr, val });
                break :blk val;
            } else if (self.ram.get(addr)) |val| {
                log.debug("reading addr {d} from ram: {d}", .{ addr, val });
                break :blk val;
            } else {
                const val = 0;
                log.debug("no value for addr {d}, returning default: {d}", .{ addr, val });
                break :blk 0;
            }
        };
        return value;
    }

    fn writeValue(self: *Self, addr: u64, value: i64) !void {
        if (addr < self.code.len) {
            log.debug("writing {d} to code at addr {d}", .{ value, addr });
            self.code[@as(usize, @intCast(addr))] = value;
        } else {
            log.debug("writing {d} to RAM at addr {d}", .{ value, addr });
            try self.ram.put(addr, value);
        }
    }
};

/// caller owns result and must call `deinit`
pub fn parseInput(allocator: Allocator, file_content: []const u8) ![]i64 {
    var list = ArrayList(i64).init(allocator);
    var it = mem.tokenize(u8, file_content, ",");
    while (it.next()) |val| {
        const stripped = mem.trimRight(u8, val, " \n");
        const num = try fmt.parseInt(i64, stripped, 10);
        try list.append(num);
    }
    return list.toOwnedSlice();
}

test {
    _ = @import("intcode_test.zig");
}
