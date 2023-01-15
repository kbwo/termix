Termix is a framework for building TUI application inspired by [bubbletea](https://github.com/charmbracelet/bubbletea). Termix's interfaces are very similar to it.


See example
```
cargo run --example <simple | views>
```

### WIP
- [ ] Mouse support
- [ ] Some useful plugins
- [ ] More customizable interface
- [ ] Windows support
and more

## References

`Termix` borrows ideas from lots of other projects:

- [bubbletea](https://github.com/charmbracelet/bubbletea)
  - Architecture and interface inspiration.
  - How to render.
- [tuikit](https://github.com/lotabout/tuikit)
  - How to enter the raw mode.
  - Part of the keycode parsing logic.
  - Also inspired by [termion](https://gitlab.redox-os.org/redox-os/termion), [rustyline](https://github.com/kkawakam/rustyline).
