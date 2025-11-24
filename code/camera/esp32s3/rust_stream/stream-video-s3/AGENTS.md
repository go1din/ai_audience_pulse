After every step run
export PATH="$HOME/.cargo/bin:$PATH" && export RUSTUP_TOOLCHAIN=esp && source ~/export-esp.sh && unbuffer timeout 40s cargo run
and if expected output is wrong fix the problem