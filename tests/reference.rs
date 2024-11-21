//! Builds a reference book into a set of slides, and checks they match what we expect.

use std::path::Path;

#[test]
fn build_slides() {
    let slide_template_string = include_str!("./data_in/template.html");
    let index_template_string = include_str!("./data_in/index_template.html");
    println!("We are in: {}", std::env::current_dir().unwrap().display());
    mdslides::run(
        Some(Path::new("tests/data_in")),
        Path::new("tests/data_out"),
        slide_template_string,
        Some(index_template_string),
    )
    .expect("mdslides failed");

    let comparison = folder_compare::FolderCompare::new(
        Path::new("tests/data_out"),
        Path::new("tests/reference_out"),
        &vec![],
    )
    .expect("failed to compare");
    if !comparison.changed_files.is_empty() {
        for filename in comparison.changed_files {
            let contents = std::fs::read_to_string(&filename).unwrap();
            eprintln!(
                "Changed File {filename}:\n{contents}",
                filename = filename.display()
            );
        }
        panic!("Some files differ");
    }
    if !comparison.new_files.is_empty() {
        for filename in comparison.new_files {
            let contents = std::fs::read_to_string(&filename).unwrap();
            eprintln!(
                "New File {filename}:\n{contents}",
                filename = filename.display()
            );
        }
        panic!("Some new files found");
    }
}
