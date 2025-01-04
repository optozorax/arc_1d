use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;
use rand::Rng;
use serde::Serialize;
use rand::prelude::SliceRandom;

// ---------------------------------------------------------------------------

const COLORS: u8 = 9;
const ADD_TRAIN_DATA: bool = false;
const TOTAL_TASKS_COUNT: usize = 1000;

// ---------------------------------------------------------------------------

// This code generates some 1D random riddles, with question and answer.

// Helper functions

fn random_color(rng: &mut StdRng) -> u8 {
    rng.gen_range(1..=COLORS)
}

fn random_color_two(rng: &mut StdRng) -> u8 {
    rng.gen_range(1..=2)
}

fn permute_color(color: u8, rng: &mut StdRng) -> u8 {
    (color + rng.gen_range(1..=COLORS)) % (COLORS + 1)
}

fn permute_color_not_black(color: u8, rng: &mut StdRng) -> u8 {
    let mut answer = permute_color(color, rng);
    while answer == 0 {
        answer = permute_color(color, rng);
    }
    answer
}

fn gen_field(size: usize) -> Vec<u8> {
    gen_field_color(size, 0)
}

fn gen_random_field(size: usize, rng: &mut StdRng) -> Vec<u8> {
    (0..size).map(|_| random_color(rng)).collect()
}

fn gen_random_sparse_field(size: usize, density: f64, rng: &mut StdRng) -> Vec<u8> {
    (0..size).map(|_| if rng.gen_range(0.0..1.0) < density { random_color(rng) } else { 0 }).collect()
}

fn gen_random_sparse_field_two_colors(size: usize, density: f64, rng: &mut StdRng) -> Vec<u8> {
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

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default, Hash, Eq)]
struct Example {
    input: Vec<u8>,
    output: Vec<u8>,
}

// ---------------------------------------------------------------------------

// Actually riddles

fn task_move_n_pix(size: usize, move_pix: usize, solid: bool, rng: &mut StdRng) -> Option<Example> {
    /* some solid block is moved to the right for move_pix pixels */

    if size <= move_pix+1 {
        return None;
    }

    // We generate size and position in such a way, that there is no need to wrap
    let block_size = rng.gen_range(1..size - move_pix);
    let block_pos = rng.gen_range(0..=size - block_size - move_pix);
    let block = if solid {
        gen_field_color(block_size, random_color(rng))
    } else {
        gen_random_field(block_size, rng)
    };

    let question = write_block(block_pos, &block, gen_field(size));
    let answer = write_block(block_pos + move_pix, &block, gen_field(size));

    Some(Example {
        input: question,
        output: answer,
    })
}

fn task_move_n_pix_wrapped(size: usize, move_pix: usize, solid: bool, rng: &mut StdRng) -> Option<Example> {
    /* some solid block is moved to the right for move_pix pixels, and if it exceed borders, it's wrapped to another side of the field */
    let block_size = rng.gen_range(1..size);
    let block_pos = rng.gen_range(0..size);
    let block = if solid {
        gen_field_color(block_size, random_color(rng))
    } else {
        gen_random_field(block_size, rng)
    };

    let question = write_block_wrapped(block_pos, &block, gen_field(size));
    let answer = write_block_wrapped(block_pos + move_pix, &block, gen_field(size));

    Some(Example {
        input: question,
        output: answer,
    })
}

fn task_gravity(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* everything is attracted to the left */
    let question = gen_random_sparse_field(size, 0.5, rng);
    let q = remove_color(0, question.clone());
    let answer = write_block(size - q.len(), &q, gen_field(size));

    Some(Example {
        input: question,
        output: answer,
    })
}

fn task_gravity_counting(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* everything is attracted to the left, and afterwards color is changed to the 1 */
    let question = gen_random_sparse_field(size, 0.5, rng);
    let q_len = remove_color(0, question.clone()).len();
    let block = gen_field_color(q_len, 1);
    let answer = write_block(0, &block, gen_field(size));

    Some(Example {
        input: question,
        output: answer,
    })
}


