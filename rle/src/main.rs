fn encode(input: &[u8]) -> String {
    let mut index = 0;
    let mut matches_count = 1;

    let mut output = String::new();

    for &b in input {
        index += 1;
        if let Some(&next_b) = input.get(index) {
            if b == next_b {
                matches_count += 1;
                continue;
            }
        }

        if matches_count > 1 {
            output.push_str(&format!(
                "{matches_count}{}",
                std::char::from_u32(b as u32).unwrap()
            ));

            matches_count = 1;
        } else {
            output.push(std::char::from_u32(b as u32).unwrap());
        }
    }

    output
}

fn main() {
    let input = b"WWWWWWWWWWWWBWWWWWWWWWWWWBBBWWWWWWWWWWWWWWWWWWWWWWWWB";

    println!("{}", encode(input));
}
