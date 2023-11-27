const std = @import("std");
const debug = std.debug;
const log = std.log;
const mem = std.mem;
const math = std.math;
const Allocator = mem.Allocator;
const TailQueue = std.TailQueue;
const ArrayList = std.ArrayList;

const Instruction = @import("instruction.zig").Instruction;

const Orientation = enum(u1) {
    left_to_right,
    right_to_left,

    const Self = @This();
    pub fn flip(self: Self) Self {
        return switch (self) {
            .left_to_right => Orientation.right_to_left,
            .right_to_left => Orientation.left_to_right,
        };
    }
};

/// Naive approach. Does not scale, so it cannot be used for part 2. Only used for testing in the end.
pub const Deck = struct {
    const Self = @This();
    /// The type of a single card.
    const Node = TailQueue(u16).Node;

    /// All cards.
    cards: TailQueue(u16),
    /// Backing store for cards; internal only. `cards` has pointers to `nodes`.
    nodes: ArrayList(Node),
    /// If we traverse the cards, do we go left->right or right->left.
    orientation: Orientation,

    /// we need to some allocations as well
    allocator: Allocator,

    pub fn init(allocator: Allocator, count: usize) !Self {
        var self = Self{
            .cards = TailQueue(u16){},
            .nodes = undefined,
            .orientation = Orientation.left_to_right,
            .allocator = allocator,
        };
        {
            var nodes = try ArrayList(Node).initCapacity(allocator, count);
            self.nodes = nodes;
        }

        var i: u16 = 0;
        while (i < count) : (i += 1) {
            self.nodes.appendAssumeCapacity(Node{ .data = i });
            // self.nodes must never resize
            self.cards.append(&self.nodes.items[self.nodes.items.len - 1]);
        }
        return self;
    }

    pub fn deinit(self: *Self) void {
        self.nodes.deinit();
    }

    pub const Iterator = struct {
        current: ?*Node,
        orientation: Orientation,

        pub fn next(it: *Iterator) ?u16 {
            if (it.current) |node| {
                it.current = switch (it.orientation) {
                    .left_to_right => node.next,
                    .right_to_left => node.prev,
                };
                return node.data;
            }
            return null;
        }
    };

    /// Return an iterator that walks the deck without consuming.
    pub fn iterator(self: Self) Iterator {
        return Iterator{
            .current = switch (self.orientation) {
                .left_to_right => self.cards.first,
                .right_to_left => self.cards.last,
            },
            .orientation = self.orientation,
        };
    }

    pub fn get_card(self: Self, pos: usize) ?usize {
        var it = self.iterator();
        var i: usize = 0;
        while (it.next()) |number| {
            if (i == pos) {
                return number;
            }
            i += 1;
        }
        return null;
    }

    /// Return the position of `needle` in the card stack.
    pub fn find_card(self: Self, needle: usize) ?usize {
        var it = self.iterator();
        var i: usize = 0;
        while (it.next()) |number| {
            if (number == needle) {
                return i;
            }
            i += 1;
        }
        return null;
    }

    /// To deal into new stack, create a new stack of cards by dealing the top
    /// card of the deck onto the top of the new stack repeatedly until you run
    /// out of cards.
    ///
    /// Complexity: O(1)
    pub fn deal_into_new(self: *Self) void {
        log.debug("deal into new stack", .{});
        self.orientation = self.orientation.flip();
    }

    /// To cut N cards, take the top N cards off the top of the deck and move
    /// them as a single unit to the bottom of the deck, retaining their order.
    ///
    /// Complexity: O(N)
    pub fn cut_n(self: *Self, n: i64) void {
        const pos_n = if (n >= 0) @as(usize, @intCast(n)) else @as(usize, @intCast(self.cards.len - @as(usize, @intCast((-1) * n))));
        log.debug("cut {d} -> cut {d}", .{ n, pos_n });
        debug.assert(pos_n < self.cards.len);
        // 0 1 2 3 4 5 6 7 8 9   Your deck
        //       3 4 5 6 7 8 9   Your deck
        // 0 1 2                 Cut cards
        // 3 4 5 6 7 8 9         Your deck
        //               0 1 2   Cut cards
        // 3 4 5 6 7 8 9 0 1 2   Your deck
        var it = self.iterator();
        var i: usize = 0;
        // go to node pos
        while (i < pos_n - 1) : (i += 1) {
            _ = it.next();
        }
        var node = it.current.?;
        switch (self.orientation) {
            .left_to_right => {
                self.cards.last.?.next = self.cards.first;
                self.cards.first.?.prev = self.cards.last;

                self.cards.first = node.next;
                self.cards.first.?.prev = null;

                self.cards.last = node;
                self.cards.last.?.next = null;
            },
            .right_to_left => {
                // same code as in left-to-right, but due to symmetry replace simultaneously:
                // * next <-> prev
                // * first <-> last
                self.cards.first.?.prev = self.cards.last;
                self.cards.last.?.next = self.cards.first;

                self.cards.last = node.prev;
                self.cards.last.?.next = null;

                self.cards.first = node;
                self.cards.first.?.prev = null;
            },
        }
    }

    /// To deal with increment N, start by clearing enough space on your table
    /// to lay out all of the cards individually in a long line. Deal the top
    /// card into the leftmost position. Then, move N positions to the right
    /// and deal the next card there. If you would move into a position past
    /// the end of the space on your table, wrap around and keep counting from
    /// the leftmost card again. Continue this process until you run out of
    /// cards.
    pub fn deal_with_increment(self: *Self, n: i64) !void {
        log.debug("deal with increment {d}", .{n});
        const count = self.cards.len;
        var current_nodes = try ArrayList(Node).initCapacity(self.allocator, count);
        defer current_nodes.deinit();
        {
            var it = self.iterator();
            // works for all orientations
            while (it.next()) |actual| {
                current_nodes.appendAssumeCapacity(Node{ .data = actual });
            }
        }

        var new_nodes = try ArrayList(Node).initCapacity(self.allocator, count);
        new_nodes.expandToCapacity();
        {
            // now do the actual work
            var src_idx: usize = 0;
            var target_idx: usize = 0;
            while (src_idx < count) : (src_idx += 1) {
                new_nodes.items[target_idx] = current_nodes.items[src_idx];
                target_idx = @as(usize, @intCast(@mod(@as(i64, @intCast(target_idx)) + n, @as(i64, @intCast(count)))));
            }
        }

        // just copy the references
        var new_cards = TailQueue(u16){};
        {
            var i: usize = 0;
            while (i < count) : (i += 1) {
                new_cards.append(&new_nodes.items[i]);
            }
        }

        // update
        self.cards = new_cards;
        self.nodes.deinit();
        self.nodes = new_nodes;
        self.orientation = Orientation.left_to_right;
    }

    pub fn apply_instructions(self: *Self, instructions: []const Instruction) !void {
        for (instructions) |item| {
            switch (item) {
                .deal_into => {
                    self.deal_into_new();
                },
                .cut_n => {
                    self.cut_n(item.cut_n);
                },
                .deal_inc => {
                    try self.deal_with_increment(item.deal_inc);
                },
            }
        }
    }
};

