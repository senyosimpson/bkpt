@default:
  just --list

objdump:
  cargo build --manifest-path ../hw/Cargo.toml --release
  objdump -d ../hw/target/release/hw --disassemble=main

bkpt:
  cargo build
  ./target/debug/bkpt -- ../hw/target/release/hw 
