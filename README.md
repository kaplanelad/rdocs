# Rdocs: Code Documentation Made Simple

<!-- 📖introduction -->
## Introduction
 When working on open-source projects or internal projects, creating
 effective documentation is essential. As a developer, I often faced the
 challenge of generating and maintaining comprehensive project documentation.
 The example below demonstrates various types of documentation I've included:

 - Examples showcasing how to use the application configuration.
 - A README guide illustrating the usage of my CLI.
 - Code snippets demonstrating the usage of my library in different
   scenarios.

 However, a common issue is ensuring that the examples provided in the
 documentation remain accurate and up-to-date. What if the example code
 changes? What if there's a typo in the documented code? How can we ensure
 that the examples always reflect the current state of the codebase?

 Imagine a tool that allows you to extract code snippets, ensuring they are
 not only reliable but also executable. What if you could easily incorporate
 pieces of code that are known to work or leverage tested examples from
 languages like Rust, which use `Doctest`? This tool is designed to address
 these concerns.
<!-- introduction📖 -->

<!-- 📖my-goal -->
## Goal
 My objectives are as follows:

 - **Alignment with Code:** Ensure documentation consistently aligns with the
   codebase.
 - **CI Validation:** Incorporate validation into Continuous Integration to
   ensure documentation validity.
 - **User-Friendly:** Prioritize ease of use for all stakeholders.
 - **Minimal Dependencies:** Enable documentation validity even without
   external tool dependencies.
<!-- my-goal📖 -->


## How It Works?
The process begins with your project. Navigate to the section you wish to include in your documentation and enclose it with `// 📖 #START <id:adding_numbers>`. Mark the end of the section with `// 📖 #END`.

For instance, it should resemble this in Rust:
```rust
//📖 #START <id:adding_numbers>
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
//📖 #END
```

Next, navigate to your documentation, such as the readme.md file, and begin the section with `<!-- 📖ID -->`. Conclude this section with `<!-- ID📖 -->`.

Upon executing the command `rdocs replace [COLLECT-FOLDER] [DOC-FOLDER]`, all examples will be gathered and seamlessly replace the corresponding content in your documentation.

## Available commands:
- `rdocs replace`: for replace content replacement
- `rdocs replace --check`: for CI example if you want to validate that the block are equal
- `rdocs collect`: show all block

<!-- 📖installation -->
## Installation

 Cargo install:
 ```sh
 cargo install rdocs
 ```

 GitHub Releases:
 https://github.com/kaplanelad/rdocs/releases/latest
<!-- installation📖 -->



## Example
to see how it work visit [here](./rdocs/src/lib.rs) and see that part of this readme taken from lib.rs file. 