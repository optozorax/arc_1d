# 1D ARC-AGI generator

DATASET VISUALIZATION: <https://optozorax.github.io/arc_1d/>

This repository contains code to generate 1D ARC-AGI like tasks. Almost all the tasks from [1D-ARC](https://github.com/khalil-research/1D-ARC) is reimplemented here.

Currently it have 32 unique tasks and 75 tasks if we count augmentation (reflection, inversion, different parameters in task (pixel offset)).

Visualization of all the tasks is generated automatically along with json files.

Why 1D? Because it's order of magnitude smaller than 2D tasks and by using this dataset you can iterate fast for your solving approaches and use less compute. If your model cannot solve 1D tasks, it definitely cannot solve 2D tasks, so this repository can be good starting point.

My main motivation for this was:
* Control grid size, so I can use small grid (currently 12) for faster training.
* Generate as many examples as possible.
* Write many tasks using LLM by the description.

How to use:
* [Install Rust](https://www.rust-lang.org/)
* `cargo run --release`

Current limitations:
* Grid size is fixed for all tasks and examples. If you want different grid sizes, then current code cannot do that. Open an issue for such option if you really need that (why?).
* In json, the main data that you should look at is "test" array. And it's visualized and generated here. "train" array is automatically generated from 3 random examples (in "test" position) from the current task dataset. And it's not guaranteed that it's possible to derive transformation rule from those 3 examples.

How tasks are generated: I just ask LLM to write a code of a task by textual description (it's written in the first line of each function), and then check this task in visualization. If you want to contribute, you may do the same.
