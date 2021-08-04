# Burraco Seed-rs frontend

Generated from https://github.com/seed-rs/seed-quickstart.git

## 1. Install / check required tools

1. Make sure you have basic tools installed:

   - [Rust](https://www.rust-lang.org)
     - Check: `$ rustc -V` => `rustc 1.43.1 (8d69840ab 2020-05-04)`
     - Install: https://www.rust-lang.org/tools/install
   - [cargo-make](https://sagiegurari.github.io/cargo-make/)
     - Check: `$ cargo make -V` => `cargo-make 0.30.7`
     - Install: `$ cargo install cargo-make`

1. Platform-specific tools like `ssl` and `pkg-config`:
    - Follow recommendations in build errors (during the next chapter).
    - _Note_: Don't hesitate to write notes or a tutorial for your platform and create a PR .

## 2. Prepare your project for work

1. Open the project in your favorite IDE (I recommend [VS Code](https://code.visualstudio.com/) + [Rust Analyzer](https://rust-analyzer.github.io/)).
1. Open a new terminal tab / window and run: `cargo make serve`
1. Open a second terminal tab and run: `cargo make watch`
1. If you see errors, try to fix them or write on our [chat](https://discord.gg/JHHcHp5) or [forum](https://seed.discourse.group/).
1. Modify files like `README.md` and `Cargo.toml` as you wish.

## 3. Write your website

1. Open [localhost:8000](http://localhost:8000) in a browser (I recommend Firefox and Chrome).
1. Modify source files (e.g. `/src/lib.rs` or `/index.html`).
1. Watch compilation in the terminal tab where you run `cargo make watch`.
1. You can watch dev-server responses in the tab where you run `cargo make serve`.
1. Refresh your browser and see changes.
1. Go to step 2.

## 4. Prepare your project for deploy

1. Run `cargo make verify` in your terminal to format and lint the code.
1. Run `cargo make build_release`.
1. Upload `index.html` and `pkg` into your server's public folder.

## Other Seed quickstarts and projects

- [seed-rs/awesome-seed-rs](https://github.com/seed-rs/awesome-seed-rs)
