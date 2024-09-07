# InteractiveAUdioEditor

How to install && run

```
git clone https://github.com/indieleet/iaue
cd iaue
cargo run --release
```

TODO
- [x] add support for cli
- [x] add row highlight
- [x] add support for themes
- [x] add init values for every row
- [x] render file to wav
- [x] fix render of cell
- [x] effects for notes
    - [x] 0: Layer new Notes relative to previous
    - [x] 1: Layer new note Additive
    - [x] 2: use Constant Frequency for one line
    - [x] 3: use Constant Duration for one line
    - [x] 4: use Constant Velocity for one line
    - [x] 5: Repeat Note
    - [x] 6: Send Parameters
    - [x] 7: Override current Frequency with constant value
    - [x] 8: Override current Duration with constant value
    - [x] 9: Override current Velocity with constant value
    - [x] 10: Don't override current values
    - [x] 11: Slice current note
    - [x] 12: use random Frequency for one line
        - [ ] add bounds for random
    - [x] 13: use random Duration for one line
    - [x] 14: use random Velocity for one line
    - [x] 15: use random Parameters for one line
- [x] add fxes on tracks
- [x] add support for cargo dirs
- [x] fix rows bounds
    - [ ] fix it in insert mode
- [x] stereo
    - [ ] now add mono mode for better performance
- [ ] add sidechain
- [ ] put render in different thread
- [ ] better workflow
    - [x] add yanking
    - [x] add pasting
    - [x] add labels
    - [x] add row numbers
    - [x] add init values
    - [ ] faster insert mode edit
    - [ ] make d work in visual mode
    - [ ] make + work in visual mode
    - [ ] make - work in visual mode
    - [ ] make = work in visual mode
    - [ ] paste count of times
    - [ ] add multitabs
    - [ ] add "swap cells" key
- [ ] add constant frequency, length, etc mode
- [ ] more generative features
    - [ ] random builtin instruments and fxes
- [ ] add tutorial
- [ ] document commands
- [ ] remove every unwrap
- [ ] audio editor
- [ ] add oscilloscope
- [ ] add realtime mode
- [ ] sunvox support
- [ ] pure data support
- [ ] dawproject file export
- [ ] remove some dependencies
    - [ ] change serde to nanoserde
questionable:
- [ ] change (frequency, len, velocity, params) to (phase, params)