fn task_gravity_antigravity(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* color 1 is moved to the right, color 2 is to the left */
    let question = gen_random_sparse_field_two_colors(size, 0.5, rng);
    let q1 = remove_color(2, remove_color(0, question.clone()));
    let q2 = remove_color(1, remove_color(0, question.clone()));
    let answer = write_block(0, &q1, write_block(size - q2.len(), &q2, gen_field(size)));

    Some(Example {
        input: question,
        output: answer,
    })
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

// These tasks generated using LLMs

fn task_block_touch_dot(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There is solid block of one color, and one dot of the other color on random place (not on block), block is moved until it touches that dot (not covers it). Color of a dot is not random and constant. */
    let dot_color = 1u8;
    let block_color = permute_color_not_black(dot_color, rng);

    let block_size = rng.gen_range(1..size);
    let dot_pos = rng.gen_range(0..size);

    let can_place_left = dot_pos >= block_size;
    let can_place_right = dot_pos + block_size < size;

    if can_place_left || can_place_right {
        let mut possible_sides = Vec::new();
        if can_place_left {
            possible_sides.push("left");
        }
        if can_place_right {
            possible_sides.push("right");
        }
        let side = possible_sides[rng.gen_range(0..possible_sides.len())];
        let (q_block_pos, a_block_pos) = match side {
            "left" => {
                let qpos = rng.gen_range(0..=dot_pos - block_size);
                (qpos, dot_pos - block_size)
            }
            "right" => {
                let qpos = rng.gen_range(dot_pos + 1..=size - block_size);
                (qpos, dot_pos + 1)
            }
            _ => unreachable!(),
        };

        let block = gen_field_color(block_size, block_color);

        let mut question = gen_field(size);
        question = write_block(q_block_pos, &block, question);
        question[dot_pos] = dot_color;

        let mut answer = gen_field(size);
        answer = write_block(a_block_pos, &block, answer);
        answer[dot_pos] = dot_color;

        return Some(Example {
            input: question,
            output: answer,
        });
    }

    return None;
}

fn task_block_touch_dot_n_pix(size: usize, move_pix: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_block_touch_dot but block is moved only N pixel towards this goal. */
    let dot_color = 2u8;
    let block_color = permute_color_not_black(dot_color, rng);

    let block_size = rng.gen_range(1..size);
    let dot_pos = rng.gen_range(0..size);

    let can_place_left = dot_pos >= block_size;
    let can_place_right = dot_pos + block_size < size;

    if can_place_left || can_place_right {
        let mut possible_sides = Vec::new();
        if can_place_left {
            possible_sides.push("left");
        }
        if can_place_right {
            possible_sides.push("right");
        }
        let side = possible_sides[rng.gen_range(0..possible_sides.len())];

        let block = gen_field_color(block_size, block_color);
        let (q_block_pos, a_block_pos) = match side {
            "left" => {
                let qpos = rng.gen_range(0..=dot_pos - block_size);
                let distance = (dot_pos - block_size).saturating_sub(qpos);
                let d = distance.min(move_pix);
                (qpos, qpos + d)
            }
            "right" => {
                let qpos = rng.gen_range(dot_pos + 1..=size - block_size);
                let distance = qpos.saturating_sub(dot_pos + 1);
                let d = distance.min(move_pix);
                (qpos, qpos - d)
            }
            _ => unreachable!(),
        };

        let mut question = gen_field(size);
        question = write_block(q_block_pos, &block, question);
        question[dot_pos] = dot_color;

        let mut answer = gen_field(size);
        answer = write_block(a_block_pos, &block, answer);
        answer[dot_pos] = dot_color;

        return Some(Example {
            input: question,
            output: answer,
        });
    }

    return None;
}

fn task_block_scale_to_dot(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_block_touch_dot, but block is scaled to this point (it's farthest end remains on its place, but other end it moved to touch the dot). */
    let dot_color = 2u8;
    let block_color = permute_color_not_black(dot_color, rng);

    let block_size = rng.gen_range(1..size);
    let dot_pos = rng.gen_range(0..size);

    let can_place_left = dot_pos >= block_size;
    let can_place_right = dot_pos + block_size < size;

    if can_place_left || can_place_right {
        let mut possible_sides = Vec::new();
        if can_place_left {
            possible_sides.push("left");
        }
        if can_place_right {
            possible_sides.push("right");
        }
        let side = possible_sides[rng.gen_range(0..possible_sides.len())];

        let q_block_pos = match side {
            "left" => rng.gen_range(0..=dot_pos - block_size),
            "right" => rng.gen_range(dot_pos + 1..=size - block_size),
            _ => unreachable!(),
        };

        let block = gen_field_color(block_size, block_color);

        let mut question = gen_field(size);
        question = write_block(q_block_pos, &block, question);
        question[dot_pos] = dot_color;

        let (a_block_pos, scaled_size) = match side {
            "left" => {
                let new_size = dot_pos - q_block_pos + 1;
                (q_block_pos, new_size)
            }
            "right" => {
                let new_size = (q_block_pos + block_size) - dot_pos;
                (dot_pos, new_size)
            }
            _ => unreachable!(),
        };

        let scaled_block = gen_field_color(scaled_size, block_color);

        let mut answer = gen_field(size);
        answer = write_block(a_block_pos, &scaled_block, answer);
        answer[dot_pos] = dot_color;

        return Some(Example {
            input: question,
            output: answer,
        });
    }

    return None;
}

fn task_two_points_and_fill(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are only two points of the same color, and in answer between them everything is filled with this color. */
    let c = random_color(rng);
    let p1 = rng.gen_range(0..size);
    let p2 = rng.gen_range(0..size);

    if p1 == p2 {
        return None;
    }
    let (pos1, pos2) = if p1 < p2 { (p1, p2) } else { (p2, p1) };

    let mut question = gen_field(size);
    question[pos1] = c;
    question[pos2] = c;

    let mut answer = question.clone();
    for i in pos1..=pos2 {
        answer[i] = c;
    }

    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_reflect_block_with_border_pixel(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* We have solid block with one pixel at left or right border of diferent color, we just reflect that block. */
    let block_size = rng.gen_range(2..=size);
    if block_size > size {
        return None; 
    }

    let c1 = random_color(rng);
    let c2 = random_color(rng);
    if c1 == c2 {
        return None; 
    }

    let side = if rng.gen_bool(0.5) { "left" } else { "right" };
    let pos = rng.gen_range(0..=size - block_size);
    let mut block = gen_field_color(block_size, c1);
    match side {
        "left" => block[0] = c2,
        "right" => block[block_size - 1] = c2,
        _ => unreachable!(),
    }

    let question = write_block(pos, &block, gen_field(size));
    let reversed_block: Vec<u8> = block.iter().rev().copied().collect();
    let answer = write_block(pos, &reversed_block, gen_field(size));

    return Some(Example { input: question, output: answer });
}

fn task_reflect_block_with_border_pixel_random(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_reflect_block_with_border_pixel, but with block consists of random colors. */
    let block_size = rng.gen_range(2..=size);
    if block_size > size {
        return None;
    }

    let side = if rng.gen_bool(0.5) { "left" } else { "right" };
    let pos = rng.gen_range(0..=size - block_size);
    let mut block = gen_random_field(block_size, rng);
    let border_color = random_color(rng);
    match side {
        "left" => {
            if block[0] == border_color {
                return None;
            }
            block[0] = border_color;
        }
        "right" => {
            if block[block_size - 1] == border_color {
                return None;
            }
            block[block_size - 1] = border_color;
        }
        _ => unreachable!(),
    }
    let question = write_block(pos, &block, gen_field(size));
    let reversed_block: Vec<u8> = block.iter().rev().copied().collect();
    let answer = write_block(pos, &reversed_block, gen_field(size));

    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_reflect_block_around_dot(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* We have some constant color dot, and block somewhere. And we just reflect that block around this dot. */
    let dot_color = 2u8;

    let dot_pos = rng.gen_range(0..size);
    let block_size = rng.gen_range(1..=size);
    let block_pos = rng.gen_range(0..=size - block_size);
    let block_end = block_pos + block_size - 1;
    let strictly_left  = block_end < dot_pos;
    let strictly_right = block_pos > dot_pos;

    if !(strictly_left || strictly_right) {
        return None;
    }
    let block = gen_field_color(block_size, permute_color_not_black(dot_color, rng));
    let min_reflect = 2 * dot_pos as isize - block_end as isize;
    let max_reflect = 2 * dot_pos as isize - block_pos as isize;
    if min_reflect < 0 || max_reflect >= size as isize {
        return None;
    }
    let mut question = gen_field(size);
    question = write_block(block_pos, &block, question);
    question[dot_pos] = dot_color;
    let mut answer = gen_field(size);
    answer[dot_pos] = dot_color;
    for i in 0..block_size {
        let reflect_idx = (2 * dot_pos) as isize - (block_pos + i) as isize;
        answer[reflect_idx as usize] = block[i];
    }

    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_block_and_noise_remove(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* We have some block and some noise of the same color, and we remove that noise (make sure that noise do not create block with size 2, or that noise does not touch block, so that's impossible to restore original picture). */
    let c = random_color(rng);
    let block_size = rng.gen_range(2..=size);
    if block_size > size {
        return None;
    }
    let block_pos = rng.gen_range(0..=size - block_size);
    let mut field = gen_field(size);
    for i in 0..block_size {
        field[block_pos + i] = c;
    }
    let mut forbidden = vec![false; size];
    for i in block_pos..(block_pos + block_size) {
        forbidden[i] = true;
    }
    if block_pos > 0 {
        forbidden[block_pos - 1] = true;
    }
    if block_pos + block_size < size {
        forbidden[block_pos + block_size] = true;
    }
    let noise_count = rng.gen_range(1..=3);
    let mut noise_positions = vec![];

    for _ in 0..noise_count {
        let allowed_positions: Vec<usize> =
            (0..size).filter(|&i| !forbidden[i]).collect();

        if allowed_positions.is_empty() {
            break;
        }
        let noise_pos = allowed_positions[rng.gen_range(0..allowed_positions.len())];
        noise_positions.push(noise_pos);
        field[noise_pos] = c;
        forbidden[noise_pos] = true;
        if noise_pos > 0 {
            forbidden[noise_pos - 1] = true;
        }
        if noise_pos + 1 < size {
            forbidden[noise_pos + 1] = true;
        }
    }
    if noise_positions.len() < noise_count {
        return None;
    }
    let question = field.clone();
    let mut answer = field.clone();
    for &p in &noise_positions {
        answer[p] = 0;
    }

    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_block_and_noise_remove_inside(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_block_and_noise_remove, but noise is inside the block, and it has different colors. */
    if size <= 6 {
        return None;
    }
    let c = random_color(rng);
    let block_size = rng.gen_range(6..=size);
    if block_size > size {
        return None;
    }
    let block_pos = rng.gen_range(0..=size - block_size);
    let mut field = gen_field(size);
    for i in 0..block_size {
        field[block_pos + i] = c;
    }
    let max_noise = (block_size / 2-1).max(1);
    let noise_count = rng.gen_range(1..=max_noise);
    let mut indices: Vec<usize> = (0..block_size).collect();
    indices.shuffle(rng);
    let noise_positions: Vec<usize> = indices.into_iter().take(noise_count).collect();
    for &offset in &noise_positions {
        let pos = block_pos + offset;
        let noise_color = permute_color_not_black(c, rng);
        field[pos] = noise_color;
    }
    let question = field.clone();
    let mut answer = field.clone();
    for &offset in &noise_positions {
        let pos = block_pos + offset;
        answer[pos] = c;
    }

    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_copy_block_to_dots(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are block of some odd size (3 or 5) at the start and then some dots. We just copy this block to center of the each dot (dots should be on a distance that resulting blocks does not touch and does not overlap). Color of block and dots is the same, but overall random. */
    let block_size = if rng.gen_bool(0.5) { 3 } else { 5 };
    if block_size >= size { return None; }
    
    let color = random_color(rng);
    let block = gen_field_color(block_size, color);
    
    // Generate dots with minimum distance to prevent overlap
    let min_gap = block_size;
    let mut dot_positions = Vec::new();
    let mut pos = block_size + block_size/2 + 1;
    
    while pos <= size - block_size {
        if rng.gen_bool(0.5) { // Control dot density
            dot_positions.push(pos);
            pos += min_gap;
        }
        pos += 1;
    }
    
    if dot_positions.is_empty() { return None; }
    
    let mut question = gen_field(size);
    question = write_block(0, &block, question);
    for &pos in &dot_positions {
        question[pos] = color;
    }
    
    let mut answer = gen_field(size);
    answer = write_block(0, &block, answer);
    for &pos in &dot_positions {
        let block_start = pos - block_size/2;
        answer = write_block(block_start, &block, answer);
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_copy_block_to_dots_colors(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_copy_block_to_dots, but color of everything is different, and when we place block, we just copy that color. */
    let block_size = if rng.gen_bool(0.5) { 3 } else { 5 };
    if block_size >= size { return None; }
    
    let block_color = random_color(rng);
    let block = gen_field_color(block_size, block_color);
    
    // Generate dots with minimum distance to prevent overlap
    let min_gap = block_size;
    let mut dot_positions = Vec::new();
    let mut dot_colors = Vec::new();
    let mut pos = block_size + block_size/2 + 1;
    
    while pos < size - block_size {
        if rng.gen_bool(0.5) {
            let dot_color = random_color(rng);
            dot_positions.push(pos);
            dot_colors.push(dot_color);
            pos += min_gap;
        }
        pos += 1;
    }
    
    if dot_positions.is_empty() { return None; }
    
    let mut question = gen_field(size);
    question = write_block(0, &block, question);
    for (i, &pos) in dot_positions.iter().enumerate() {
        question[pos] = dot_colors[i];
    }
    
    let mut answer = gen_field(size);
    answer = write_block(0, &block, answer);
    for (i, &pos) in dot_positions.iter().enumerate() {
        let block_start = pos - block_size/2;
        let colored_block = gen_field_color(block_size, dot_colors[i]);
        answer = write_block(block_start, &colored_block, answer);
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_paint_biggest_block(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* We have some amount of blocks of some constant color, and we just paint biggest of them to another constant color. */
    let target_color = 1u8;
    let initial_color = permute_color_not_black(target_color, rng);
    if initial_color == 0 {
        return None;
    }
    
    // Generate random blocks
    let mut question = gen_field(size);
    let mut blocks = Vec::new();
    let mut pos = 0;
    
    while pos < size {
        if rng.gen_bool(0.4) && size - pos >= 2 {
            let block_size = rng.gen_range(2..=((size - pos).min(6)));
            blocks.push((pos, block_size));
            for i in 0..block_size {
                question[pos + i] = initial_color;
            }
            pos += block_size + 1;
        } else {
            pos += 1;
        }
    }
    
    if blocks.len() < 2 { return None; }  // Need at least two blocks
    
    // Find biggest block
    let (biggest_pos, biggest_size) = *blocks.iter()
        .max_by_key(|(_pos, size)| *size)
        .unwrap();

    let biggest_count = blocks.iter()
        .filter(|(_pos, size)| *size == biggest_size)
        .count();

    if biggest_count > 1 {
        return None;
    }
        
    // Create answer by recoloring the biggest block
    let mut answer = question.clone();
    for i in 0..biggest_size {
        answer[biggest_pos + i] = target_color;
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_sort_blocks_by_size(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are many blocks of the same color and different length. We sort them by size. After sorting smallest block is on left side, and distance between blocks equal to 1. */
    let color = random_color(rng);
    let mut blocks = Vec::new();
    let mut pos = 0;
    
    // Generate random blocks with random sizes
    while pos < size {
        if rng.gen_bool(0.4) && size - pos >= 2 {
            let block_size = rng.gen_range(1..=((size - pos).min(6)));
            blocks.push((pos, block_size));
            pos += block_size + rng.gen_range(1..=4); // Random gaps between blocks
        } else {
            pos += 1;
        }
    }
    
    if blocks.len() < 2 { return None; } // Need at least two blocks
    
    // Create input field
    let mut question = gen_field(size);
    for &(pos, size) in &blocks {
        for i in 0..size {
            question[pos + i] = color;
        }
    }
    
    // Sort blocks by size
    blocks.sort_by_key(|&(_, size)| size);
    
    // Check if sorted blocks fit with gaps
    let total_space = blocks.iter().map(|&(_, size)| size).sum::<usize>() + blocks.len() - 1;
    if total_space > size { return None; }
    
    // Create answer field with sorted blocks
    let mut answer = gen_field(size);
    let mut current_pos = 0;
    
    for &(_, block_size) in &blocks {
        for i in 0..block_size {
            answer[current_pos + i] = color;
        }
        current_pos += block_size + 1; // One pixel gap
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_sort_complete_sequence(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Same as task_sort_blocks_by_size, but there are all block sizes from 1 to maximum possible number (how much size allows), with gap size 1. And there are random permutation of this on the input. */
    // Calculate max possible block size given total array size
    let mut max_size = 1;
    let mut total_space = 0;
    while total_space + max_size + 1 <= size {
        total_space += max_size + 1;
        max_size += 1;
    }
    if max_size < 2 { return None; }
    max_size -= 1;
    
    let color = random_color(rng);
    
    // Create sequence of all sizes from 1 to max_size
    let mut blocks: Vec<usize> = (1..=max_size).collect();
    blocks.shuffle(rng);
    
    // Create input field with shuffled blocks
    let mut question = gen_field(size);
    let mut pos = 0;
    for &block_size in &blocks {
        for i in 0..block_size {
            question[pos + i] = color;
        }
        pos += block_size + 1;
    }
    
    // Create answer field with sorted blocks
    let mut answer = gen_field(size);
    let mut pos = 0;
    for block_size in 1..=max_size {
        for i in 0..block_size {
            answer[pos + i] = color;
        }
        pos += block_size + 1;
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_recolor_blocks_by_size(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are two blocks of random size (not equal size) of color 3. Biggest block painted as color 1, smallest block painted as color 2. */
    // Generate two different random sizes
    let size1 = rng.gen_range(2..=8);
    let mut size2 = rng.gen_range(2..=8);
    while size2 == size1 {
        size2 = rng.gen_range(2..=8);
    }
    
    // Ensure both blocks fit with at least 1 gap
    if size1 + size2 + 1 > size { return None; }
    
    // Place blocks with gap
    let pos1 = rng.gen_range(0..=size - (size1 + size2 + 1));
    let pos2 = rng.gen_range(pos1 + size1 + 1..=size - size2);
    
    // Create input field with both blocks color 3
    let mut question = gen_field(size);
    for i in 0..size1 {
        question[pos1 + i] = 3;
    }
    for i in 0..size2 {
        question[pos2 + i] = 3;
    }
    
    // Create answer field with recolored blocks
    let mut answer = question.clone();
    if size1 > size2 {
        for i in 0..size1 { answer[pos1 + i] = 1; }
        for i in 0..size2 { answer[pos2 + i] = 2; }
    } else {
        for i in 0..size1 { answer[pos1 + i] = 2; }
        for i in 0..size2 { answer[pos2 + i] = 1; }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_gravity_one_step(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Gravity to the left, but any point can be moved only 1 pixel towards left. */
    let question = gen_random_sparse_field(size, 0.5, rng);
    let mut answer = question.clone();
    
    // Move each non-zero pixel one step left if possible
    for i in 1..size {
        if answer[i] != 0 && answer[i-1] == 0 {
            answer[i-1] = answer[i];
            answer[i] = 0;
        }
    }
    
    Some(Example {
        input: question,
        output: answer,
    })
}

fn task_move_block_by_own_size(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There is only one solid block. It's moved to the right for size of that block pixels. */
    let block_size = rng.gen_range(1..=size/2);  // Ensure space for movement
    let pos = rng.gen_range(0..=size - block_size * 2);  // Space for block and movement
    let color = random_color(rng);
    
    let mut question = gen_field(size);
    let block = gen_field_color(block_size, color);
    question = write_block(pos, &block, question);
    
    let answer = write_block(pos + block_size, &block, gen_field(size));
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_change_to_five(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Every non-zero color is changed to color 5. */
    let question = gen_random_sparse_field(size, 0.5, rng);
    let answer: Vec<u8> = question.iter()
        .map(|&x| if x != 0 { 5 } else { 0 })
        .collect();
    
    Some(Example {
        input: question,
        output: answer,
    })
}

fn task_recolor_blocks_from_palette(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are many blocks with same size (range: 2..=4) and color 5. And then there is random colors (except 0) at the most left corner. There are as many these colors as there blocks. In output all these blocks are recolored according to this colors. */
    // Generate blocks of same size
    let block_size = rng.gen_range(2..=4);
    let mut blocks = Vec::new();
    let mut pos = 0;
    
    while pos + block_size <= size {
        if rng.gen_bool(0.4) {
            blocks.push(pos);
            pos += block_size + 1;
        } else {
            pos += 1;
        }
    }

    loop {
        let palette_size = blocks.len();
        if blocks.last()? + block_size + palette_size + 1 >= size {
            blocks.pop();
        } else {
            break;
        }
    }

    let palette_size = blocks.len();
    for block in &mut blocks {
        *block += palette_size + 1;
    }
    
    if blocks.is_empty() { return None; }
    
    // Generate color palette
    let mut colors: Vec<u8> = Vec::new();
    for _ in 0..blocks.len() {
        let mut color;
        loop {
            color = random_color(rng);
            if !colors.contains(&color) { break; }
        }
        colors.push(color);
    }
    
    // Create question with color palette and blocks
    let mut question = gen_field(size);
    
    // Place color palette at start
    for (i, &color) in colors.iter().enumerate() {
        question[i] = color;
    }
    
    // Place blocks of color 5
    for &block_pos in &blocks {
        for i in 0..block_size {
            question[block_pos + i] = 5;
        }
    }
    
    // Create answer with recolored blocks
    let mut answer = question.clone();
    for (block_idx, &block_pos) in blocks.iter().enumerate() {
        let color = colors[block_idx];
        for i in 0..block_size {
            answer[block_pos + i] = color;
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_duplicate_block_from_seeds(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are one block with size at least 2 with color 1. And then there one pixel on right or on left or on both sides with random color, with distance 1 to that block. And block is being duplicated, starting from each pixel with this pixel color, and repeated indefinitely until it reaches end of the field. Block can be drawn partially on border (draw block while current position is inside field). */
    let block_size = rng.gen_range(2..=4);
    if block_size + 1 >= size { return None; }
    if size <= 3 + block_size { return None; }
    
    // Position block with space for seeds
    let block_pos = rng.gen_range(2..size - block_size - 1);
    
    // Decide seed placement
    let left_seed = rng.gen_bool(0.5);
    let right_seed = rng.gen_bool(0.5);
    if !left_seed && !right_seed { return None; }
    
    // Create input
    let mut question = gen_field(size);
    
    // Place main block
    for i in 0..block_size {
        question[block_pos + i] = 1;
    }
    
    // Place seeds with gaps
    let mut seeds = Vec::new();
    if left_seed {
        let color = random_color(rng);
        question[block_pos - 2] = color;
        seeds.push(("left", block_pos - 2, color));
    }
    if right_seed {
        let color = random_color(rng);
        question[block_pos + block_size + 1] = color;
        seeds.push(("right", block_pos + block_size + 1, color));
    }
    
    // Create answer with duplicated blocks
    let mut answer = question.clone();
    
    for &(side, seed_pos, color) in &seeds {
        match side {
            // For left seed, blocks end at seed
            "left" => {
                let mut end_pos = seed_pos as i32;
                while end_pos >= 0 {
                    let start_pos = end_pos - block_size as i32 + 1;
                    for pos in start_pos..=end_pos {
                        if pos >= 0 && (pos as usize) < size {
                            answer[pos as usize] = color;
                        }
                    }
                    if start_pos < 1 { break; }
                    end_pos = start_pos - 2; // -1 for gap
                }
            },
            // For right seed, blocks start at seed
            "right" => {
                let mut start_pos = seed_pos;
                while start_pos < size {
                    for offset in 0..block_size {
                        if start_pos + offset >= size { break; }
                        answer[start_pos + offset] = color;
                    }
                    if start_pos + block_size + 1 >= size { break; }
                    start_pos = start_pos + block_size + 1; // +1 for gap
                }
            },
            _ => unreachable!(),
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_fill_from_pixel(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There is some solid block of random color with size at least 3, and one pixel with random color on left or right side. This pixel fills right or left side with its color. */
    let block_size = rng.gen_range(3..=6);
    if block_size >= size - 2 { return None; }
    
    // Position block with space for seed
    let block_pos = rng.gen_range(1..size - block_size - 1);
    
    // Create input
    let mut question = gen_field(size);
    
    // Place main block
    let block_color = random_color(rng);
    for i in 0..block_size {
        question[block_pos + i] = block_color;
    }
    
    // Place seed pixel and determine fill direction
    let seed_color = permute_color_not_black(block_color, rng);
    let is_left = rng.gen_bool(0.5);
    
    if is_left {
        question[block_pos - 1] = seed_color;
    } else {
        question[block_pos + block_size] = seed_color;
    }
    
    // Create answer with fill
    let mut answer = question.clone();
    
    if is_left {
        // Fill from seed to left border
        for i in 0..block_pos {
            answer[i] = seed_color;
        }
    } else {
        // Fill from seed to right border
        for i in (block_pos + block_size)..size {
            answer[i] = seed_color;
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_mark_size_two_blocks(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are many blocks of size from 1 to 3 with gap at least 2 with color 1. Each block with size 2 is surrounded with pixels of color 3 at its sides. */
    let mut blocks = Vec::new();
    let mut pos = 0;
    
    // Generate blocks with minimum gap of 2
    while pos < size {
        if rng.gen_bool(0.4) {
            let block_size = rng.gen_range(1..=3);
            // Check if we have space for block and potential markers
            let needed_space = block_size + if block_size == 2 { 2 } else { 0 };
            if pos + needed_space < size {
                blocks.push((pos, block_size));
                pos += block_size + 2; // Minimum gap of 2
            }
        }
        pos += 1;
    }
    
    if blocks.len() < 2 { return None; }
    
    // Verify gaps between blocks (including markers)
    let mut valid = true;
    for i in 0..blocks.len()-1 {
        let (pos1, size1) = blocks[i];
        let (pos2, _) = blocks[i+1];
        let needed_gap = if size1 == 2 { 3 } else { 2 };
        if pos2 - (pos1 + size1) < needed_gap {
            valid = false;
            break;
        }
    }
    if !valid { return None; }
    
    // Create input with blocks
    let mut question = gen_field(size);
    for &(pos, block_size) in &blocks {
        // Place block
        for i in 0..block_size {
            question[pos + i] = 1;
        }
    }
    
    // Create answer with markers
    let mut answer = question.clone();
    for &(pos, block_size) in &blocks {
        if block_size == 2 {
            // Add markers for size 2 blocks
            if pos > 0 {
                answer[pos - 1] = 3;
            }
            if pos + block_size < size {
                answer[pos + block_size] = 3;
            }
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_color_left_half_blocks(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are many blocks with size from 2 to 8 with gap 1 and color 2. On the output left half of them is colored to color 8. */
    let mut pos = 0;
    let mut question = gen_field(size);
    let mut blocks = Vec::new();
    
    // Generate blocks with gap 1
    while pos < size {
        if rng.gen_bool(0.4) {
            let block_size = rng.gen_range(2..=8);
            if pos + block_size >= size { break; }
            
            blocks.push((pos, block_size));
            for i in 0..block_size {
                question[pos + i] = 2;
            }
            pos += block_size + 1; // block size + gap
        } else {
            pos += 1;
        }
    }
    
    if blocks.len() < 2 { return None; }
    
    // Create answer with half-colored blocks
    let mut answer = question.clone();
    for &(pos, block_size) in &blocks {
        let half_size = block_size / 2;
        for i in 0..half_size {
            answer[pos + i] = 8;
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_fill_until_collision(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* There are one pixel at left or right side with color 5, and there are couple of pixels with random color. Each pixel fills empty space with its color in the direction of pixel on the side, until it reaches another pixel. */
    // At least 4 positions for meaningful puzzle
    if size < 4 { return None; }
    
    let is_left = rng.gen_bool(0.5);
    let mut question = gen_field(size);
    
    // Place the side marker
    if is_left {
        question[0] = 5;
    } else {
        question[size - 1] = 5;
    }
    
    // Place 2-4 random pixels
    let num_pixels = rng.gen_range(2..=4);
    let mut positions = Vec::new();
    
    if is_left {
        // Skip first position
        for _ in 0..num_pixels {
            let mut pos;
            loop {
                pos = rng.gen_range(1..size);
                if !positions.contains(&pos) { break; }
            }
            positions.push(pos);
        }
    } else {
        // Skip last position
        for _ in 0..num_pixels {
            let mut pos;
            loop {
                pos = rng.gen_range(0..size-1);
                if !positions.contains(&pos) { break; }
            }
            positions.push(pos);
        }
    }
    
    // Color random pixels
    for &pos in &positions {
        question[pos] = permute_color_not_black(5, rng);
    }
    
    positions.sort_unstable();
    
    // Create answer
    let mut answer = question.clone();
    
    if is_left {
        // Fill right from each pixel
        let mut prev_pos = 0; // Start from marker
        for &pos in &positions {
            let color = question[pos];
            // Fill from previous position to current
            for i in (prev_pos + 1)..pos {
                answer[i] = color;
            }
            prev_pos = pos;
        }
    } else {
        // Fill right from each pixel
        let mut prev_pos = size-1; // Start from marker
        for &pos in positions.iter().rev() {
            let color = question[pos];
            // Fill from previous position to current
            for i in pos+1..prev_pos {
                answer[i] = color;
            }
            prev_pos = pos;
        }
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_repeat_pattern_full(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* At left corner there are set of random pixels with size in range 2..=5. And it's repeated two times. In output this pattern should be repeated as many times as possible. After repetition, there should be no 0 pixels. */
    // Generate initial pattern
    let pattern_size = rng.gen_range(2..=5);
    let mut pattern = Vec::with_capacity(pattern_size);
    for _ in 0..pattern_size {
        pattern.push(random_color(rng));
    }
    
    // Calculate total size needed for 2 repetitions
    let double_size = pattern_size * 2;
    if double_size >= size { return None; }
    
    // Create input with 2 repetitions
    let mut question = gen_field(size);
    for i in 0..pattern_size {
        question[i] = pattern[i];
        question[i + pattern_size] = pattern[i];
    }
    
    // Create answer with maximum repetitions
    let mut answer = gen_field(size);
    let mut pos = 0;
    while pos + pattern_size <= size {
        for i in 0..pattern_size {
            answer[pos + i] = pattern[i];
        }
        pos += pattern_size;
    }
    
    // Fill remaining space (if any) with pattern elements
    for i in pos..size {
        answer[i] = pattern[i - pos];
    }
    
    return Some(Example {
        input: question,
        output: answer,
    });
}

fn task_gravity_weighted_colors(size: usize, rng: &mut StdRng) -> Option<Example> {
    /* Gravity to the left from the random field, but color 2 is heavier than color 1, so it's always all pixels of color 2, then pixels of color 1 at the output. */
    // Generate random field with only colors 1 and 2
    let question = (0..size).map(|_| {
        if rng.gen_bool(0.5) {
            if rng.gen_bool(0.5) { 1 } else { 2 }
        } else {
            0
        }
    }).collect::<Vec<_>>();
    
    // Count colors
    let count_1 = question.iter().filter(|&&x| x == 1).count();
    let count_2 = question.iter().filter(|&&x| x == 2).count();
    
    // Create answer with sorted colors
    let mut answer = gen_field(size);
    
    // Place heavier color 2 first
    for i in 0..count_2 {
        answer[i] = 2;
    }
    
    // Then place color 1
    for i in 0..count_1 {
        answer[count_2 + i] = 1;
    }
    
    Some(Example {
        input: question,
        output: answer,
    })
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

fn task_mirror(example: Option<Example>) -> Option<Example> {
    let mut example = example?;
    example.input.reverse();
    example.output.reverse();
    Some(example)
}

fn task_inverse(example: Option<Example>) -> Option<Example> {
    let mut example = example?;
    std::mem::swap(&mut example.input, &mut example.output);
    Some(example)
}

fn task_identity(example: Option<Example>) -> Option<Example> {
    example
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


fn generate_task<F: FnMut(usize, &mut StdRng) -> Option<Example>>(rng: &mut StdRng, mut f: F) -> Vec<Example> {
    let mut examples = HashSet::new();
    for _ in 0..(TOTAL_TASKS_COUNT * 2) {
        let size = rng.gen_range(5..30);
        let res = f(size, rng);
        if let Some(task) = res {
            examples.insert(task);
        }
        if examples.len() >= TOTAL_TASKS_COUNT {
            break;
        }
    }
    let mut examples = examples.into_iter().collect::<Vec<_>>();
    examples.shuffle(rng);
    examples
}

fn mkdir(dir: &str) {
    if !std::fs::exists(dir).unwrap() {
        std::fs::create_dir(dir).unwrap();
    }
}

fn save_task(name: &str, examples: Vec<Example>) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    mkdir("tasks");
    let mut tasks = vec![];
    for example in examples.iter() {
        let task = ArcTask2D {
            train: if ADD_TRAIN_DATA { 
                examples
                    .choose_multiple(&mut rng, 4)
                    .cloned()
                    .filter(|x| x != example)
                    .take(3)
                    .map(|x| Example2D::from(x))
                    .collect()
            } else {
                vec![] 
            },
            test: vec![example.clone().into()],
        };
        tasks.push(task);
    }
    save_json_to_file(&tasks, &format!("tasks/{name}.json"));
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

fn get_background_mask(grid: &[i64], background: i64) -> Vec<bool> {
    grid.iter().map(|&i| i == background).collect()
}

fn count_non_background(mask: &[bool]) -> usize {
    mask.iter().filter(|&&x| !x).count()
}

fn count_colors(grid: &[i64]) -> Vec<usize> {
    let mut res = vec![0; COLORS as usize + 1];
    for &i in grid {
        res[i as usize] += 1;
    }
    res
}

fn empty_palette() -> Vec<bool> {
    vec![false; COLORS as usize + 1]
}

fn convert_to_palette(colors_count: &[usize]) -> Vec<bool> {
    colors_count.iter().map(|&count| count != 0).collect()
}

fn or_palette(palette1: &[bool], palette2: &[bool]) -> Vec<bool> {
    palette1.iter().zip(palette2.iter())
        .map(|(&a, &b)| a || b)
        .collect()
}

fn get_new_colors(palette_prev: &[bool], palette_new: &[bool]) -> Vec<bool> {
    palette_prev.iter().zip(palette_new.iter())
        .map(|(&prev, &new)| !prev && new)
        .collect()
}

fn is_subset_palette(palette: &[bool], palette_sub: &[bool]) -> bool {
    palette.iter().zip(palette_sub.iter()).all(|(&full, &sub)| !sub || full)
}

fn calc_invariants(task_datas: &[Value]) -> (bool, bool, bool, bool, bool, Vec<bool>, Vec<bool>) {
    let mut same_mask_all = true;
    let mut same_count_all = true;
    let mut same_colors_all = true;
    let mut same_palette_all = true;
    let mut subset_palette_all = true;
    let mut palette_output_all = empty_palette();
    let mut palette_new_colors_all = empty_palette();
    let background = 0;

    for task_data in task_datas {
        if let Some(test_cases) = task_data["test"].as_array() {
            for test_case in test_cases {
                let input = test_case["input"][0].as_array().unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect::<Vec<_>>();
                
                let output = test_case["output"][0].as_array().unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect::<Vec<_>>();

                let mask_input = get_background_mask(&input, background);
                let count_input = count_non_background(&mask_input);
                let mut colors_input = count_colors(&input);
                colors_input[background as usize] = 0;
                let palette_input = convert_to_palette(&colors_input);

                let mask_output = get_background_mask(&output, background);
                let count_output = count_non_background(&mask_output);
                let mut colors_output = count_colors(&output);
                colors_output[background as usize] = 0;
                let palette_output = convert_to_palette(&colors_output);

                let same_mask = mask_input == mask_output;
                let same_count = count_input == count_output;
                let same_colors = colors_input == colors_output;
                let same_palette = palette_input == palette_output;
                let subset_palette = is_subset_palette(&palette_input, &palette_output);

                same_mask_all &= same_mask;
                same_count_all &= same_count;
                same_colors_all &= same_colors;
                same_palette_all &= same_palette;
                subset_palette_all &= subset_palette;

                let palette_new_colors = get_new_colors(&palette_input, &palette_output);

                palette_output_all = or_palette(&palette_output_all, &palette_output);
                palette_new_colors_all = or_palette(&palette_new_colors_all, &palette_new_colors);
            }
        }
    }

    (
        same_mask_all,
        same_count_all,
        same_colors_all,
        same_palette_all,
        subset_palette_all,
        palette_output_all,
        palette_new_colors_all
    )
}

fn create_palette_html(text: &str, palette: &[bool]) -> String {
    if !palette.iter().any(|x| *x) {
        return Default::default();
    }
    let mut html = format!(r#"<div>{text}:</div><div class="palette">"#);
    for (i, &present) in palette.iter().enumerate() {
        if present {
            html.push_str(&format!(r#"<div class="cell color-{}"></div>"#, i));
        }
    }
    html.push_str("</div>");
    html
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::Value;

const CELL_SIZE: u32 = 15;
const INDEX_TAKE_JSONS: usize = 5;
const CSS_TEMPLATE: &str = r#"
@import url("https://fonts.googleapis.com/css2?family=Anonymous+Pro:ital,wght@0,400;0,700;1,400;1,700");

@font-face {
    font-family: 'AtariClassicChunky';
    src: url('https://arcprize.org/media/fonts/AtariClassicChunky.eot');
    src: url('https://arcprize.org/media/fonts/AtariClassicChunky.eot?#iefix') format('embedded-opentype'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.woff2') format('woff2'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.woff') format('woff'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.svg#AtariClassicChunky') format('svg');
    font-weight: normal;
    font-style: normal;
    font-display: swap;
}
:root {
    --white: #EEEEEE;
    --offwhite: #C0C0C0;
    --black: #000000;
    --magenta: #E53AA3;
    --magenta-light: #ff7bcc;
    --red: #F93C31;
    --blue: #1E93FF;
    --blue-light: #87D8F1;
    --yellow: #FFDC00;
    --orange: #FF851B;
    --maroon: #921231;
    --green: #4FCC30;
    --gray: #555555;
    --gray-light: #999999;
}

body {
    background-color: var(--black);
    color: var(--white);
    font-family: 'Anonymous Pro', monospace;
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: 0;
    padding: 20px;
}

h1 {
    font-family: 'AtariClassicChunky', monospace;
    color: var(--magenta);
    margin-bottom: 30px;
}

h3 {
    word-break: break-all;
    word-wrap: anywhere;
    white-space: normal;
    height: 35pt;
    margin: 0px;
}

.task-container {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    width: 100%;
    max-width: 2200px;
    justify-content: center;
}

.task {
    flex: 0 1 auto;
    min-width: 200px;
    // max-width: 400px;

    background-color: var(--black);
    padding: 10px;

    border: 0.5px solid var(--gray);
}

.subtask {
    flex: 0 1 auto;
    min-width: 200px;
    // max-width: 400px;

    background-color: var(--black);
    padding: 10px;
}

.task-title {
    color: var(--offwhite);
    margin-bottom: 3px;
    font-size: 14px;
}

.grid-container {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.grid {
    display: grid;
    gap: 1px;
}

.cell {
    width: 15px;
    height: 15px;
    border: 0.2px solid var(--gray);
}

.color-0 { background-color: var(--black); }
.color-1 { background-color: var(--blue); }
.color-2 { background-color: var(--red); }
.color-3 { background-color: var(--green); }
.color-4 { background-color: var(--yellow); }
.color-5 { background-color: var(--gray-light); }
.color-6 { background-color: var(--magenta); }
.color-7 { background-color: var(--orange); }
.color-8 { background-color: var(--blue-light); }
.color-9 { background-color: var(--maroon); }

a {
    color: var(--blue);
    text-decoration: none;
}

a:hover {
    color: var(--blue-light);
}

p {
    margin: 0px;
}

.invariants-container {
    margin-top: 10px;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.invariant {
    display: inline-block;
    margin: 2px 5px;
    padding: 2px 6px;
    background-color: #262626;
    border-radius: 3px;
    font-size: 0.9em;
}

.palette-container {
    margin-top: 10px;
}

.palette {
    display: flex;
    gap: 2px;
    margin: 5px 0;
}
"#;

fn create_grid_html(data: &[i64], columns: usize) -> String {
    let mut grid_html = format!(
        r#"<div class="grid" style="grid-template-columns: repeat({}, {}px);">"#,
        columns, CELL_SIZE
    );
    
    for &cell in data {
        grid_html.push_str(&format!(r#"<div class="cell color-{}"></div>"#, cell));
    }
    grid_html.push_str("</div>");
    grid_html
}

fn create_task_html(task_data: &Value, task_name: &str) -> String {
    let input_data = task_data["test"][0]["input"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect::<Vec<_>>();
    
    let output_data = task_data["test"][0]["output"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect::<Vec<_>>();
    
    let columns = input_data.len();
    
    format!(
        r#"
        <div class="subtask">
            <div class="task-title">{}</div>
            <div class="grid-container">
                {}
                {}
            </div>
        </div>
        "#,
        task_name,
        create_grid_html(&input_data, columns),
        create_grid_html(&output_data, columns)
    )
}

fn generate_single_task_page(task_path: &Path, output_dir: &Path) -> std::io::Result<PathBuf> {
    let task_name = task_path.file_name().unwrap().to_string_lossy();

    let content = fs::read_to_string(&task_path)?;
    let all_files: Vec<Value> = serde_json::from_str(&content)?;
    
    let mut task_html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ARC Task: {}</title>
            <style>{}</style>
        </head>
        <body>
            <a href="index.html">go back to all tasks</a>
            <h1>{}</h1>
            <h3>({} files)</h3>
            <div class="task-container">
        "#,
        task_name, CSS_TEMPLATE, task_name, all_files.len()
    );
    
    for (i, task_data) in all_files.iter().enumerate() {
        task_html.push_str(&create_task_html(&task_data, &i.to_string()));
    }
    
    task_html.push_str(
        r#"
            </div>
        </body>
        </html>
        "#
    );
    
    let output_path = output_dir.join(format!("{}.html", task_name));
    let mut file = File::create(&output_path)?;
    file.write_all(task_html.as_bytes())?;
    
    Ok(output_path)
}

fn generate_index_page(tasks_dir: &Path, output_dir: &Path) -> std::io::Result<()> {
    let mut index_html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ARC Tasks Overview</title>
            <style>{}</style>
        </head>
        <body>
            <h1>ARC Tasks Overview</h1>
            <div class="task-container">
        "#,
        CSS_TEMPLATE
    );
    
    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        let task_dir = path.file_name().unwrap().to_string_lossy();
        let content = fs::read_to_string(&path)?;
        let all_files: Vec<Value> = serde_json::from_str(&content)?;

        if !all_files.is_empty() {

            index_html.push_str(&format!(
                r#"<div class="task"><h3><a href="{}.html">{}</a></h3>"#,
                task_dir, task_dir.strip_suffix(".json").unwrap()
            ));

            let (same_mask_all, same_count_all, same_colors_all, same_palette_all, subset_palette_all, palette_output_all, palette_new_colors_all) = calc_invariants(&all_files);
            let mut invariants_html = String::new();
            if same_mask_all {
                invariants_html.push_str("<div class='invariant'>Same mask</div>");
            }
            if same_count_all {
                invariants_html.push_str("<div class='invariant'>Same count</div>");
            }
            if same_colors_all {
                invariants_html.push_str("<div class='invariant'>Same colors</div>");
            }
            if same_palette_all {
                invariants_html.push_str("<div class='invariant'>Same palette</div>");
            }
            if subset_palette_all {
                invariants_html.push_str("<div class='invariant'>Subset palette</div>");
            }
            index_html.push_str(&format!(
                r#"
                <div class="invariants-container">
                    {}
                    <div class="palette-container">
                        {}
                        {}
                    </div>
                </div>
                "#,
                invariants_html,
                create_palette_html("Output palette", &palette_output_all),
                create_palette_html("New colors", &palette_new_colors_all)
            ));
            
            let files_count = all_files.len();
            index_html.push_str(&format!(r#"<center><p>({} files)</p></center>"#, files_count));
            
            for (i, json_file) in all_files.iter().enumerate().take(INDEX_TAKE_JSONS) {
                index_html.push_str(&create_task_html(&json_file, &i.to_string()));
            }
            index_html.push_str("</div>");
        }
    }
    
    index_html.push_str(
        r#"
            </div>
        </body>
        </html>
        "#
    );
    
    let mut file = File::create(output_dir.join("index.html"))?;
    file.write_all(index_html.as_bytes())?;
    Ok(())
}

fn draw() -> std::io::Result<()> {
    let tasks_dir = Path::new("tasks");
    let output_dir = Path::new("visualization");
    
    fs::create_dir_all(output_dir)?;
    
    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        generate_single_task_page(&path, output_dir)?;
    }
    
    generate_index_page(tasks_dir, output_dir)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

fn main() {
    color_backtrace::install();

    fs::create_dir_all("tasks").unwrap();

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let mirrors = [("right", task_identity as fn(Option<Example>) -> Option<Example>), ("left", task_mirror as fn(Option<Example>) -> Option<Example>)];
    let inverses = [("", task_identity as fn(Option<Example>) -> Option<Example>), ("_inv", task_inverse as fn(Option<Example>) -> Option<Example>)];

    for pixels in 1..=4 {
        save_task(&format!("block_touch_dot_{pixels}_pix"), generate_task(&mut rng, |size, rng| task_block_touch_dot_n_pix(size, pixels, rng)));
        for (dir, conversion) in mirrors.clone() {
            for (style, solid) in [("solid", true), ("colorful", false)] {
                save_task(&format!("move_{pixels}pix_{style}_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_move_n_pix(size, pixels, solid, rng))));
                save_task(&format!("move_{pixels}pix_{style}_{dir}_wrapped"), generate_task(&mut rng, |size, rng| conversion(task_move_n_pix_wrapped(size, pixels, solid, rng))));
            }
        }
    }

    for (dir, conversion) in mirrors.clone() {
        save_task(&format!("gravity_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_gravity(size, rng))));
        save_task(&format!("gravity_antigravity_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_gravity_antigravity(size, rng))));
        save_task(&format!("gravity_counting_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_gravity_counting(size, rng))));
        save_task(&format!("gravity_one_step_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_gravity_one_step(size, rng))));
        save_task(&format!("move_block_by_own_size_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_move_block_by_own_size(size, rng))));
        save_task(&format!("gravity_weighted_colors_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_gravity_weighted_colors(size, rng))));
        save_task(&format!("color_left_half_blocks_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_color_left_half_blocks(size, rng))));
        save_task(&format!("recolor_blocks_from_palette_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_recolor_blocks_from_palette(size, rng))));
        save_task(&format!("sort_complete_sequence_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_sort_complete_sequence(size, rng))));
        save_task(&format!("sort_blocks_by_size_{dir}"), generate_task(&mut rng, |size, rng| conversion(task_sort_blocks_by_size(size, rng))));
    }

    for (name, conversion) in inverses.clone() {
        save_task(&format!("two_points_and_fill{name}"), generate_task(&mut rng, |size, rng| conversion(task_two_points_and_fill(size, rng))));
    }

    save_task("block_touch_dot", generate_task(&mut rng, |size, rng| task_block_touch_dot(size, rng)));
    save_task("block_scale_to_dot", generate_task(&mut rng, |size, rng| task_block_scale_to_dot(size, rng)));
    save_task("two_points_and_fill_inv", generate_task(&mut rng, |size, rng| task_inverse(task_two_points_and_fill(size, rng))));
    save_task("reflect_block_with_border_pixel", generate_task(&mut rng, |size, rng| task_reflect_block_with_border_pixel(size, rng)));
    save_task("reflect_block_random", generate_task(&mut rng, |size, rng| task_reflect_block_with_border_pixel_random(size, rng)));
    save_task("reflect_block_around_dot", generate_task(&mut rng, |size, rng| task_reflect_block_around_dot(size, rng)));
    save_task("block_and_noise_remove", generate_task(&mut rng, |size, rng| task_block_and_noise_remove(size, rng)));
    save_task("block_and_noise_remove_inside", generate_task(&mut rng, |size, rng| task_block_and_noise_remove_inside(size, rng)));
    save_task("copy_block_to_dots", generate_task(&mut rng, |size, rng| task_copy_block_to_dots(size, rng)));
    save_task("copy_block_to_dots_colors", generate_task(&mut rng, |size, rng| task_copy_block_to_dots_colors(size, rng)));
    save_task("paint_biggest_block", generate_task(&mut rng, |size, rng| task_paint_biggest_block(size, rng)));
    save_task("recolor_blocks_by_size", generate_task(&mut rng, |size, rng| task_recolor_blocks_by_size(size, rng)));
    save_task("change_to_five", generate_task(&mut rng, |size, rng| task_change_to_five(size, rng)));
    save_task("duplicate_block_from_seeds", generate_task(&mut rng, |size, rng| task_duplicate_block_from_seeds(size, rng)));
    save_task("fill_from_pixel", generate_task(&mut rng, |size, rng| task_fill_from_pixel(size, rng)));
    save_task("mark_size_two_blocks", generate_task(&mut rng, |size, rng| task_mark_size_two_blocks(size, rng)));
    save_task("fill_until_collision", generate_task(&mut rng, |size, rng| task_fill_until_collision(size, rng)));
    save_task("repeat_pattern_full", generate_task(&mut rng, |size, rng| task_repeat_pattern_full(size, rng)));

    draw().unwrap();
}
