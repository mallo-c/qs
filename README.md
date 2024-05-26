# qs
qs is a quest engine.
## How to build qs
```sh
git clone https://github.com/mallo-c/qs
cargo build --release
```
A binary file `qs` will appear in `target/release` directory.
## Example configuration
```yaml
levels:
  start:
    legend: |
      <p>Welcome to level 1!</p>
    next:
      caption: GO!
      to: lev2
  lev2:
    legend: |
      <p>Guess a number from 1 to 10.</p>
    next:
      caption: I guessed it!
      to: lev3
  lev3:
    key: 7
    legend: |
      <p>it is 7. Well, how about something more difficult?
      Guess a number between 1 and 100.</p>
      <p>Tip: h0w 4b0ut l00k1ng t0 th1s t3xt?</p>
    next:
      caption: Submit
      to: success
  success:
    key: 40
    legend: |
      That is it!
strings:
  name: demo quest
```
## How to run

Run:
```sh
path/to/qs --config path/to/config.yml
```