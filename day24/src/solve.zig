const std = @import("std");
const debug = std.debug;
const mem = std.mem;
const log = std.log;
const testing = std.testing;
const Allocator = mem.Allocator;
const AutoHashMap = std.AutoHashMap;
const StaticBitSet = std.StaticBitSet;

const readInput = @import("parser.zig").readInput;
const grid = @import("grid.zig");
const Grid = grid.Grid;

const Empty = struct {};

pub fn part1() !usize {
    var fixed_buffer_mem: [10 * 1024]u8 = undefined;
    var fixed_allocator = std.heap.FixedBufferAllocator.init(fixed_buffer_mem[0..]);

    var old: Grid = readInput();

    var seen = AutoHashMap(Grid, Empty).init(fixed_allocator.allocator());
    try seen.put(old, Empty{});

    while (true) {
        var new = old;

        var i: usize = 0;
        while (i < grid.row_count) : (i += 1) {
            var j: usize = 0;
            while (j < grid.col_count) : (j += 1) {
                const neighbor_bugs = countNeighborBugs(old, i, j);
                if (new.isBug(i, j)) {
                    // A bug *dies* (becoming an empty space) unless there is *exactly one* bug adjacent to it.
                    if (neighbor_bugs != 1) {
                        new.clearBug(i, j);
                    }
                } else {
                    // An empty space *becomes infested* with a bug if *exactly one or two* bugs are adjacent to it.
                    if (neighbor_bugs == 1 or neighbor_bugs == 2) {
                        new.setBug(i, j);
                    }
                }
            }
        }

        const gop = try seen.getOrPut(new);
        if (gop.found_existing) {
            // watch for the first time a layout of bugs and empty spaces *matches any previous layout*
            return biodiversityRating(new);
        }
        gop.value_ptr.* = Empty{};
        old = new;
    }
    unreachable;
}

inline fn countNeighborBugs(g: Grid, row: usize, col: usize) usize {
    var count: usize = 0;
    // Tiles on the edges of the grid have fewer than four adjacent tiles; the missing tiles count as empty space.

    // east
    if (col + 1 < grid.col_count and g.isBug(row, col + 1)) {
        count += 1;
    }

    // south
    if (row + 1 < grid.row_count and g.isBug(row + 1, col)) {
        count += 1;
    }

    // west
    if (col > 0 and g.isBug(row, col - 1)) {
        count += 1;
    }

    // north
    if (row > 0 and g.isBug(row - 1, col)) {
        count += 1;
    }

    return count;
}

inline fn biodiversityRating(g: Grid) u64 {
    var sum: u64 = 0;
    var i: u6 = 0;
    while (i < @as(u6, @intCast(grid.row_count * grid.col_count))) : (i += 1) {
        if (g.bs.isSet(i)) {
            sum += @as(u64, 1) << i;
        }
    }
    return sum;
}

