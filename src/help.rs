pub static TEXT: [&str; 2] = [r#"Normal Mode | Visual Mode
h | <- - left
j | down arrow - down
k | up arrow - up
l | -> left
+ - add row
d - delete row
y - yank current cell
p - paste yanked cell
P - paste yanked cell before
= - add column
- - delete column
g - go to specific line or to start
G - go to specific line or to end
Ctr + e - open editor
Ctr + r - run program
any number - times to repeat command
q - quit program
Ctr + s - save project
Ctr + o - load project
i - enter insert mode
v - enter visual mode
: - enter command mode
Insert Mode
r - reset value to randome one
any number - append to selected
"#,
r#"0: Layer new Notes relative to previous
1: Layer new note Additive
2: use Constant Frequency for one line
3: use Constant Duration for one line
4: use Constant Velocity for one line
5: Repeat Note
6: Send Parameters
7: Override current Frequency with constant value
8: Override current Duration with constant value
9: Override current Velocity with constant value
10: Don't override current values
11: Slice current note
12: use random Frequency for one line
13: use random Duration for one line
14: use random Velocity for one line
15: use random Frequency, Duration, Velocity for one line
16: override current Frequency to random
17: override current Duration to random
18: override current Velocity to random
19: override Frequency, Duration, Velocity to random
"#];
