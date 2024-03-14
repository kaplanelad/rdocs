/// Function to add two numbers and return the result
//📖 #START <id:adding_numbers>
// remove this line
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
//📖 #END

// Function to greet a person by name
//📖 #START <id:greet_person>
fn greet_person(name: &str) {
    println!("Hello, {}! Welcome to the Rust example.", name);
}
//📖 #END

// Main function where the program execution begins
//📖 #START <id:total_example>
fn main() {
    // Call the add_numbers function
    let result = add_numbers(5, 7);
    println!("Result of adding numbers: {}", result);

    // Call the greet_person function
    greet_person("Alice");
}
//📖 #END