pub fn part2() !usize {
    const maxRounds = 200;

    // in each round, we can grow by at most one dimension in both directions because new Grids can only arise from Grids with bugs
    var grids_alpha: [2 * maxRounds]Grid = undefined;
    var grids_beta: [2 * maxRounds]Grid = undefined;
    const base_level = maxRounds; // level 0

    // if true, then grids_alpha contains the most recent version of all grids
    var active_alpha = true;

    var min_level: usize = base_level; // the minimum level at which bugs exist
    var max_level: usize = base_level; // the maximum level at which bugs exist

    {
        // init grids
        var i: usize = 0;
        while (i < grids_alpha.len) : (i += 1) {
            // Initially, no other levels contain bugs.
            grids_alpha[i] = Grid.init();
        }
        grids_alpha[base_level] = readInput();
    }

    var round: usize = 1;
    while (round <= @as(usize, maxRounds)) : (round += 1) {
        log.debug("round {d} starts. min_level: {d}, max_level: {d}", .{ round, min_level, max_level });

        var active_grids: []Grid = undefined;
        var inactive_grids: []Grid = undefined;
        if (active_alpha) {
            active_grids = grids_alpha[0..];
            inactive_grids = grids_beta[0..];
        } else {
            active_grids = grids_beta[0..];
            inactive_grids = grids_alpha[0..];
        }

        // create copy
        // TODO: only copy from min to max
        mem.copy(Grid, inactive_grids, active_grids);

        const new_min_level = if (min_level > 0) min_level - 1 else 0;
        const new_max_level = @min(max_level + 1, active_grids.len - 1);

        var lvl: usize = new_min_level;
        while (lvl <= new_max_level) : (lvl += 1) {
            // process each grid in each dimension
            var i: usize = 0;
            while (i < grid.row_count) : (i += 1) {
                var j: usize = 0;
                while (j < grid.col_count) : (j += 1) {
                    if (i == grid.center and j == grid.center) {
                        continue;
                    }
                    const neighbor_bugs = try countNeighborBugsRec(active_grids, lvl, i, j);
                    log.debug("layer {d}: ({d}, {d}) has {d} neighbors", .{ @as(i64, @intCast(lvl)) - @as(i64, @intCast(base_level)), i, j, neighbor_bugs });

                    if (active_grids[lvl].isBug(i, j)) {
                        // A bug *dies* (becoming an empty space) unless there is *exactly one* bug adjacent to it.
                        if (neighbor_bugs != 1) {
                            log.debug("lvl {d}: bug at ({d}, {d}) dies", .{ lvl, i, j });
                            inactive_grids[lvl].clearBug(i, j);
                        }
                    } else {
                        // An empty space *becomes infested* with a bug if *exactly one or two* bugs are adjacent to it.
                        if (neighbor_bugs == 1 or neighbor_bugs == 2) {
                            log.debug("lvl {d}: creating bug at ({d}, {d})", .{ lvl, i, j });
                            inactive_grids[lvl].setBug(i, j);
                        }
                    }
                }
                // The middle tile of your scan is empty to accommodate the recursive grids within it. I
                debug.assert(inactive_grids[lvl].isBug(grid.center, grid.center) == false);
            }
        }

        // update min_level
        if (inactive_grids[new_min_level].countBugs() > 0) {
            min_level = new_min_level;
        }
        // update max_level
        if (inactive_grids[new_max_level].countBugs() > 0) {
            max_level = new_max_level;
        }

        active_alpha = !active_alpha;
    }

    const active_grids = if (active_alpha) grids_alpha[0..] else grids_beta[0..];
    return countBugsRec(active_grids, base_level, min_level, max_level);
}

fn countNeighborBugsRec(grids: []const Grid, lvl: usize, row: usize, col: usize) !usize {
    const irow = @as(i64, @intCast(row));
    const icol = @as(i64, @intCast(col));

    const neighbors = [_][2]i64{
        [_]i64{ irow - 1, icol }, // north
        [_]i64{ irow, icol + 1 }, // east
        [_]i64{ irow + 1, icol }, // south
        [_]i64{ irow, icol - 1 }, // west
    };

    const center = grid.center;

    var sum: usize = 0;

    const myself = grids[lvl];
    log.debug("checking neighbors: lvl: {d}, row: {d}, col: {d}", .{ lvl, row, col });
    for (neighbors) |nb| {
        const nb_row = nb[0];
        const nb_col = nb[1];

        if (nb_row < 0) {
            // has neighbors in lvl-1
            if (lvl == 0) {
                return error.ParentMissing;
            }
            const parent_grid = grids[lvl - 1];
            if (parent_grid.isBug(center - 1, center)) {
                sum += 1;
            }
        } else if (nb_row >= grid.row_count) {
            // has neighbors in lvl-1
            if (lvl == 0) {
                return error.ParentMissing;
            }
            const parent_grid = grids[lvl - 1];
            if (parent_grid.isBug(center + 1, center)) {
                sum += 1;
            }
        } else if (nb_col < 0) {
            // has neighbors in lvl-1
            if (lvl == 0) {
                return error.ParentMissing;
            }
            const parent_grid = grids[lvl - 1];
            if (parent_grid.isBug(center, center - 1)) {
                sum += 1;
            }
        } else if (nb_col >= grid.col_count) {
            // has neighbors in lvl-1
            if (lvl == 0) {
                return error.ParentMissing;
            }
            const parent_grid = grids[lvl - 1];
            if (parent_grid.isBug(center, center + 1)) {
                sum += 1;
            }
        } else if (nb_row == center and nb_col == center) {
            // has neighbors in lvl+1
            if (lvl + 1 >= grids.len) {
                return error.ChildMissing;
            }
            const child_grid = grids[lvl + 1];
            // which row or column do we have to check in child grid?
            if (row == center - 1 and col == center) {
                log.debug("lvl {d} checking child grid north row", .{lvl});
                sum += child_grid.countBugsRow(0);
            } else if (row == center + 1 and col == center) {
                log.debug("lvl {d} checking child grid south row", .{lvl});
                sum += child_grid.countBugsRow(grid.row_count - 1);
            } else if (row == center and col == center - 1) {
                log.debug("lvl {d} checking child grid west col", .{lvl});
                sum += child_grid.countBugsCol(0);
            } else if (row == center and col == center + 1) {
                log.debug("lvl {d} checking child grid east col", .{lvl});
                sum += child_grid.countBugsCol(grid.col_count - 1);
            }
        } else {
            // neighbor is in our grid
            if (myself.isBug(@as(usize, @intCast(nb_row)), @as(usize, @intCast(nb_col)))) {
                sum += 1;
            }
        }
    }

    return sum;
}

