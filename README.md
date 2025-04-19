# twitter-rust-practice

## How to Run Rust Code

1. Ensure you have Rust installed. If you are using the provided development container, Rust is already pre-installed.
2. Open a terminal in the project directory.
3. Use the following command to run a Rust file (replace `main.rs` with your file name):
   ```bash
   rustc main.rs && ./main
   ```
4. For projects with a `Cargo.toml` file, you can use Cargo to build and run the project:
   ```bash
   cargo run
   ```

## How to Run the Backend Server

1. Ensure you have Rust installed. If you are using the provided development container, Rust is already pre-installed.
2. Open a terminal in the project directory.
3. Use the following command to run the backend server:
   ```bash
   cargo run
   ```

## Example of Sending a POST Request

You can use `curl` to send a POST request to the server. Here is an example:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"user":"example_user","content":"Hello, world!"}' http://127.0.0.1:8080/post_tweet
```
