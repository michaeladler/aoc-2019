const std = @import("std");
const debug = std.debug;
const mem = std.mem;
const log = std.log;
const testing = std.testing;
const Allocator = mem.Allocator;
const AutoHashMap = std.AutoHashMap;
const TailQueue = std.TailQueue;
const ArrayList = std.ArrayList;

const intcode = @import("intcode");
const IntcodeProgram = intcode.IntcodeProgram;

const Address = u16;

const Packet = struct { src: Address, dest: Address, x: i64, y: i64 };

const MessageQueue = ArrayList(i64);

const file_content = @embedFile("input.txt");

pub const NIC = struct {
    const Self = @This();

    allocator: Allocator,
    address: Address,
    nic: IntcodeProgram,
    incoming: MessageQueue,
    outgoing: ArrayList(Packet),
    buf: ArrayList(i64),

    pub fn init(allocator: Allocator, code: []const i64, address: Address) !Self {
        var self = Self{
            .allocator = allocator,
            .address = address,
            .nic = try IntcodeProgram.init(allocator, code),
            .incoming = try MessageQueue.initCapacity(allocator, 16),
            .outgoing = try ArrayList(Packet).initCapacity(allocator, 6),
            .buf = try ArrayList(i64).initCapacity(allocator, 18),
        };

        // when each computer boots up, it will request its network address via a single input instruction
        self.incoming.appendAssumeCapacity(address);

        return self;
    }

    pub fn deinit(self: *Self) void {
        self.nic.deinit();
        self.incoming.deinit();
        self.outgoing.deinit();
        self.buf.deinit();
    }

    const no_data = [_]i64{-1};

    /// Result can be picked up in self.outgoing.
    pub fn run(self: *Self) !void {
        // reset lists for outgoing messages
        self.outgoing.items.len = 0;
        self.buf.items.len = 0;

        const input = blk: {
            if (self.incoming.items.len > 0) {
                break :blk self.incoming.items;
            } else {
                // If the incoming packet queue is *empty*, provide `-1`
                break :blk no_data[0..];
            }
        };
        log.debug("NIC {d} starts running with input {d}", .{ self.address, input });
        const status = try self.nic.run(i64, input, i64, &self.buf);
        debug.assert(status == .blocked);
        log.debug("NIC {d} finished with output {d}", .{ self.address, self.buf.items });

        // reset incoming
        self.incoming.items.len = 0;

        // three i64 (addr, x, y) fit into a single packet
        try self.outgoing.ensureTotalCapacity(self.buf.items.len / 3);

        var i: usize = 0;
        const src = @as(u16, @intCast(self.address));
        while (i < self.buf.items.len) : (i += 3) {
            const packet = Packet{
                .src = src,
                .dest = @as(u16, @intCast(self.buf.items[i])),
                .x = self.buf.items[i + 1],
                .y = self.buf.items[i + 2],
            };
            self.outgoing.appendAssumeCapacity(packet);
        }
        log.debug("> NIC {d} sending: {any}", .{ self.address, self.outgoing.items });
    }

    pub fn enqueue(self: *Self, packet: Packet) !void {
        if (packet.dest != self.address) {
            return error.PacketMismatch;
        }
        log.debug("> NIC {d}: adding {any} to queue", .{ self.address, packet });
        try self.incoming.append(packet.x);
        try self.incoming.append(packet.y);
    }
};

pub fn part1(allocator: Allocator) !i64 {
    var arena = std.heap.ArenaAllocator.init(allocator);
    defer arena.deinit();

    var input = try intcode.parseInput(arena.allocator(), file_content);
    var network = AutoHashMap(Address, NIC).init(arena.allocator());

    const computerCount = 50;
    var i: u16 = 0;
    while (i < computerCount) : (i += 1) {
        try network.put(i, try NIC.init(arena.allocator(), input, i));
    }

    while (true) {
        var it = network.iterator();
        while (it.next()) |kv| {
            var nic = kv.value_ptr;
            try nic.run();

            for (nic.outgoing.items) |packet| {
                if (packet.dest == 255) {
                    return packet.y;
                }
                if (network.getEntry(packet.dest)) |entry| {
                    try entry.value_ptr.enqueue(packet);
                }
            }
        }
    }
    unreachable;
}

pub fn part2(allocator: Allocator) !i64 {
    var arena = std.heap.ArenaAllocator.init(allocator);
    defer arena.deinit();

    var input = try intcode.parseInput(arena.allocator(), file_content);
    var network = AutoHashMap(Address, NIC).init(arena.allocator());

    const computerCount = 50;
    var i: u16 = 0;
    while (i < computerCount) : (i += 1) {
        try network.put(i, try NIC.init(arena.allocator(), input, i));
    }

    // addr 255
    var nat: ?Packet = null;
    var last_packet_for_0: ?Packet = null;

    var is_idle: bool = undefined;
    var idle_counter: usize = 0;
    while (true) {
        is_idle = true;
        var it = network.iterator();
        while (it.next()) |kv| {
            var nic = kv.value_ptr;
            try nic.run();

            if (nic.outgoing.items.len > 0) {
                is_idle = false;
            }
            for (nic.outgoing.items) |packet| {
                if (packet.dest == 255) {
                    nat = packet;
                }
                if (network.getEntry(packet.dest)) |entry| {
                    try entry.value_ptr.enqueue(packet);
                }
            }
        }
        if (is_idle) {
            idle_counter += 1;
            log.debug("network is idle for {d} rounds. nat: {?}", .{ idle_counter, nat });
        }
        if (idle_counter > 0) {
            if (nat) |packet| {
                log.debug("NAT delivering packet to 0: {any}", .{packet});
                var packetCopy = packet;
                // do the NAT'ing
                packetCopy.src = 255;
                packetCopy.dest = 0;
                if (last_packet_for_0) |pk_for_0| {
                    if (pk_for_0.x == packetCopy.x and pk_for_0.y == packetCopy.y) {
                        return packetCopy.y;
                    }
                }
                try network.getEntry(0).?.value_ptr.enqueue(packetCopy);
                last_packet_for_0 = packetCopy;
                nat = null;
                idle_counter = 0;
            }
        }
    }
    unreachable;
}

test "2019 Day 23, Part 1" {
    const answer = try part1(testing.allocator);
    try testing.expectEqual(@as(i64, 15662), answer);
}

test "2019 Day 23, Part 2" {
    const answer = try part2(testing.allocator);
    try testing.expectEqual(@as(i64, 10854), answer);
}
