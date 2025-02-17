use std::collections::HashMap;

fn decrypt(input: &[u32]) -> String {
    if input.is_empty() {
        return "".into();
    }

    let mut map: HashMap<u32, String> = (0..=255)
        .map(|i| (i, (i as u8 as char).to_string()))
        .collect();

    let mut prev = map[&input[0]].clone();
    let decoded = prev.clone();

    input
        .iter()
        .skip(1)
        .fold(
            (256, decoded), // (starting code, initial decoded string)
            |(code, mut decoded_acc), &c| {
                let current_str = if map.contains_key(&c) {
                    map[&c].clone()
                } else {
                    format!("{}{}", prev, prev.chars().next().unwrap())
                };

                decoded_acc.push_str(&current_str);

                map.insert(
                    code,
                    format!("{}{}", prev, current_str.chars().next().unwrap()),
                );

                prev = current_str;

                (code + 1, decoded_acc)
            },
        )
        .1
}

fn encrypt(input: &str) -> Vec<u32> {
    if input.is_empty() {
        return vec![];
    }

    let mut map: HashMap<String, u32> = (0..=255)
        .map(|i| ((i as u8 as char).to_string(), i))
        .collect();

    // SAFETY:
    //  we checked that it's not empty, so we know that there is at least one emelent
    let mut cursor = input.chars().next().unwrap().to_string();

    //// we don't want the first character because it's in the cursor
    let mut enc = input
        .chars()
        .skip(1)
        .fold((Vec::new(), 256), |(mut acc, mut code), c| {
            let combined = cursor.clone() + &c.to_string();
            if map.contains_key(&combined) {
                cursor.push(c);
            } else {
                acc.push(*map.get(&cursor).unwrap());
                map.insert(combined, code);
                code += 1;

                cursor = c.to_string();
            }

            (acc, code)
        })
        .0;
    enc.push(*map.get(&cursor).unwrap());
    enc
}

fn main() {
    let enc = encrypt("hello");
    let dec = decrypt(&enc);

    assert_eq!(dec.as_str(), "hello");
}
