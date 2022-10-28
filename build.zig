// generated using `zig init-exe`

const std = @import("std");
const Builder = std.build.Builder;
const Pkg = std.build.Pkg;

const Day = struct {
    dir: []const u8,
    packages: ?[]const Pkg,
};

// my packages
const pkg_numtheory = Pkg{ .name = "numtheory", .source = .{ .path = "share/zig/numtheory.zig" } };
const pkg_combination = Pkg{ .name = "combination", .source = .{ .path = "share/zig/combination.zig" } };
const pkg_intcode = Pkg{ .name = "intcode", .source = .{ .path = "share/zig/intcode.zig" } };

// would be nice if this could be inlined
const arr_numtheory = [_]Pkg{pkg_numtheory};
const arr_intcode = [_]Pkg{pkg_intcode};
const arr_intcode_combination = [_]Pkg{ pkg_intcode, pkg_combination };

const days = [_]Day{
    .{
        .dir = "day22",
        .packages = arr_numtheory[0..],
    },
    .{
        .dir = "day23",
        .packages = arr_intcode[0..],
    },
    .{
        .dir = "day24",
        .packages = null,
    },
    .{
        .dir = "day25",
        .packages = arr_intcode_combination[0..],
    },
};

pub fn build(b: *Builder) !void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});

    // Standard release options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall.
    const mode = b.standardReleaseOptions();

    const allocator = b.allocator;

    const test_step = b.step("test", "Run library tests");

    // add intcode
    var intcode_tests = b.addTest("share/zig/intcode.zig");
    intcode_tests.setBuildMode(mode);
    test_step.dependOn(&intcode_tests.step);

    for (days) |d| {
        const main_zig = try std.fmt.allocPrint(allocator, "{s}/src/main.zig", .{d.dir});

        const exe = b.addExecutable(d.dir, main_zig);

        exe.setTarget(target);
        exe.setBuildMode(mode);
        exe.install();

        const run_cmd = exe.run();
        run_cmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            run_cmd.addArgs(args);
        }

        const run_step = b.step(d.dir, d.dir);
        run_step.dependOn(&run_cmd.step);

        var tests = b.addTest(main_zig);
        tests.setBuildMode(mode);

        test_step.dependOn(&tests.step);

        if (d.packages) |packages| {
            for (packages) |p| {
                exe.addPackage(p);
                tests.addPackage(p);
            }
        }
    }
}
