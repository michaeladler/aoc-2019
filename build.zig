// generated using `zig init-exe`

const std = @import("std");
const Builder = std.build.Builder;

const days = [_][]const u8{
    "day22",
    "day23",
    "day24",
    "day25",
};

pub fn build(b: *Builder) !void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const allocator = b.allocator;

    const test_step = b.step("test", "Run unit tests");

    // my packages
    const pkg_numtheory = b.createModule(.{
        .source_file = .{ .path = "share/zig/numtheory.zig" },
    });
    const pkg_combination = b.createModule(.{
        .source_file = .{ .path = "share/zig/combination.zig" },
    });
    const pkg_intcode = b.createModule(.{
        .source_file = .{ .path = "share/zig/intcode.zig" },
    });

    // add intcode
    const intcode_tests = b.addTest(.{
        .root_source_file = .{ .path = "share/zig/intcode.zig" },
        .target = target,
        .optimize = optimize,
    });
    const run_intcode_tests = b.addRunArtifact(intcode_tests);
    test_step.dependOn(&run_intcode_tests.step);

    for (days) |d| {
        const main_zig = try std.fmt.allocPrint(allocator, "{s}/src/main.zig", .{d});

        const exe = b.addExecutable(.{
            .name = d,
            .root_source_file = .{ .path = main_zig },
            .target = target,
            .optimize = optimize,
        });
        exe.addModule("numtheory", pkg_numtheory);
        exe.addModule("combination", pkg_combination);
        exe.addModule("intcode", pkg_intcode);
        b.installArtifact(exe);

        const run_cmd = b.addRunArtifact(exe);
        run_cmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            run_cmd.addArgs(args);
        }

        const run_step = b.step(d, d);
        run_step.dependOn(&run_cmd.step);

        var tests = b.addTest(.{
            .root_source_file = .{ .path = main_zig },
            .target = target,
            .optimize = optimize,
        });
        tests.addModule("numtheory", pkg_numtheory);
        tests.addModule("combination", pkg_combination);
        tests.addModule("intcode", pkg_intcode);
        const run_tests = b.addRunArtifact(tests);

        test_step.dependOn(&run_tests.step);
    }
}
