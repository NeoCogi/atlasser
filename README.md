# Atlasser
The atlasser for the microui/neocogi UI

## Build

Small size build

```
cargo +nightly run --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu
```
