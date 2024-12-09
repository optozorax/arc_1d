use std::collections::HashSet;
use rand::Rng;
use serde::Serialize;

const COLORS: u8 = 2;

fn random_color(rng: &mut impl Rng) -> u8 {
    rng.gen_range(1..=COLORS)
}

fn permute_color(color: u8, rng: &mut impl Rng) -> u8 {
    (color + rng.gen_range(1..=COLORS)) % (COLORS + 1)
}

fn gen_field(size: usize) -> Vec<u8> {
    (0..size).map(|_| 0).collect()
}

fn write_block(pos: usize, size: usize, color: u8, mut field: Vec<u8>) -> Vec<u8> {
    for i in 0..size {
        field[pos + i] = color;
    }
    field
}

fn print(x: &[u8], name: &str) {
    print!("{name}: ");
    for i in x {
        print!("{i}");
    }
    println!();
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default, Hash, Eq)]
struct Example {
    input: Vec<u8>,
    output: Vec<u8>,
}

fn mirror(mut example: Example) -> Example {
    example.input.reverse();
    example.output.reverse();
    example
}

fn remove_duplicates_examples(examples: &mut Vec<Example>) {
    let mut seen = HashSet::new();
    examples.retain(|example| seen.insert(example.clone()));
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default)]
struct Example2D {
    input: Vec<Vec<u8>>,
    output: Vec<Vec<u8>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default)]
struct ArcTask2D {
    train: Vec<Example2D>,
    test: Vec<Example2D>,
}

impl From<Example> for Example2D {
    fn from(x: Example) -> Self {
        Example2D {
            input: vec![x.input],
            output: vec![x.output],
        }
    }
}

pub fn save_json_to_file<T: Serialize>(t: &T, name: &str) {
    use std::io::Write;
    let mut file = std::fs::File::create(name).unwrap();
    let json = serde_json::to_string(&t).unwrap();
    write!(file, "{}", json).unwrap();
}

fn task_1d_move_n_pix(size: usize, move_pix: usize, rng: &mut impl Rng) -> Example {
    let block_size = rng.gen_range(1..size - move_pix);
    let block_pos = rng.gen_range(0..=size - block_size - move_pix);
    let color = random_color(rng);

    let question = write_block(block_pos, block_size, color, gen_field(size));
    let answer = write_block(block_pos + move_pix, block_size, color, gen_field(size));

    Example {
        input: question,
        output: answer,
    }
}

fn generate_task<F: FnMut() -> Example>(mut f: F) -> Vec<Example> {
    let mut examples = vec![];
    for _ in 0..1000 {
        examples.push(f());
    }
    remove_duplicates_examples(&mut examples);
    examples.sort_by_key(|x| x.input.clone());
    examples
}

fn mkdir(dir: &str) {
    if !std::fs::exists(dir).unwrap() {
        std::fs::create_dir(dir).unwrap();
    }
}

fn save_task(name: &str, examples: Vec<Example>) {
    mkdir("tasks");
    mkdir(&format!("tasks/{name}"));
    for (i, example) in examples.into_iter().enumerate() {
        let task = ArcTask2D {
            train: vec![],
            test: vec![example.into()],
        };

        save_json_to_file(&task, &format!("tasks/{name}/{i}.json"));
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let size = 12;

    save_task("move_1pix_right", generate_task(|| task_1d_move_n_pix(size, 1, &mut rng)));
    save_task("move_2pix_right", generate_task(|| task_1d_move_n_pix(size, 2, &mut rng)));
    save_task("move_3pix_right", generate_task(|| task_1d_move_n_pix(size, 3, &mut rng)));
    save_task("move_4pix_right", generate_task(|| task_1d_move_n_pix(size, 4, &mut rng)));
    save_task("move_1pix_left", generate_task(|| mirror(task_1d_move_n_pix(size, 1, &mut rng))));
    save_task("move_2pix_left", generate_task(|| mirror(task_1d_move_n_pix(size, 2, &mut rng))));
    save_task("move_3pix_left", generate_task(|| mirror(task_1d_move_n_pix(size, 3, &mut rng))));
    save_task("move_4pix_left", generate_task(|| mirror(task_1d_move_n_pix(size, 4, &mut rng))));
}
