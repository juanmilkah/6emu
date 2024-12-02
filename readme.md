### Small 8086 emulator
It cannot run the following instructions due to skill issues
- ```IN```
- ```OUT```
- ```LOCK```
- ```WAIT```


#### Build
```bash
cargo build
```

We can execute the binary directly or serve
to use HTML gui

#### Serve at localhost:8023
```python
cd emu8086
python3 server.py -f target/debug/emu8086
```
#### Direct usage
```
nasm -f  bin -o code.bin my_file.s
emu8086 -f code.bin

```