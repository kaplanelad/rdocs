/// Function to add two numbers and return the result
//START_KEY <id:adding_numbers>
// remove this line
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
//END_KEY

// Function to greet a person by name
//START_KEY <id:greet_person>
fn greet_person(name: &str) {
    println!("Hello, {}! Welcome to the Rust example.", name);
}
//END_KEY

// Main function where the program execution begins
//START_KEY <id:total_example>
fn main() {
    // Call the add_numbers function
    let result = add_numbers(5, 7);
    println!("Result of adding numbers: {}", result);

    // Call the greet_person function
    greet_person("Alice");
}
//END_KEY
