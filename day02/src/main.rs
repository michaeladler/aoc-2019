fn run_program(input: &mut [i64]) {
    let n = input.len();
    let mut i = 0;
    'outer: while i < n {
        let op_code = input[i];
        match op_code {
            1 => {
                let in_pos1 = input[i + 1] as usize;
                let in_pos2 = input[i + 2] as usize;
                let out_pos = input[i + 3] as usize;
                input[out_pos] = input[in_pos1] + input[in_pos2];
            }
            2 => {
                let in_pos1 = input[i + 1] as usize;
                let in_pos2 = input[i + 2] as usize;
                let out_pos = input[i + 3] as usize;
                input[out_pos] = input[in_pos1] * input[in_pos2];
            }
            99 => {
                break 'outer;
            }
            _ => println!("Unsupported op code: {}", op_code),
        }
        i += 4;
    }
}

#[test]
fn test_run_program() {
    let mut prog = [1, 0, 0, 0, 99];
    run_program(&mut prog);
    assert_eq!(prog, [2, 0, 0, 0, 99]);

    let mut prog = [2, 3, 0, 3, 99];
    run_program(&mut prog);
    assert_eq!(prog, [2, 3, 0, 6, 99]);

    let mut prog = [2, 4, 4, 5, 99, 0];
    run_program(&mut prog);
    assert_eq!(prog, [2, 4, 4, 5, 99, 9801]);

    let mut prog = [1, 1, 1, 4, 99, 5, 6, 0, 99];
    run_program(&mut prog);
    assert_eq!(prog, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
}

fn main() {
    let input = [
        1, 0, 0, 3, 1, 1, 2, 3, 1, 3, 4, 3, 1, 5, 0, 3, 2, 13, 1, 19, 1, 10, 19, 23, 1, 23, 9, 27, 1, 5, 27, 31, 2, 31, 13, 35, 1, 35, 5,
        39, 1, 39, 5, 43, 2, 13, 43, 47, 2, 47, 10, 51, 1, 51, 6, 55, 2, 55, 9, 59, 1, 59, 5, 63, 1, 63, 13, 67, 2, 67, 6, 71, 1, 71, 5,
        75, 1, 75, 5, 79, 1, 79, 9, 83, 1, 10, 83, 87, 1, 87, 10, 91, 1, 91, 9, 95, 1, 10, 95, 99, 1, 10, 99, 103, 2, 103, 10, 107, 1, 107,
        9, 111, 2, 6, 111, 115, 1, 5, 115, 119, 2, 119, 13, 123, 1, 6, 123, 127, 2, 9, 127, 131, 1, 131, 5, 135, 1, 135, 13, 139, 1, 139,
        10, 143, 1, 2, 143, 147, 1, 147, 10, 0, 99, 2, 0, 14, 0,
    ];

    let mut part1 = input.clone();
    part1[1] = 12;
    part1[2] = 2;
    run_program(&mut part1);
    println!("[Solution] Part 1: {}", part1[0]);

    for i in 0..100 {
        for j in 0..100 {
            let mut part2 = input.clone();
            part2[1] = i;
            part2[2] = j;
            run_program(&mut part2);
            if part2[0] == 19690720 {
                let result = 100 * i + j;
                println!("[Solution] Part 2: {}", result);
            }
        }
    }
}