const testing = std.testing;
const expectEqual = testing.expectEqual;
const expectEqualSlices = testing.expectEqualSlices;

test "init a new non-empty Deck" {
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();
    try expectEqual(@as(usize, 10), deck.nodes.items.len);
    try expectEqual(@as(usize, 10), deck.cards.len);
    try expectEqual(@as(u16, 0), deck.cards.first.?.data);
    try expectEqual(@as(u16, 1), deck.cards.first.?.next.?.data);
    try expectEqual(@as(u16, 2), deck.cards.first.?.next.?.next.?.data);
    try expectEqual(@as(u16, 9), deck.cards.last.?.data);
}

test "iterator left to right" {
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();
    var it = deck.iterator();
    var card_number: u16 = 0;
    while (it.next()) |actual| {
        try expectEqual(card_number, actual);
        card_number += 1;
    }
    try expectEqual(deck.cards.len, card_number);
}

test "deal_into_new" {
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    deck.deal_into_new();
    {
        var it = deck.iterator();
        var card_number: u16 = 10;
        while (it.next()) |actual| {
            card_number -= 1;
            try expectEqual(card_number, actual);
        }
        try expectEqual(@as(u16, 0), card_number);
    }

    // test idempotence
    deck.deal_into_new();
    {
        var card_number: u16 = 0;
        var it = deck.iterator();
        while (it.next()) |actual| {
            try expectEqual(card_number, actual);
            card_number += 1;
        }
        try expectEqual(deck.cards.len, card_number);
    }
}

test "cut 3: left to right" {
    var expectedCards = [_]u16{ 3, 4, 5, 6, 7, 8, 9, 0, 1, 2 };
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    deck.cut_n(3);

    var actualCards: [10]u16 = undefined;
    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);

    // flip and check again
    deck.deal_into_new();
    mem.reverse(u16, expectedCards[0..]);

    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);
}

test "cut 3: right to left" {
    var expectedCards = [_]u16{ 3, 4, 5, 6, 7, 8, 9, 0, 1, 2 };
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    // reverse cards because we go from right to left
    {
        var it = deck.cards.first;
        while (it) |node| : (it = node.next) {
            node.data = 9 - node.data;
        }
    }
    deck.orientation = .right_to_left;

    deck.cut_n(3);

    var actualCards: [10]u16 = undefined;
    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);

    // flip and check again
    deck.deal_into_new();
    mem.reverse(u16, expectedCards[0..]);

    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);
}

test "cut -4: left to right" {
    var expectedCards = [_]u16{ 6, 7, 8, 9, 0, 1, 2, 3, 4, 5 };

    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    deck.cut_n(-4);

    var actualCards: [10]u16 = undefined;
    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);

    // flip and check again
    deck.deal_into_new();
    mem.reverse(u16, expectedCards[0..]);

    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);
}

test "cut -4: right to left" {
    var expectedCards = [_]u16{ 6, 7, 8, 9, 0, 1, 2, 3, 4, 5 };
    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    // reverse cards because we go from right to left
    {
        var it = deck.cards.first;
        while (it) |node| : (it = node.next) {
            node.data = 9 - node.data;
        }
    }
    deck.orientation = .right_to_left;

    deck.cut_n(-4);

    var actualCards: [10]u16 = undefined;
    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);

    // flip and check again
    deck.deal_into_new();
    mem.reverse(u16, expectedCards[0..]);

    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);
}

test "deal with increment 3: left to right" {
    var expectedCards = [_]u16{ 0, 7, 4, 1, 8, 5, 2, 9, 6, 3 };

    var deck = try Deck.init(testing.allocator, 10);
    defer deck.deinit();

    try deck.deal_with_increment(3);

    var actualCards: [10]u16 = undefined;
    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);

    // flip and check again
    deck.deal_into_new();
    mem.reverse(u16, expectedCards[0..]);

    {
        var it = deck.iterator();
        var i: usize = 0;
        while (it.next()) |actual| {
            actualCards[i] = actual;
            i += 1;
        }
    }
    try expectEqualSlices(u16, expectedCards[0..], actualCards[0..]);
}
