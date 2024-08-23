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
      Welcome to level 1!
    next:
      caption: GO!
      to: lev2
  lev2:
    legend: |
      Guess a number from 1 to 10.
    next:
      caption: I guessed it!
      to: lev3
  lev3:
    key: !exact 7
    legend: |
      it is 7. Well, how about something more difficult?
      Guess a number between 1 and 100.

      Tip: h0w 4b0ut l00k1ng t0 th1s t3xt?
    next:
      caption: Submit
      to: success
  success:
    key: !exact 40
    legend: |
      That is it!
strings:
  name: demo quest
```
## How to run

Run:
```sh
path/to/qs path/to/config.yml
```