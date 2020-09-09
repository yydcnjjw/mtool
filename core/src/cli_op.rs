use std::io::Write;
use std::str::FromStr;

fn read_line<T>() -> Result<T, T::Err>
where
    T: FromStr,
{
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("read line failed");
    buffer.trim().parse::<T>()
}

pub fn read_choice(msg: &str, options: &Vec<String>) -> usize {
    println!("{}", msg);
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));
    print!("Input Number: ");
    std::io::stdout().flush().unwrap();
    return read_line().unwrap();
}

#[allow(dead_code)]
pub fn read_choice_cb<F>(msg: &str, options: &Vec<String>, f: F)
where
    F: Fn(usize),
{
    println!("{}", msg);
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));
    print!("Input Number: ");
    std::io::stdout().flush().unwrap();
    f(read_line().unwrap());
}

pub fn read_y_or_n(msg: &str) -> bool {
    print!("{}", msg);
    std::io::stdout().flush().unwrap();

    let c: String = read_line().unwrap();

    match c.to_lowercase().as_ref() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => false,
    }
}