fn countBugsRec(grids: []const Grid, center: usize, min_level: usize, max_level: usize) usize {
    var sum: usize = 0;
    // count bugs
    var i: usize = min_level;
    while (i <= max_level) : (i += 1) {
        const bugs = grids[i].countBugs();
        const level: i64 = @as(i64, @intCast(i)) - @as(i64, @intCast(center));
        log.debug("level {d} has bugs: {d}", .{ level, bugs });
        sum += bugs;
    }
    return sum;
}

test "2019 Day 24, Part 1" {
    const answer = try part1();
    try testing.expectEqual(@as(usize, 19923473), answer);
}

test "2019 Day 24, Part 2" {
    const answer = try part2();
    try testing.expectEqual(@as(usize, 1902), answer);
}

test "countNeighborBugsRec" {
    // ....#
    // #..#.
    // #..##
    // ..#..
    // #....
    var g = Grid.init();
    g.setBug(0, 4);
    g.setBug(1, 0);
    g.setBug(1, 3);
    g.setBug(2, 0);
    g.setBug(2, 3);
    g.setBug(2, 4);
    g.setBug(3, 2);
    g.setBug(4, 0);

    const grids = [_]Grid{
        Grid.init(),
        Grid.init(),
        g,
        Grid.init(),
        Grid.init(),
    };
    const center = 2;

    const test_cases = [_][4]usize{
        // lvl, row, col, expected
        [_]usize{ center - 1, 1, 2, 1 },
        [_]usize{ center - 1, 2, 1, 3 },
        [_]usize{ center - 1, 3, 2, 1 },

        [_]usize{ center, 0, 0, 1 },
        [_]usize{ center, 0, 3, 2 },
        [_]usize{ center, 1, 0, 1 },
        [_]usize{ center, 1, 1, 1 },
        [_]usize{ center, 1, 2, 1 },
        [_]usize{ center, 1, 3, 1 },
        [_]usize{ center, 1, 4, 3 },
        [_]usize{ center, 2, 0, 1 },
        [_]usize{ center, 2, 1, 1 },
        [_]usize{ center, 2, 3, 2 },
        [_]usize{ center, 2, 4, 1 },
        [_]usize{ center, 3, 0, 2 },
        [_]usize{ center, 3, 1, 1 },
        [_]usize{ center, 3, 3, 2 },
        [_]usize{ center, 3, 4, 1 },
        [_]usize{ center, 4, 1, 1 },
        [_]usize{ center, 4, 2, 1 },

        [_]usize{ center + 1, 0, 4, 1 },
        [_]usize{ center + 1, 1, 4, 1 },
        [_]usize{ center + 1, 2, 4, 1 },
        [_]usize{ center + 1, 3, 4, 1 },
        [_]usize{ center + 1, 4, 0, 1 },
        [_]usize{ center + 1, 4, 1, 1 },
        [_]usize{ center + 1, 4, 2, 1 },
        [_]usize{ center + 1, 4, 3, 1 },
        [_]usize{ center + 1, 4, 4, 2 },
    };

    for (test_cases) |tc| {
        log.debug("tc {d}", .{tc});
        try testing.expectEqual(tc[3], try countNeighborBugsRec(grids[0..], tc[0], tc[1], tc[2]));
    }
}
