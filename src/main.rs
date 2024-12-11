use std::collections::HashSet;
use rand::Rng;
use serde::Serialize;

const COLORS: u8 = 2;

fn random_color(rng: &mut impl Rng) -> u8 {
    rng.gen_range(1..=COLORS)
}

fn random_color_two(rng: &mut impl Rng) -> u8 {
    rng.gen_range(1..=2)
}

fn permute_color(color: u8, rng: &mut impl Rng) -> u8 {
    (color + rng.gen_range(1..=COLORS)) % (COLORS + 1)
}

fn gen_field(size: usize) -> Vec<u8> {
    gen_field_color(size, 0)
}

fn gen_random_field(size: usize, rng: &mut impl Rng) -> Vec<u8> {
    (0..size).map(|_| random_color(rng)).collect()
}

fn gen_random_sparse_field(size: usize, density: f64, rng: &mut impl Rng) -> Vec<u8> {
    (0..size).map(|_| if rng.gen_range(0.0..1.0) < density { random_color(rng) } else { 0 }).collect()
}

fn gen_random_sparse_field_two_colors(size: usize, density: f64, rng: &mut impl Rng) -> Vec<u8> {
    (0..size).map(|_| if rng.gen_range(0.0..1.0) < density { random_color_two(rng) } else { 0 }).collect()
}

fn gen_field_color(size: usize, color: u8) -> Vec<u8> {
    (0..size).map(|_| color).collect()
}

fn write_block(pos: usize, block: &[u8], mut field: Vec<u8>) -> Vec<u8> {
    for (i, color) in block.iter().enumerate() {
        field[pos + i] = *color;
    }
    field
}

fn remove_color(color: u8, field: Vec<u8>) -> Vec<u8> {
    field.into_iter().filter(|x| *x != color).collect::<Vec<_>>()
}

fn write_block_wrapped(pos: usize, block: &[u8], mut field: Vec<u8>) -> Vec<u8> {
    let len = field.len();
    for (i, color) in block.iter().enumerate() {
        field[(pos + i) % len] = *color;
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

fn task_mirror(mut example: Example) -> Example {
    example.input.reverse();
    example.output.reverse();
    example
}

fn task_identity(example: Example) -> Example {
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

fn task_move_n_pix(size: usize, move_pix: usize, solid: bool, rng: &mut impl Rng) -> Example {
    let block_size = rng.gen_range(1..size - move_pix);
    let block_pos = rng.gen_range(0..=size - block_size - move_pix);
    let block = if solid {
        gen_field_color(block_size, random_color(rng))
    } else {
        gen_random_field(block_size, rng)
    };

    let question = write_block(block_pos, &block, gen_field(size));
    let answer = write_block(block_pos + move_pix, &block, gen_field(size));

    Example {
        input: question,
        output: answer,
    }
}

fn task_move_n_pix_wrapped(size: usize, move_pix: usize, solid: bool, rng: &mut impl Rng) -> Example {
    let block_size = rng.gen_range(1..size);
    let block_pos = rng.gen_range(0..size);
    let block = if solid {
        gen_field_color(block_size, random_color(rng))
    } else {
        gen_random_field(block_size, rng)
    };

    let question = write_block_wrapped(block_pos, &block, gen_field(size));
    let answer = write_block_wrapped(block_pos + move_pix, &block, gen_field(size));

    Example {
        input: question,
        output: answer,
    }
}

fn task_gravity(size: usize, rng: &mut impl Rng) -> Example {
    let question = gen_random_sparse_field(size, 0.5, rng);
    let q = remove_color(0, question.clone());
    let answer = write_block(size - q.len(), &q, gen_field(size));

    Example {
        input: question,
        output: answer,
    }
}

fn task_gravity_counting(size: usize, rng: &mut impl Rng) -> Example {
    let question = gen_random_sparse_field(size, 0.5, rng);
    let q_len = remove_color(0, question.clone()).len();
    let block = gen_field_color(q_len, 1);
    let answer = write_block(0, &block, gen_field(size));

    Example {
        input: question,
        output: answer,
    }
}


fn task_gravity_antigravity(size: usize, rng: &mut impl Rng) -> Example {
    let question = gen_random_sparse_field_two_colors(size, 0.5, rng);
    let q1 = remove_color(2, remove_color(0, question.clone()));
    let q2 = remove_color(1, remove_color(0, question.clone()));
    let answer = write_block(0, &q1, write_block(size - q2.len(), &q2, gen_field(size)));

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

    let mirrors = [("right", task_identity as fn(Example) -> Example), ("left", task_mirror as fn(Example) -> Example)];

    for pixels in 1..=4 {
        for (dir, conversion) in mirrors.clone() {
            for (style, solid) in [("solid", true), ("colorful", false)] {
                save_task(&format!("move_{pixels}pix_{style}_{dir}"), generate_task(|| conversion(task_move_n_pix(size, pixels, solid, &mut rng))));
                save_task(&format!("move_{pixels}pix_{style}_{dir}_wrapped"), generate_task(|| conversion(task_move_n_pix_wrapped(size, pixels, solid, &mut rng))));
            }
        }
    }

    for (dir, conversion) in mirrors.clone() {
        save_task(&format!("gravity_{dir}"), generate_task(|| conversion(task_gravity(size, &mut rng))));
        save_task(&format!("gravity_antigravity_{dir}"), generate_task(|| conversion(task_gravity_antigravity(size, &mut rng))));
        save_task(&format!("gravity_counting_{dir}"), generate_task(|| conversion(task_gravity_counting(size, &mut rng))));
        
    }
}
